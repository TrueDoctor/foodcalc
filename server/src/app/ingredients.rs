//! Ingredient use-cases: filtered list/search, usages and ownership transfer.

use super::audit;
use super::{assert_ownership_transfer, AppCtx, MyAppState, Result};

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::ingredient::IngredientWithSource;
use foodlib_new::entities::recipe::Recipe;
use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{
    GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId, UnitId,
};
use foodlib_new::ingredient::{Ingredient, IngredientSource, IngredientWeight};
use foodlib_new::ops::ingredients::IngredientRepoFilter;
use foodlib_new::policy::{GlobalPermission, IngredientPerm, Permission};
use foodlib_new::property::{IngredientProperties, Property};
use foodlib_new::types::money::Money;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct IngredientListQuery {
    /// Case-insensitive substring search on the name.
    pub search: Option<String>,
    /// Only ingredients carrying this property (e.g. a spice tag).
    pub property_id: Option<PropertyId>,
}

/// ingredients visible to the caller using all readable groups
/// with sources, optionally filtered by text/property.
pub async fn list_ingredients(
    state: &MyAppState,
    ctx: &AppCtx,
    query: IngredientListQuery,
) -> Result<Vec<IngredientWithSource>> {
    let readable_groups: Vec<GroupId> = ctx.ctx.policy.group_permissions.keys().copied()
        .filter(|&gid| {
            ctx.ctx
                .has_group_permission(gid, Permission::Ingredient(IngredientPerm::View))
        })
        .collect();

    if readable_groups.is_empty() {
        return Ok(Vec::new());
    }

    state
        .ingredients()
        .list_filtered_with_sources(IngredientRepoFilter {
            group_ids: Some(&readable_groups),
            search: query.search.as_deref(),
            property_id: query.property_id,
        })
        .await
}


pub async fn usages(
    state: &MyAppState,
    ctx: &AuthCtx,
    id: IngredientId
) -> Result<Vec<Recipe>> {
    let recipes = state.ingredients().usages(id).await?;
    // TODO: Maybe add a filter here to only return recipes that the user has permission to view.
    //   But then the view might be empty even if the ingredient is used in recipes, which could be confusing.
    Ok(recipes)
}

#[derive(Debug, Deserialize)]
pub struct IngredientCreateInput {
    pub name: String,
    /// Energy density in kJ/g.
    pub energy: String,
    pub comment: Option<String>,
    pub group_id: GroupId,
}

#[derive(Debug, Deserialize)]
pub struct IngredientUpdateInput {
    pub name: Option<String>,
    pub energy: Option<String>,
    /// `null` clears the comment, absent keeps it.
    pub comment: Option<Option<String>>,
}

#[derive(Debug, Serialize)]
pub struct IngredientDetailDto {
    pub ingredient: Ingredient,
    pub sources: Vec<IngredientSource>,
    /// Per-unit gram weights for amount conversions.
    pub weights: Vec<IngredientWeight>,
    /// Allergen / dietary properties aggregated for this ingredient.
    pub properties: Vec<Property>,
}

pub async fn create_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    payload: IngredientCreateInput,
) -> Result<Ingredient> {
    actor.ctx.assert_global(GlobalPermission::CreateResources)?;
    actor.ctx.assert_group_perm(payload.group_id, Permission::Ingredient(IngredientPerm::Create))?;

    let energy = parse_amount(&payload.energy, "energy")?;

    let ingredient = Ingredient {
        id: IngredientId::default(),
        name: payload.name,
        energy,
        comment: payload.comment,
        group_id: payload.group_id,
    };
    state.ingredients().create(ingredient).await
}

pub async fn get_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    id: IngredientId,
) -> Result<IngredientDetailDto> {
    let ingredient = state.ingredients().get(id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::View))?;

    let sources = state.ingredients().get_sources(id).await?;
    let weights = state.ingredients().get_weights(id).await?;
    let properties = state.properties().get_ingredient_properties(id).await?.properties;

    Ok(IngredientDetailDto {
        ingredient,
        sources,
        weights,
        properties,
    })
}

pub async fn update_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    id: IngredientId,
    payload: &IngredientUpdateInput,
) -> Result<Ingredient> {
    let current = state.ingredients().get(id).await?;
    actor.ctx.assert_group_perm(current.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    let energy = match &payload.energy {
        Some(energy_str) => Some(parse_amount(energy_str, "energy")?),
        None => None,
    };

    let updated = Ingredient {
        id: current.id,
        name: payload.name.clone().unwrap_or(current.name),
        energy: energy.unwrap_or(current.energy),
        comment: payload.comment.clone().unwrap_or(current.comment),
        group_id: current.group_id,
    };
    let result = state.ingredients().update(updated).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient"),
        Some(id.into()),
        Some(current.group_id),
        None,
        Some(json!(result)),
    )
        .await;

    Ok(result)
}

pub async fn delete_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    id: IngredientId,
) -> Result<()> {
    let current = state.ingredients().get(id).await?;
    actor.ctx.assert_group_perm(current.group_id, Permission::Ingredient(IngredientPerm::Delete))?;

    let usages = state.ingredients().usages(id).await?;
    if !usages.is_empty() {
        return Err(foodlib_new::Error::InvalidInput {
            message: format!("cannot delete ingredient {}: it is used in {} recipe(s)", id, usages.len()),
        });
    }

    state.ingredients().delete(id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient"),
        Some(id.into()),
        Some(current.group_id),
        None,
        None,
    )
        .await;
    Ok(())
}

// --------------------------------------------------------------- sources

pub async fn list_sources(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
) -> Result<Vec<IngredientSource>> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::View))?;
    state.ingredients().list_sources(Some(ingredient_id)).await
}

