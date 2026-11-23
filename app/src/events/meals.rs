//! Meal use-cases.

use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{EventId, MealId, PlaceId, RecipeId};
use time::OffsetDateTime;

use crate::app_ctx::AppCtx;
use crate::{MyAppState, Result};

/// Deletes one meal from an event.
/// Requires `EditEvent` and `EditMeal` permission.
pub async fn delete_meal(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    actor.ctx.assert_can_edit_meal(meal_id).await?;
    state.meals().remove_meal(meal_id).await?;
    Ok(())
}

/// Duplicates one meal within its event.
/// Requires `EditEvent` and `EditMeal` permission.
pub async fn duplicate_meal(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    actor.ctx.assert_can_edit_meal(meal_id).await?;
    state.meals().duplicate_meal(meal_id).await?;
    Ok(())
}

/// Adds or updates one meal. An invalid `meal_id` is the form's "new meal"
/// sentinel and creates a meal; a valid one updates it. The meal's own
/// `EditMeal` permission is asserted for existing meals (the sentinel id `-1`
/// is tolerated for the inline add-row).
/// Requires `EditEvent` permission.
pub async fn save_meal(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
    recipe_id: RecipeId,
    place_id: PlaceId,
    start_time: OffsetDateTime,
    end_time: OffsetDateTime,
    energy: Amount,
    servings: i32,
    comment: Option<String>,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    if meal_id != MealId(-1) {
        actor.ctx.assert_can_edit_meal(meal_id).await?;
    }

    if meal_id.is_valid() {
        state
            .meals()
            .update_meal(
                meal_id,
                recipe_id,
                place_id,
                start_time,
                end_time,
                energy,
                servings,
                comment,
            )
            .await?;
    } else {
        state
            .meals()
            .add_meal(
                event_id,
                recipe_id,
                place_id,
                start_time,
                end_time,
                energy,
                servings,
                comment,
            )
            .await?;
    }
    Ok(())
}
