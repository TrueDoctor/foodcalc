//! Ingredient use-cases.

use crate::ctx::assert_can_move_to;
use crate::entities::allergen::{canonical, ClassifiedProperty, PropertySource};
use crate::entities::ingredient::{
    CreateIngredient, CreateIngredientSource, Ingredient, IngredientSource, IngredientWithSource,
};
use crate::entities::property::IngredientProperties;
use crate::entities::recipe::Recipe;
use crate::entities::store::Store;
use crate::types::amount::Amount;
use crate::types::ids::{GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId, UnitId};
use crate::types::money::Money;
use crate::{AppCtx, AppState, Error, Result};

/// All ingredients together with a "has sources" flag.
#[allow(deprecated)]
pub async fn list_with_sources(state: &AppState) -> Result<Vec<IngredientWithSource>> {
    state.db().ingredients().list_with_sources().await
}

/// Creates a new ingredient owned by the actor's personal group.
pub async fn create_ingredient(
    state: &AppState,
    actor: &AppCtx,
    name: String,
    energy: Amount,
    comment: Option<String>,
) -> Result<()> {
    let group = state.db().users().get_personal_group(actor.user.id).await?;
    state
        .db()
        .ingredients()
        .create(CreateIngredient {
            name,
            energy,
            comment,
            group_id: group.id,
        })
        .await?;
    Ok(())
}

/// Updates an existing ingredient and optionally transfers it to another
/// group. Requires edit rights on the ingredient's owner group; moving to
/// another group additionally requires membership in (or admin over) the
/// target group. Returns the refreshed ingredient.
pub async fn update_ingredient(
    state: &AppState,
    actor: &AppCtx,
    mut ingredient: Ingredient,
    target_group: GroupId,
) -> Result<Ingredient> {
    let existing = state.db().ingredients().get(ingredient.id).await?;
    actor.assert_group(existing.group_id)?;

    ingredient.group_id = existing.group_id;
    if existing.group_id != target_group {
        assert_can_move_to(actor, target_group)?;
    }
    let updated = state.db().ingredients().update(ingredient).await?;
    if existing.group_id != target_group {
        state
            .db()
            .ingredients()
            .set_group(updated.id, target_group)
            .await?;
    }
    state.db().ingredients().get(updated.id).await
}

/// Deletes an ingredient. Only human administrators may delete ingredients.
pub async fn delete_ingredient(
    state: &AppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    state.db().ingredients().delete(ingredient_id).await
}

/// Lists the recipes that use an ingredient (shown before confirming it is
/// deleted). Only administrators may delete ingredients.
pub async fn gather_usages(
    state: &AppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
) -> Result<Vec<Recipe>> {
    actor.require_admin()?;
    state.db().ingredients().usages(ingredient_id).await
}

/// Everything the sources dialog renders, fetched in one use case so
/// transports only have to deal with presentation.
pub struct SourcesViewData {
    pub sources: Vec<IngredientSource>,
    pub stores: Vec<Store>,
    pub ingredient: Ingredient,
    pub user_group_ids: Vec<GroupId>,
}

/// Collects all data needed to render an ingredient's sources dialog.
pub async fn gather_sources(
    state: &AppState,
    actor: Option<&AppCtx>,
    ingredient_id: IngredientId,
) -> Result<SourcesViewData> {
    let sources = state.db().ingredients().get_sources(ingredient_id).await?;
    let stores = state.db().stores().list().await?;
    let ingredient = state.db().ingredients().get(ingredient_id).await?;
    let user_group_ids = actor.map(|a| a.group_ids.clone()).unwrap_or_default();

    Ok(SourcesViewData {
        sources,
        stores,
        ingredient,
        user_group_ids,
    })
}

