//! Event use-cases.

use crate::ctx::assert_can_move_to;
use crate::entities::event::{CreateEvent, Event, FoodPrep, ShoppingTour};
use crate::entities::ingredient::Ingredient;
use crate::entities::meal::Meal;
use crate::entities::recipe::Recipe;
use crate::entities::store::Store;
use crate::types::ids::{EventId, GroupId};
use crate::types::money::Money;
use crate::{entities, AppCtx, AppState, Result};
use bigdecimal::FromPrimitive;

pub mod logistics;
pub mod meals;
pub mod shopping;
pub mod timeline;

/// Creates a new event owned by the actor's personal group. An empty comment
/// is normalized to `None`.
pub async fn create_event(
    state: &AppState,
    actor: &AppCtx,
    name: String,
    comment: Option<String>,
    budget: Option<f64>,
) -> Result<()> {
    let group = state.db().users().get_personal_group(actor.user.id).await?;
    state
        .db()
        .events()
        .create(CreateEvent {
            name,
            comment: comment.filter(|s| !s.is_empty()),
            budget: budget
                .and_then(bigdecimal::BigDecimal::from_f64)
                .map(Money::from),
            group_id: group.id,
        })
        .await?;
    Ok(())
}

/// Duplicates an event into the actor's personal group and returns the new
/// event's id.
/// Requires `EditEvent` permission on the source event.
pub async fn duplicate_event(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<EventId> {
    actor.assert_can_edit_event(event_id).await?;
    let group = state.db().users().get_personal_group(actor.user.id).await?;
    state.db().events().duplicate(event_id, group.id).await
}

/// Deletes an event.
/// Requires `EditEvent` permission.
pub async fn delete_event(state: &AppState, actor: &AppCtx, event_id: EventId) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    state.db().events().delete(event_id).await?;
    Ok(())
}

/// Updates an event's metadata and optionally transfers it to another group.
/// An empty comment is normalized to `None`.
/// Requires `EditEvent` permission; moving to another group additionally
/// requires membership in (or admin over) the target group.
pub async fn update_event(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    name: String,
    comment: String,
    budget: Option<f64>,
    target_group: GroupId,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    let current = state.db().events().get(event_id).await?;
    if current.group_id != target_group {
        assert_can_move_to(actor, target_group)?;
    }

    state
        .db()
        .events()
        .update(Event {
            id: event_id,
            name,
            comment: (!comment.is_empty()).then_some(comment),
            budget: budget
                .and_then(bigdecimal::BigDecimal::from_f64)
                .map(Money::from),
            group_id: GroupId(0), // Placeholder; ops-level update ignores group_id
        })
        .await?;

    if current.group_id != target_group {
        state.db().events().set_group(event_id, target_group).await?;
    }
    Ok(())
}

/// Everything the event edit form renders, fetched in one use case so
/// transports only have to deal with presentation.
pub struct EventFormData {
    pub event: Event,
    pub stores: Vec<Store>,
    pub overrides: Vec<entities::event::SourceOverrideView>,
    pub ingredients: Vec<Ingredient>,
    pub meals: Vec<Meal>,
    pub recipes: Vec<Recipe>,
    pub preps: Vec<FoodPrep>,
}

/// Collects all data needed to render the event edit form.
/// Requires `EditEvent` permission.
pub async fn gather_form_data(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<EventFormData> {
    actor.assert_can_edit_event(event_id).await?;
    let stores = state.db().stores().list().await?;
    let overrides = state.db().events().get_source_overrides(event_id).await?;
    let ingredients = state.db().ingredients().list().await?;
    let meals = state.db().meals().get_event_meals(event_id).await?;
    let recipes = state.db().recipes().list().await?;
    let preps = state.db().events().get_food_prep_tasks(event_id).await?;
    let event = state.db().events().get(event_id).await?;

    Ok(EventFormData {
        event,
        stores,
        overrides,
        ingredients,
        meals,
        recipes,
        preps,
    })
}

/// Fetches one event (e.g. for delete-confirmation dialogs or overview
/// pages). Requires `EditEvent` permission.
pub async fn get_event(state: &AppState, actor: &AppCtx, event_id: EventId) -> Result<Event> {
    actor.assert_can_edit_event(event_id).await?;
    state.db().events().get(event_id).await
}

/// Lists all events.
pub async fn list_events(state: &AppState) -> Result<Vec<Event>> {
    state.db().events().list().await
}

/// Lists all places (meal locations).
pub async fn list_places(state: &AppState) -> Result<Vec<entities::event::Place>> {
    state.db().events().get_places().await
}

/// Lists an event's ingredient source default overrides.
/// Requires `EditEvent` permission.
pub async fn list_source_overrides(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<Vec<entities::event::SourceOverrideView>> {
    actor.assert_can_edit_event(event_id).await?;
    state.db().events().get_source_overrides(event_id).await
}

/// Lists an event's shopping tours.
pub async fn get_shopping_tours(state: &AppState, event_id: EventId) -> Result<Vec<ShoppingTour>> {
    state.db().events().get_shopping_tours(event_id).await
}

/// Raw building blocks for event timeline-style views: the event's meals,
/// shopping tours, food preps, and all recipes (for name lookups). Layout,
/// grouping and filtering stay presentation.
pub struct EventItemSources {
    pub meals: Vec<Meal>,
    pub tours: Vec<ShoppingTour>,
    pub preps: Vec<FoodPrep>,
    pub recipes: Vec<Recipe>,
}

pub async fn gather_event_item_sources(
    state: &AppState,
    event_id: EventId,
) -> Result<EventItemSources> {
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

/// Fetches one event without edit-permission checks (for public read-only views).
pub async fn get_event_public(state: &AppState, event_id: EventId) -> Result<Event> {
    state.db().events().get(event_id).await
}

/// Fetches the currently ongoing event, or the next upcoming event after `at`.
pub async fn get_current_or_upcoming_event(
    state: &AppState,
    at: time::OffsetDateTime,
) -> Result<Option<Event>> {
    state.db().events().get_current_or_upcoming_event(at).await
}