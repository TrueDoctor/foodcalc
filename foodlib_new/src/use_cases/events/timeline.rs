use crate::types::ids::{EventId, FoodPrepId, MealId, ShoppingTourId};
use crate::{AppCtx, AppState, Error};
use crate::entities::event::{FoodPrep, ShoppingTour};
use crate::entities::meal::Meal;
use crate::entities::recipe::Recipe;

/// Apply a drag/resize update from the timeline to a meal, shopping tour or
/// food-prep task. Read-modify-write: the current row is fetched and only its
/// time fields are patched, so no other data is trusted from the client.
/// Requires `EditEvent` permission.
pub async fn move_timeline_item(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    kind: &str,
    item_id: i32,
    new_start: time::OffsetDateTime,
    new_end: Option<time::OffsetDateTime>,
) -> crate::Result<()> {
    actor.assert_can_edit_event(event_id).await?;

    match kind {
        "meal" => {
            let m = state.db().meals().get_meal(MealId(item_id)).await?;
            // The timeline only shows items of `event_id`; reject a mismatched
            // id so one event's editor cannot move another event's row.
            if m.event_id != event_id {
                return Err(Error::Validation {
                    message: "meal does not belong to this event".into(),
                });
            }
            // Preserve the meal's duration on a plain move; honor new_end on a
            // resize. Guard against a non-positive span by falling back to the
            // original duration.
            let new_end = new_end
                .filter(|e| *e > new_start)
                .unwrap_or(new_start + (m.end_time - m.start_time));
            state
                .db()
                .meals()
                .update_meal(
                    m.meal_id,
                    m.recipe_id,
                    m.place_id,
                    new_start,
                    new_end,
                    m.energy,
                    m.servings,
                    m.comment,
                )
                .await?;
        }
        "tour" => {
            let mut tour = state
                .db()
                .events()
                .get_shopping_tour(ShoppingTourId(item_id))
                .await?;
            if tour.event_id != event_id {
                return Err(Error::Validation {
                    message: "shopping tour does not belong to this event".into(),
                });
            }
            tour.tour_date = new_start;
            state.db().events().update_shopping_tour(tour).await?;
        }
        "prep" => {
            let mut prep = state.db().events().get_food_prep(FoodPrepId(item_id)).await?;
            if prep.event_id != event_id {
                return Err(Error::Validation {
                    message: "food prep task does not belong to this event".into(),
                });
            }
            // Keep the use window's length relative to the (possibly changed)
            // prep time so a moved prep still "uses until" the same offset.
            let delta = new_start - prep.prep_date;
            prep.prep_date = new_start;
            prep.use_until += delta;
            if let Some(from) = prep.use_from {
                prep.use_from = Some(from + delta);
            }
            state.db().events().update_food_prep(prep).await?;
        }
        other => {
            return Err(Error::Misc(format!("unknown kind: {other}")));
        }
    }
    Ok(())
}

/// Shifts every dated item (meals, shopping tours, food prep) of an event so
/// that the first meal starts at `target`.
/// Requires `EditEvent` permission.
pub async fn shift_event_times(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    target: time::OffsetDateTime,
) -> crate::Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    state.db().events().shift_event_times(event_id, target).await?;
    Ok(())
}


/// Everything needed to render an event's timeline.
pub struct EventItemSources {
    pub meals: Vec<Meal>,
    pub tours: Vec<ShoppingTour>,
    pub preps: Vec<FoodPrep>,
    pub recipes: Vec<Recipe>,
}

/// Loads meals, shopping tours, food-prep tasks, and recipe names for an event.
pub async fn gather_event_item_sources(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> crate::Result<EventItemSources> {
    actor.assert_can_edit_event(event_id).await?;
    let meals = state.db().meals().get_event_meals(event_id).await?;
    let tours = state.db().events().get_shopping_tours(event_id).await?;
    let preps = state.db().events().get_food_prep_tasks(event_id).await?;
    let recipes = state.db().recipes().list().await?;
    Ok(EventItemSources {
        meals,
        tours,
        preps,
        recipes,
    })
}