#[derive(Debug, Deserialize)]
pub struct IngredientSourceInput {
    pub store_id: StoreId,
    pub package_size: String,
    pub unit_id: UnitId,
    pub price: String,
    pub url: Option<String>,
    pub comment: Option<String>,
}

pub async fn add_source(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    input: IngredientSourceInput,
) -> Result<IngredientSource> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    let package_size = parse_amount(&input.package_size, "package_size")?;
    let price = parse_money(&input.price, "price")?;

    let source = IngredientSource {
        id: IngredientSourceId::default(),
        ingredient_id,
        store_id: input.store_id,
        package_size,
        unit_id: input.unit_id,
        price,
        url: input.url,
        comment: input.comment,
    };

    let added_source = state.ingredients().add_source(source).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_source"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!(added_source)),
    )
        .await;

    Ok(added_source)
}

pub async fn update_source(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    source_id: IngredientSourceId,
    input: IngredientSourceInput,
) -> Result<IngredientSource> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    let package_size = parse_amount(&input.package_size, "package_size")?;
    let price = parse_money(&input.price, "price")?;

    let source = IngredientSource {
        id: source_id,
        ingredient_id,
        store_id: input.store_id,
        package_size,
        unit_id: input.unit_id,
        price,
        url: input.url,
        comment: input.comment,
    };

    let updated_source = state.ingredients().update_source(source).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_source"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!(updated_source)),
    )
        .await;

    Ok(updated_source)
}

pub async fn delete_source(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    source_id: IngredientSourceId,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    state.ingredients().delete_source(source_id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient_source"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!({ "deleted_source_id": source_id })),
    )
        .await;

    Ok(())
}

// ---------------------------------------------------------------- weights

#[derive(Debug, Deserialize)]
pub struct IngredientWeightInput {
    pub unit_id: UnitId,
    /// Weight in **kg** per unit of `unit_id` (e.g. 1 piece = 0.12 kg).
    pub weight: String,
}

#[derive(Debug, Deserialize)]
pub struct IngredientWeightUpdateInput {
    pub weight: String,
}

pub async fn list_weights(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
) -> Result<Vec<IngredientWeight>> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::View))?;
    state.ingredients().get_weights(ingredient_id).await
}

pub async fn add_weight(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    input: IngredientWeightInput,
) -> Result<IngredientWeight> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    let weight = parse_amount(&input.weight, "weight")?;

    let new_weight = IngredientWeight {
        ingredient_id,
        unit_id: input.unit_id,
        weight,
    };
    let added_weight = state.ingredients().add_weight(new_weight).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_weight"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!(added_weight)),
    )
        .await;

    Ok(added_weight)
}

pub async fn update_weight(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    unit_id: UnitId,
    input: IngredientWeightUpdateInput,
) -> Result<IngredientWeight> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    let weight = parse_amount(&input.weight, "weight")?;

    let updated_weight = IngredientWeight {
        ingredient_id,
        unit_id,
        weight,
    };
    let result = state.ingredients().update_weight(updated_weight).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_weight"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!(result)),
    )
        .await;

    Ok(result)
}

pub async fn delete_weight(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    unit_id: UnitId,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    state.ingredients().delete_weight(ingredient_id, unit_id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient_weight"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!({ "deleted_unit_id": unit_id })),
    )
        .await;

    Ok(())
}

// ------------------------------------------------------------- properties

#[derive(Debug, Deserialize)]
pub struct IngredientPropertyInput {
    pub property_id: PropertyId,
}

pub async fn list_ingredient_properties(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
) -> Result<IngredientProperties> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::View))?;
    state.properties().get_ingredient_properties(ingredient_id).await
}

pub async fn add_property(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    input: IngredientPropertyInput,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    state.properties().assign_to_ingredient(ingredient_id, input.property_id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_property"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!({ "added_property_id": input.property_id })),
    )
        .await;

    Ok(())
}

pub async fn delete_property(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    property_id: PropertyId,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    actor.ctx.assert_group_perm(ingredient.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    state.properties().remove_from_ingredient(ingredient_id, property_id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient_property"),
        Some(ingredient_id.into()),
        Some(ingredient.group_id),
        None,
        Some(json!({ "deleted_property_id": property_id })),
    )
        .await;

    Ok(())
}

// --------------------------------------------------------------- ownership

pub async fn transfer_owner(
    state: &MyAppState,
    actor: &AppCtx,
    id: IngredientId,
    target_group_id: GroupId,
) -> Result<()> {
    let current = state.ingredients().get(id).await?;
    assert_ownership_transfer(
        &actor.ctx,
        "ingredient",
        current.group_id,
        target_group_id,
        Permission::Ingredient(IngredientPerm::TransferOwner),
    )?;
    state.ingredients().set_group(id, target_group_id).await?;
    audit::record(
        state,
        actor,
        audit::action::OWNER_TRANSFERRED,
        Some("ingredient"),
        Some(id.into()),
        Some(target_group_id),
        None,
        Some(json!({ "from_group_id": current.group_id })),
    )
        .await;
    Ok(())
}

// --------------------------------------------------------------- helpers

fn parse_amount(raw: &str, field: &str) -> Result<Amount> {
    Amount::from_str(raw).map_err(|_| foodlib_new::Error::InvalidInput {
        message: format!("invalid {field} value: {raw}"),
    })
}

fn parse_money(raw: &str, field: &str) -> Result<Money> {
    Money::from_str(raw).map_err(|_| foodlib_new::Error::InvalidInput {
        message: format!("invalid {field} value: {raw}"),
    })
}