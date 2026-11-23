//! Event use-cases.

use bigdecimal::FromPrimitive;
use foodlib_new::entities::types::ids::{
    EventId, GroupId,
};
use foodlib_new::entities::types::money::Money;
use foodlib_new::event::{CreateEvent, Event, FoodPrep, ShoppingTour};
use foodlib_new::ingredient::Ingredient;
use foodlib_new::meal::Meal;
use foodlib_new::recipe::Recipe;
use foodlib_new::store::Store;

use crate::app_ctx::{AppCtx, assert_can_move_to};
use crate::{MyAppState, Result};

pub mod meals;
pub mod shopping;
pub mod timeline;
pub mod logistics;

/// Creates a new event owned by the actor's personal group. An empty comment
/// is normalized to `None`.
pub async fn create_event(
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
    comment: Option<String>,
    budget: Option<f64>,
) -> Result<()> {
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    state
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
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<EventId> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    state.events().duplicate(event_id, group.id).await
}

/// Deletes an event.
/// Requires `EditEvent` permission.
pub async fn delete_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    state.events().delete(event_id).await?;
    Ok(())
}

/// Updates an event's metadata and optionally transfers it to another group.
/// An empty comment is normalized to `None`.
/// Requires `EditEvent` permission; moving to another group additionally
/// requires membership in (or admin over) the target group.
pub async fn update_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    name: String,
    comment: String,
    budget: Option<f64>,
    target_group: GroupId,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let current = state.events().get(event_id).await?;
    if current.group_id != target_group {
        assert_can_move_to(actor, target_group)?;
    }

    state
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
        state.events().set_group(event_id, target_group).await?;
    }
    Ok(())
}

/// Everything the event edit form renders, fetched in one use case so
/// transports only have to deal with presentation.
pub struct EventFormData {
    pub event: Event,
    pub stores: Vec<Store>,
    pub overrides: Vec<foodlib_new::event::SourceOverrideView>,
    pub ingredients: Vec<Ingredient>,
    pub meals: Vec<Meal>,
    pub recipes: Vec<Recipe>,
    pub preps: Vec<FoodPrep>,
}

/// Collects all data needed to render the event edit form.
/// Requires `EditEvent` permission.
pub async fn gather_form_data(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<EventFormData> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let stores = state.stores().list().await?;
    let overrides = state.events().get_source_overrides(event_id).await?;
    let ingredients = state.ingredients().list().await?;
    let meals = state.meals().get_event_meals(event_id).await?;
    let recipes = state.recipes().list().await?;
    let preps = state.events().get_food_prep_tasks(event_id).await?;
    let event = state.events().get(event_id).await?;

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
pub async fn get_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<Event> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    state.events().get(event_id).await
}

/// Lists all events.
pub async fn list_events(state: &MyAppState) -> Result<Vec<Event>> {
    state.events().list().await
}

/// Lists all places (meal locations).
pub async fn list_places(state: &MyAppState) -> Result<Vec<foodlib_new::event::Place>> {
    state.events().get_places().await
}

/// Lists an event's ingredient source default overrides.
/// Requires `EditEvent` permission.
pub async fn list_source_overrides(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<Vec<foodlib_new::event::SourceOverrideView>> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    state.events().get_source_overrides(event_id).await
}

/// Lists an event's shopping tours.
pub async fn get_shopping_tours(
    state: &MyAppState,
    event_id: EventId,
) -> Result<Vec<ShoppingTour>> {
    state.events().get_shopping_tours(event_id).await
}

/// Raw building blocks for event timeline-style views: the event's meals,
/// shopping tours, food preps, and all recipes (for name lookups). Layout,
/// grouping and filtering stay presentation.
pub struct EventItemSources {
    pub meals: Vec<Meal>,
    pub tours: Vec<foodlib_new::event::ShoppingTour>,
    pub preps: Vec<FoodPrep>,
    pub recipes: Vec<Recipe>,
}

pub async fn gather_event_item_sources(
    state: &MyAppState,
    event_id: EventId,
) -> Result<EventItemSources> {
    let meals = state.meals().get_event_meals(event_id).await?;
    let tours = state.events().get_shopping_tours(event_id).await?;
    let preps = state.events().get_food_prep_tasks(event_id).await?;
    let recipes = state.recipes().list().await?;
    Ok(EventItemSources {
        meals,
        tours,
        preps,
        recipes,
    })
}
