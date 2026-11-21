//! Ingredient use-cases: filtered list/search, usages and ownership transfer.

use super::audit;
use super::{assert_ownership_transfer, AppCtx, MyAppState, Result};
use bigdecimal::BigDecimal;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::ingredient::IngredientWithSource;
use foodlib_new::entities::recipe::Recipe;
use foodlib_new::ingredient::{Ingredient, IngredientSource, IngredientWeight};
use foodlib_new::policy::{GlobalPermission, IngredientPerm, Permission};
use foodlib_new::property::{IngredientProperties, Property};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct IngredientListQuery {
    /// Case-insensitive substring search on the name.
    pub search: Option<String>,
    /// Only ingredients carrying this property (e.g. a spice tag).
    pub property_id: Option<i32>,
}

/// `GET /api/ingredients` — ingredients visible to the caller (their groups),
/// with sources, optionally filtered by text/property.
pub async fn list_ingredients(
    state: &MyAppState,
    ctx: &AppCtx,
    query: IngredientListQuery,
) -> Result<Vec<IngredientWithSource>> {
    let ingredients = state.ingredients().list_with_sources().await?;

    let search = query.search.unwrap_or_default().to_lowercase();

    // Property filter: resolve the matching ids once, then intersect —
    // keeps the list query untouched and the properties table is tiny.
    let property_ids = match query.property_id {
        Some(property_id) => Some(state.ingredients().ingredient_ids_for_property(property_id).await?),
        None => None,
    };

    Ok(ingredients
        .into_iter()
        // Admins halten sich an Mitgliedschaften (kein Inline-Bypass).
        .filter(|ingredient| ctx.ctx.group_ids.contains(&ingredient.group_id))
        .filter(|ingredient| search.is_empty() || ingredient.name.to_lowercase().contains(&search))
        .filter(|ingredient| {
            property_ids
                .as_ref()
                .map_or(true, |ids| ids.contains(&ingredient.id))
        })
        .collect())
}

/// Rezepte, die diese Zutat verwenden — nur Ressourcen sichtbarer Gruppen
/// (kein Inline-Bypass). Grundlage für „Verwendungen" in der Zutaten-Ansicht
/// und die Lösch-Warnung.
pub async fn usages(state: &MyAppState, ctx: &AuthCtx, id: i32) -> Result<Vec<Recipe>> {
    let recipes = state.ingredients().usages(id).await?;
    Ok(recipes
        .into_iter()
        .filter(|recipe| ctx.group_ids.contains(&recipe.group_id))
        .collect())
}


#[derive(Debug, Deserialize)]
pub struct IngredientCreateInput {
    pub name: String,
    /// Energy density in kJ/g.
    pub energy: String,
    pub comment: Option<String>,
    pub group_id: i32,
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
    // Check user is blocked
    actor.ctx.assert_global(GlobalPermission::AllowedToCreateNewDomainObjects)?;
    // Check user has permission to create ingredients in the target group
    actor.ctx.assert_group_perm(payload.group_id, Permission::Ingredient(IngredientPerm::Create))?;

    let energy: BigDecimal = BigDecimal::from_str(&payload.energy)
        .map_err(|_| 
            foodlib_new::Error::InvalidInput {
                message: format!("invalid energy value: {}", payload.energy)
            }
        )?;

    let ingredient = Ingredient {
        id: 0,
        name: payload.name,
        energy,
        comment: payload.comment,
        group_id: payload.group_id,
    };
    let result = state.ingredients().create(ingredient).await?;
    Ok(result)
}

pub async fn get_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
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
    id: i32,
    payload: &IngredientUpdateInput,
) -> Result<Ingredient> {
    let current = state.ingredients().get(id).await?;
    actor.ctx.assert_group_perm(current.group_id, Permission::Ingredient(IngredientPerm::Edit))?;

    let energy = match &payload.energy {
        Some(energy_str) => Some(BigDecimal::from_str(energy_str)
            .map_err(|_| 
                foodlib_new::Error::InvalidInput {
                    message: format!("invalid energy value: {}", energy_str)
                }
            )?),
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
        Some(id),
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
    id: i32,
) -> Result<()> {
    let current = state.ingredients().get(id).await?;
    actor.ctx.assert_group_perm(current.group_id, Permission::Ingredient(IngredientPerm::Delete))?;

    // Check for usages in recipes
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
        Some(id),
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
    id: i32,
) -> Result<Vec<IngredientSource>> {
    actor.ctx.assert_group_perm(id, Permission::Ingredient(IngredientPerm::View))?;

    let sources = state.ingredients().list_sources(Option::from(id)).await?;
    Ok(sources) // filtering by group is not needed here, as sources are tied to ingredients,
}


#[derive(Debug, Deserialize)]
pub struct IngredientSourceInput {
    pub store_id: i32,
    pub package_size: String,
    pub unit_id: i32,
    pub price: String,
    pub url: Option<String>,
    pub comment: Option<String>,
}
pub async fn add_source(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
    input: IngredientSourceInput,
) -> Result<IngredientSource> {
    // Asserts
    actor.ctx.assert_group_perm(id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Validate numeric fields
    let package_size: BigDecimal = BigDecimal::from_str(&input.package_size)
        .map_err(|_|             foodlib_new::Error::InvalidInput {
                message: format!("invalid package_size value: {}", input.package_size)
            }
        )?;
    let price: BigDecimal = BigDecimal::from_str(&input.price)
        .map_err(|_| foodlib_new::Error::InvalidInput {
            message: format!("invalid price value: {}", input.price)
        })?;

    let source = IngredientSource {
        id: 0, // TODO: auto-incremented by the database
        ingredient_id: id,
        store_id: input.store_id,
        package_size,
        unit_id: input.unit_id,
        price,
        url: input.url,
        comment: input.comment,
    };

    // Action
    let added_source = state.ingredients().add_source(source).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_source"),
        Some(id),
        None,
        None,
        Some(json!(added_source)),
    )
        .await;
    // Return
    Ok(added_source)
}

pub async fn update_source(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    source_id: i32,
    input: IngredientSourceInput,
) -> Result<IngredientSource> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Validate numeric fields
    let package_size: BigDecimal = BigDecimal::from_str(&input.package_size)
        .map_err(|_| 
            foodlib_new::Error::InvalidInput {
                message: format!("invalid package_size value: {}", input.package_size)
            }
        )?;
    let price: BigDecimal = BigDecimal::from_str(&input.price)
        .map_err(|_| 
            foodlib_new::Error::InvalidInput {
                message: format!("invalid price value: {}", input.price)
            }
        )?;

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

    // Action
    let updated_source = state.ingredients().update_source(source).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_source"),
        Some(ingredient_id),
        None,
        None,
        Some(json!(updated_source)),
    )
        .await;
    // Return
    Ok(updated_source)
}

