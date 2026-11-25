//! Meal use-cases.

use time::OffsetDateTime;

use crate::entities::{event, meal, recipe};
use crate::types::amount::Amount;
use crate::types::ids::{EventId, MealId, PlaceId, RecipeId};
use crate::{AppCtx, AppState, Error, Result};
use crate::entities::meal::Meal;

/// Fetches one meal.
pub async fn get_meal(state: &AppState, meal_id: MealId) -> Result<meal::Meal> {
    state.db().meals().get_meal(meal_id).await
}

/// Fetches one meal. Requires `EditMeal` permission.
pub async fn get_meal_for_actor(
    state: &AppState,
    actor: &AppCtx,
    meal_id: MealId,
) -> Result<meal::Meal> {
    actor.assert_can_edit_meal(meal_id).await?;
    state.db().meals().get_meal(meal_id).await
}

/// Fetches one meal's total price. Requires `EditMeal` permission.
pub async fn get_meal_price_for_actor(
    state: &AppState,
    actor: &AppCtx,
    meal_id: MealId,
) -> Result<Amount> {
    actor.assert_can_edit_meal(meal_id).await?;
    state.db().meals().get_meal_price(meal_id).await
}

/// Fetches one meal in the context of an event (e.g. a delete-confirmation
/// dialog). Requires `EditEvent` and `EditMeal` permission.
pub async fn get_meal_for_event(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<meal::Meal> {
    actor.assert_can_edit_event(event_id).await?;
    actor.assert_can_edit_meal(meal_id).await?;
    let meal = state.db().meals().get_meal(meal_id).await?;
    if meal.event_id != event_id {
        return Err(Error::Validation {
            message: "meal does not belong to this event".into(),
        });
    }
    Ok(meal)
}

/// Guards meal mutations against cross-event ids: the meal's `event_id` must
/// match the event whose editor the request came from.
async fn assert_meal_in_event(state: &AppState, event_id: EventId, meal_id: MealId) -> Result<()> {
    let meal = state.db().meals().get_meal(meal_id).await?;
    if meal.event_id != event_id {
        return Err(Error::Validation {
            message: "meal does not belong to this event".into(),
        });
    }
    Ok(())
}

/// Authorization for rendering the meal form: requires `EditEvent`, plus
/// `EditMeal` for an existing meal (the `-1` sentinel is the "new meal" row).
pub async fn assert_meal_form_access(
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    if meal_id != MealId(-1) {
        actor.assert_can_edit_meal(meal_id).await?;
    }
    Ok(())
}

/// Fetches one meal's total price.
pub async fn get_meal_price(state: &AppState, meal_id: MealId) -> Result<Amount> {
    state.db().meals().get_meal_price(meal_id).await
}

/// Fetches one meal's ingredients (scaled rows for the per-serving dialog).
pub async fn get_meal_ingredients(
    state: &AppState,
    meal_id: MealId,
) -> Result<Vec<meal::MealIngredient>> {
    state.db().meals().get_meal_ingredients(meal_id).await
}

/// Everything the meal form renders: the meal being edited (`None` for a new
/// one), all recipes and all places. Sorting stays presentation.
pub struct MealFormData {
    pub meal: Option<meal::Meal>,
    pub recipes: Vec<recipe::Recipe>,
    pub places: Vec<event::Place>,
}

pub async fn gather_meal_form(state: &AppState, meal_id: MealId) -> Result<MealFormData> {
    let meal = if meal_id.is_valid() {
        Some(state.db().meals().get_meal(meal_id).await?)
    } else {
        None
    };
    let recipes = state.db().recipes().list().await?;
    let places = state.db().events().get_places().await?;
    Ok(MealFormData {
        meal,
        recipes,
        places,
    })
}

/// Deletes one meal from an event.
/// Requires `EditEvent` and `EditMeal` permission.
pub async fn delete_meal(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    actor.assert_can_edit_meal(meal_id).await?;
    assert_meal_in_event(state, event_id, meal_id).await?;
    state.db().meals().remove_meal(meal_id).await?;
    Ok(())
}

/// Duplicates one meal within its event.
/// Requires `EditEvent` and `EditMeal` permission.
pub async fn duplicate_meal(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    actor.assert_can_edit_meal(meal_id).await?;
    assert_meal_in_event(state, event_id, meal_id).await?;
    state.db().meals().duplicate_meal(meal_id).await?;
    Ok(())
}

/// Adds or updates one meal. An invalid `meal_id` is the form's "new meal"
/// sentinel and creates a meal; a valid one updates it. The meal's own
/// `EditMeal` permission is asserted for existing meals (the sentinel id `-1`
/// is tolerated for the inline add-row).
/// Requires `EditEvent` permission.
pub async fn save_meal(
    state: &AppState,
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
    actor.assert_can_edit_event(event_id).await?;
    if meal_id != MealId(-1) {
        actor.assert_can_edit_meal(meal_id).await?;
        assert_meal_in_event(state, event_id, meal_id).await?;
    }

    if meal_id.is_valid() {
        state
            .db()
            .meals()
            .update_meal(
                meal_id, recipe_id, place_id, start_time, end_time, energy, servings, comment,
            )
            .await?;
    } else {
        state
            .db()
            .meals()
            .add_meal(
                event_id, recipe_id, place_id, start_time, end_time, energy, servings, comment,
            )
            .await?;
    }
    Ok(())
}

/// Lists all meals belonging to an event.
pub async fn get_event_meals(state: &AppState, event_id: EventId) -> Result<Vec<Meal>> {
    state.db().meals().get_event_meals(event_id).await
}