/// Creates or updates one ingredient source; an invalid `source_id` means
/// create (the sources dialog submits a sentinel id for the inline add row).
/// Requires edit rights on the ingredient's owner group.
#[allow(clippy::too_many_arguments)]
pub async fn save_source(
    state: &AppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    source_id: IngredientSourceId,
    store_id: StoreId,
    package_size: Amount,
    price: Money,
    url: Option<String>,
    comment: Option<String>,
) -> Result<()> {
    actor.assert_can_edit_ingredient(ingredient_id).await?;

    if !source_id.is_valid() {
        state
            .db()
            .ingredients()
            .add_source(CreateIngredientSource {
                ingredient_id,
                store_id,
                package_size,
                price,
                unit_id: UnitId(0),
                url,
                comment,
            })
            .await?;
    } else {
        let sources = state.db().ingredients().get_sources(ingredient_id).await?;
        if !sources.iter().any(|s| s.id == source_id) {
            return Err(Error::NotFound {
                entity: "IngredientSource",
                id: source_id.to_string(),
            });
        }
        state
            .db()
            .ingredients()
            .update_source(IngredientSource {
                id: source_id,
                ingredient_id,
                store_id,
                package_size,
                price,
                unit_id: UnitId(0),
                url,
                comment,
            })
            .await?;
    }
    Ok(())
}

/// Deletes one ingredient source after verifying it belongs to `ingredient_id`.
/// Requires edit rights on the ingredient's owner group.
pub async fn delete_source(
    state: &AppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    source_id: IngredientSourceId,
) -> Result<()> {
    actor.assert_can_edit_ingredient(ingredient_id).await?;
    let sources = state.db().ingredients().get_sources(ingredient_id).await?;
    if !sources.iter().any(|s| s.id == source_id) {
        return Err(Error::NotFound {
            entity: "IngredientSource",
            id: source_id.to_string(),
        });
    }
    state.db().ingredients().delete_source(source_id).await
}

/// Everything the allergen (properties) dialog renders, fetched in one use
/// case so transports only have to deal with presentation.
pub struct PropertiesViewData {
    pub properties: IngredientProperties,
    pub ingredient: Ingredient,
    pub user_group_ids: Vec<GroupId>,
}

/// Collects all data needed to render an ingredient's allergen dialog.
pub async fn gather_properties(
    state: &AppState,
    actor: Option<&AppCtx>,
    ingredient_id: IngredientId,
) -> Result<PropertiesViewData> {
    let properties = state
        .db()
        .properties()
        .get_ingredient_properties(ingredient_id)
        .await?;
    let ingredient = state.db().ingredients().get(ingredient_id).await?;
    let user_group_ids = actor.map(|a| a.group_ids.clone()).unwrap_or_default();

    Ok(PropertiesViewData {
        properties,
        ingredient,
        user_group_ids,
    })
}

/// Removes one property (e.g. allergen tag) from an ingredient.
/// Requires edit rights on the ingredient's owner group.
pub async fn delete_property(
    state: &AppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    property_id: PropertyId,
) -> Result<()> {
    actor.assert_can_edit_ingredient(ingredient_id).await?;
    state
        .db()
        .properties()
        .remove_from_ingredient(ingredient_id, property_id)
        .await
}

/// Adds one canonical allergen tag to an ingredient. `kind` is either
/// `"contains"` or `"may_contain"`.
/// Requires edit rights on the ingredient's owner group.
pub async fn add_property(
    state: &AppState,
    actor: &AppCtx,
    ingredient_id: IngredientId,
    canonical: String,
    kind: String,
) -> Result<()> {
    actor.assert_can_edit_ingredient(ingredient_id).await?;

    if !canonical::ALL.contains(&canonical.as_str()) {
        return Err(Error::Validation {
            message: format!("Unknown allergen: {}", canonical),
        });
    }
    let source = match kind.as_str() {
        "contains" => PropertySource::Contains,
        "may_contain" => PropertySource::MayContain,
        _ => {
            return Err(Error::Validation {
                message: format!("Invalid kind: {}", kind),
            })
        }
    };

    state
        .db()
        .allergens()
        .apply_to_ingredient(
            ingredient_id,
            &[ClassifiedProperty {
                canonical_name: canonical,
                source,
            }],
        )
        .await
}