pub async fn delete_source(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    source_id: i32,
) -> Result<()> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Action
    state.ingredients().delete_source(source_id).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient_source"),
        Some(ingredient_id),
        None,
        None,
        Some(json!({ "deleted_source_id": source_id })),
    )
        .await;
    // Return
    Ok(())
}

// ---------------------------------------------------------------- weights

/// Parses a decimal amount ("1.5") into a `BigDecimal`.

#[derive(Debug, Deserialize)]
pub struct IngredientWeightInput {
    pub unit_id: i32,
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
    ingredient_id: i32,
) -> Result<Vec<IngredientWeight>> {
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::View))?;
    let weights = state.ingredients().get_weights(ingredient_id).await?;
    Ok(weights)
}


pub async fn add_weight(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    input: IngredientWeightInput,
) -> Result<IngredientWeight> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Parse strings
    let weight: BigDecimal = BigDecimal::from_str(&input.weight)
        .map_err(|_| 
            foodlib_new::Error::InvalidInput {
                message: format!("invalid weight value: {}", input.weight)
            }
        )?;

    // Action
    let new_weight = IngredientWeight {
        ingredient_id,
        unit_id: input.unit_id,
        weight,
    };
    let added_weight = state.ingredients().add_weight(new_weight).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_weight"),
        Some(ingredient_id),
        None,
        None,
        Some(json!(added_weight)),
    )
        .await;
    // Return
    Ok(added_weight)
}

pub async fn update_weight(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    unit_id: i32,
    input: IngredientWeightUpdateInput,
) -> Result<IngredientWeight> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Parse strings
    let weight: BigDecimal = BigDecimal::from_str(&input.weight)
        .map_err(|_| 
            foodlib_new::Error::InvalidInput {
                message: format!("invalid weight value: {}", input.weight)
            }
        )?;

    // Action
    let updated_weight = IngredientWeight {
        ingredient_id,
        unit_id,
        weight,
    };
    let result = state.ingredients().update_weight(updated_weight).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_weight"),
        Some(ingredient_id),
        None,
        None,
        Some(json!(result)),
    )
        .await;
    // Return
    Ok(result)
}

pub async fn delete_weight(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    unit_id: i32,
) -> Result<()> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Action
    state.ingredients().delete_weight(ingredient_id, unit_id).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient_weight"),
        Some(ingredient_id),
        None,
        None,
        Some(json!({ "deleted_unit_id": unit_id })),
    )
        .await;
    // Return
    Ok(())
}

// ------------------------------------------------------------- properties

#[derive(Debug, Deserialize)]
pub struct IngredientPropertyInput {
    pub property_id: i32,
}

pub async fn list_ingredient_properties(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
) -> Result<IngredientProperties> {
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::View))?;
    let properties = state.properties().get_ingredient_properties(ingredient_id).await?;
    Ok(properties)
}

pub async fn add_property(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    input: IngredientPropertyInput,
) -> Result<()> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Action
    state.properties().assign_to_ingredient(ingredient_id, input.property_id).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("ingredient_property"),
        Some(ingredient_id),
        None,
        None,
        Some(json!({ "added_property_id": input.property_id })),
    )
        .await;
    // Return
    Ok(())
}

pub async fn delete_property(
    state: &MyAppState,
    actor: &AppCtx,
    ingredient_id: i32,
    property_id: i32,
) -> Result<()> {
    // Asserts
    actor.ctx.assert_group_perm(ingredient_id, Permission::Ingredient(IngredientPerm::Edit))?;

    // Action
    state.properties().remove_from_ingredient(ingredient_id, property_id).await?;
    // Logging
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("ingredient_property"),
        Some(ingredient_id),
        None,
        None,
        Some(json!({ "deleted_property_id": property_id })),
    )
        .await;
    // Return
    Ok(())
}


// --------------------------------------------------------------- ownership

pub async fn transfer_owner(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
    target_group_id: i32,
) -> Result<()> {
    let ctx = &actor.ctx;
    let current = state.ingredients().get(id).await?;
    assert_ownership_transfer(
        ctx,
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
        Some(id),
        Some(target_group_id),
        None,
        Some(json!({ "from_group_id": current.group_id })),
    )
        .await;
    Ok(())
}
