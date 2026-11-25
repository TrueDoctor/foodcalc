use crate::ctx::AppCtx;
use crate::entities::event::{CreateFoodPrep, FoodPrep, IngredientWithoutTour};
use crate::types::ids::{EventId, FoodPrepId, IngredientId, IngredientSourceId, StoreId};
use crate::{AppState, Error, Result};

/// Removes one ingredient source default override from an event.
/// Requires `EditEvent` permission.
pub async fn delete_source_override(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    source_id: IngredientSourceId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    state
        .db()
        .events()
        .delete_source_override(event_id, source_id)
        .await?;
    Ok(())
}

/// Sets the default ingredient source override for an event: the ingredient
/// is resolved by name (the form submits a datalist name), then the override
/// points at that ingredient's source for the given store.
/// Requires `EditEvent` permission.
pub async fn set_source_default_override(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    ingredient_name: &str,
    store_id: StoreId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    let ingredient = state
        .db()
        .ingredients()
        .get_by_name(ingredient_name)
        .await?;
    let sources = state.db().ingredients().get_sources(ingredient.id).await?;
    let source = sources
        .iter()
        .find(|s| s.store_id == store_id)
        .ok_or_else(|| {
            Error::Misc(
                "This Source is not defined for this ingredient! \n Add this source to the ingredient in the ingredients tab."
                    .to_string(),
            )
        })?;
    state
        .db()
        .events()
        .set_source_default_override(event_id, source.id)
        .await?;
    Ok(())
}

/// Adds one food-prep task to an event.
/// Requires `EditEvent` permission.
pub async fn add_food_prep(
    state: &AppState,
    actor: &AppCtx,
    prep: CreateFoodPrep,
) -> Result<()> {
    actor.assert_can_edit_event(prep.event_id).await?;
    state.db().events().add_food_prep(prep).await?;
    Ok(())
}

/// Updates one food-prep task. Ensures the task belongs to `prep.event_id`
/// and checks `EditEvent` permission.
pub async fn update_food_prep(
    state: &AppState,
    actor: &AppCtx,
    prep: FoodPrep,
) -> Result<()> {
    actor.assert_can_edit_event(prep.event_id).await?;
    let existing = state.db().events().get_food_prep(prep.id).await?;
    if existing.event_id != prep.event_id {
        return Err(Error::NotFound {
            entity: "FoodPrep",
            id: prep.id.to_string(),
        });
    }
    state.db().events().update_food_prep(prep).await?;
    Ok(())
}

/// Deletes one food-prep task after verifying it belongs to `event_id`.
/// Requires `EditEvent` permission.
pub async fn delete_food_prep(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    prep_id: FoodPrepId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    let existing = state.db().events().get_food_prep(prep_id).await?;
    if existing.event_id != event_id {
        return Err(Error::NotFound {
            entity: "FoodPrep",
            id: prep_id.to_string(),
        });
    }
    state.db().events().delete_food_prep(prep_id).await?;
    Ok(())
}

/// Ingredients of an event not yet covered by any shopping tour, ordered as
/// the ops layer returns them, with water filtered out (water is never
/// shopped).
/// Requires `EditEvent` permission.
pub async fn gather_uncovered_ingredients(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<Vec<IngredientWithoutTour>> {
    actor.assert_can_edit_event(event_id).await?;
    let mut uncovered = state
        .db()
        .events()
        .get_ingredients_without_tour(event_id)
        .await?;
    // Remove water from ingredient list // TODO: Bad smell
    uncovered.retain(|i| i.ingredient_id != IngredientId(4));
    Ok(uncovered)
}

/// Fetches one food-prep task. Requires `EditEvent` permission on its parent event.
pub async fn get_food_prep(
    state: &AppState,
    actor: &AppCtx,
    prep_id: FoodPrepId,
) -> Result<FoodPrep> {
    actor.assert_can_edit_food_prep(prep_id).await?;
    state.db().events().get_food_prep(prep_id).await
}

/// Fetches one food-prep task in the context of an event (dialogs, edit
/// forms). Requires `EditEvent` permission, and verifies the task belongs
/// to `event_id` for existing tasks.
pub async fn get_food_prep_with_permission(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    prep_id: FoodPrepId,
) -> Result<FoodPrep> {
    actor.assert_can_edit_event(event_id).await?;
    let prep = state.db().events().get_food_prep(prep_id).await?;
    if prep_id.is_valid() && prep.event_id != event_id {
        return Err(Error::NotFound {
            entity: "FoodPrep",
            id: prep_id.to_string(),
        });
    }
    Ok(prep)
}