use crate::{AppCtx, MyAppState};
use crate::Result;
use foodlib_new::Error;
use foodlib_new::event::{CreateFoodPrep, FoodPrep};
use foodlib_new::types::ids::{EventId, FoodPrepId, IngredientId, IngredientSourceId, StoreId};

/// Removes one ingredient source default override from an event.
/// Requires `EditEvent` permission.
pub async fn delete_source_override(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    source_id: IngredientSourceId,
) -> foodlib_new::Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    state
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
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    ingredient_name: &str,
    store_id: StoreId,
) -> foodlib_new::Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let ingredient = state
        .ingredients()
        .get_by_name(ingredient_name)
        .await?;
    let sources = state.ingredients().get_sources(ingredient.id).await?;
    let source = sources
        .iter()
        .find(|s| s.store_id == store_id)
        .ok_or(Error::Misc("This Source is not defined for this ingredient! \n Add this source to the ingredient in the ingredients tab.".to_string()))?;
    state
        .events()
        .set_source_default_override(event_id, source.id)
        .await?;
    Ok(())
}

/// Adds one food-prep task to an event.
/// Requires `EditEvent` permission.
pub async fn add_food_prep(
    state: &MyAppState,
    actor: &AppCtx,
    prep: CreateFoodPrep,
) -> foodlib_new::Result<()> {
    actor.ctx.assert_can_edit_event(prep.event_id).await?;
    state.events().add_food_prep(prep).await?;
    Ok(())
}

/// Updates one food-prep task. The task's own `EditFoodPrep` permission is
/// asserted in addition to the event's.
pub async fn update_food_prep(
    state: &MyAppState,
    actor: &AppCtx,
    prep: FoodPrep,
) -> foodlib_new::Result<()> {
    actor.ctx.assert_can_edit_event(prep.event_id).await?;
    actor.ctx.assert_can_edit_food_prep(prep.id).await?;
    state.events().update_food_prep(prep).await?;
    Ok(())
}

/// Deletes one food-prep task.
/// Requires `EditEvent` and `EditFoodPrep` permission.
pub async fn delete_food_prep(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    prep_id: FoodPrepId,
) -> foodlib_new::Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    actor.ctx.assert_can_edit_food_prep(prep_id).await?;
    state.events().delete_food_prep(prep_id).await?;
    Ok(())
}

/// Ingredients of an event not yet covered by any shopping tour, ordered as
/// the ops layer returns them, with water filtered out (water is never
/// shopped).
/// Requires `EditEvent` permission.
pub async fn gather_uncovered_ingredients(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> foodlib_new::Result<Vec<foodlib_new::event::IngredientWithoutTour>> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let mut uncovered = state.events().get_ingredients_without_tour(event_id).await?;
    // Remove water from ingredient list // TODO: Bad smell
    uncovered.retain(|i| i.ingredient_id != IngredientId(4));
    Ok(uncovered)
}

/// Fetches one food-prep task.
pub async fn get_food_prep(
    state: &MyAppState,
    prep_id: FoodPrepId,
) -> Result<FoodPrep> {
    state.events().get_food_prep(prep_id).await
}

/// Fetches one food-prep task in the context of an event (dialogs, edit
/// forms). Requires `EditEvent` permission, plus `EditFoodPrep` for an
/// existing task (an invalid `prep_id` is the "new task" sentinel).
pub async fn get_food_prep_with_permission(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    prep_id: FoodPrepId,
) -> Result<FoodPrep> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    if prep_id.is_valid() {
        actor.ctx.assert_can_edit_food_prep(prep_id).await?;
    }
    state.events().get_food_prep(prep_id).await
}