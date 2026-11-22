//! Event use-cases: CRUD, ownership transfer, detail composition and
//! shopping-tour management.

use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::audit;
use super::{assert_ownership_transfer, AppCtx, Error, MyAppState, Result};
use crate::app::lib::parse_time;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::{
    Event, EventSummary, IngredientWithoutTour, ShoppingListItem, ShoppingTour,
};
use foodlib_new::entities::meal::Meal;
use foodlib_new::entities::types::ids::{EventId, GroupId, IngredientId, StoreId, TourId};
use foodlib_new::entities::types::money::Money;
use foodlib_new::policy::{EventPerm, GlobalPermission, Permission};

pub mod exports;
pub mod logistics;
pub mod meals;
pub mod members;
pub mod shopping;

// ------------------------------------------------------------------ read

/// all events of the user's groups with planner totals,
/// newest activity first. The op already scopes the result to the caller's
/// groups.
pub async fn list_summaries(state: &MyAppState, ctx: &AuthCtx) -> Result<Vec<EventSummary>> {
    state.events().list_summaries_for_user(ctx.user.id).await
}

/// Fully composed event detail for `GET /api/events/{id}`: frame data,
/// planned meals, total cost and the caller's resolved permissions for the
/// event (covers direct collaborators that are not members of the owning
/// group).
#[derive(Debug, Serialize)]
pub struct EventDetails {
    pub event: Event,
    pub meals: Vec<Meal>,
    pub total_cost: Money,
    pub permissions: Vec<Permission>,
}

pub async fn get_event_details(
    state: &MyAppState,
    ctx: &AuthCtx,
    id: EventId,
) -> Result<EventDetails> {
    ctx.assert_event_action(id, EventPerm::ViewEvent).await?;
    let event = state.events().get(id).await?;
    let (meals, total_cost) = {
        let meals_ops = state.meals();
        let events_ops = state.events();
        tokio::try_join!(meals_ops.get_event_meals(id), events_ops.get_total_cost(id))?
    };
    let permissions = ctx.policy.resolved_event_permissions(id, event.group_id);
    Ok(EventDetails {
        event,
        meals,
        total_cost,
        permissions,
    })
}

// ----------------------------------------------------------------- frame

#[derive(Debug, Deserialize)]
pub struct EventCreateInput {
    pub name: String,
    pub comment: Option<String>,
    /// Budget as a decimal string (Amount over the wire).
    pub budget: Option<String>,
    pub group_id: GroupId,
}

pub async fn create_new_event(
    state: &MyAppState,
    actor: &AppCtx,
    input: EventCreateInput,
) -> Result<Event> {
    let ctx = &actor.ctx;
    ctx.assert_global(GlobalPermission::CreateResources)?;
    ctx.assert_group_perm(input.group_id, Permission::Event(EventPerm::CreateEvent))?;
    let budget = parse_budget(input.budget)?;

    let event = Event {
        id: EventId::default(),
        name: input.name,
        comment: input.comment,
        budget,
        group_id: input.group_id,
    };
    let created = state.events().create(event).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_CREATED,
        Some("event"),
        Some(created.id.into()),
        Some(created.group_id),
        Some(created.id),
        Some(json!({ "name": created.name })),
    )
        .await;
    Ok(created)
}

#[derive(Debug, Deserialize)]
pub struct EventUpdateInput {
    pub name: Option<String>,
    /// `null` keeps the current comment; an empty string clears it.
    pub comment: Option<String>,
    pub budget: Option<String>,
}

pub async fn update_event(
    state: &MyAppState,
    actor: &AppCtx,
    id: EventId,
    input: EventUpdateInput,
) -> Result<Event> {
    actor.ctx.assert_event_action(id, EventPerm::EditEvent).await?;
    let current = state.events().get(id).await?;
    let budget = match input.budget {
        Some(raw) if raw.trim().is_empty() => None,
        Some(raw) => parse_budget(Some(raw))?,
        None => current.budget,
    };
    let comment = match input.comment {
        Some(comment) if comment.is_empty() => None,
        Some(comment) => Some(comment),
        None => current.comment,
    };

    let updated = Event {
        id: current.id,
        name: input.name.unwrap_or(current.name),
        comment,
        budget,
        group_id: current.group_id,
    };
    state.events().update(updated).await
}

/// delete the event (cascades to meals/tours).
pub async fn delete_event(state: &MyAppState, actor: &AppCtx, id: EventId) -> Result<()> {
    actor.ctx.assert_event_action(id, EventPerm::DeleteEvent).await?;
    state.events().delete(id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("event"),
        Some(id.into()),
        None,
        Some(id),
        None,
    )
        .await;
    Ok(())
}

pub async fn clone_created_event(state: &MyAppState, actor: &AppCtx, id: EventId) -> Result<EventId> {
    actor.ctx.assert_event_action(id, EventPerm::CloneEvent).await?;
    let event = state.events().get(id).await?;
    let cloned = state.events().duplicate(id, event.group_id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_CREATED,
        Some("event"),
        Some(cloned.into()),
        Some(event.group_id),
        Some(cloned),
        Some(json!({ "cloned_from": id })),
    )
        .await;
    Ok(cloned)
}

/// explicit Besitz-Übertragung: erlaubt für
/// den Besitzer (persönliche Gruppe) oder Manager der besitzenden Gruppe;
/// die Zielgruppe muss für den Aufrufer zugänglich sein (Admin oder
/// Mitgliedschaft). Danach regelt die Manager-Rolle der neuen Gruppe.
pub async fn transfer_event_owner(
    state: &MyAppState,
    actor: &AppCtx,
    id: EventId,
    target_group_id: GroupId,
) -> Result<()> {
    let current = state.events().get(id).await?;
    assert_ownership_transfer(
        &actor.ctx,
        "event",
        current.group_id,
        target_group_id,
        Permission::Event(EventPerm::TransferOwner),
    )?;
    state.events().set_group(id, target_group_id).await?;
    audit::record(
        state,
        actor,
        audit::action::OWNER_TRANSFERRED,
        Some("event"),
        Some(id.into()),
        Some(target_group_id),
        Some(id),
        Some(json!({ "from_group_id": current.group_id })),
    )
        .await;
    Ok(())
}

/// Parses a budget decimal string; an empty string clears the budget.
fn parse_budget(budget: Option<String>) -> Result<Option<Money>> {
    match budget.as_deref() {
        None | Some("") => Ok(None),
        Some(value) => Money::from_str(value)
            .map(Some)
            .map_err(|_| Error::Validation {
                message: format!("invalid budget: {value}"),
            }),
    }
}

// ---------------------------------------------------------- shopping list

/// Shopping list of an event, grouped by tour.
#[derive(Serialize)]
pub struct ShoppingListDto {
    pub tours: Vec<TourWithItems>,
    /// Ingredients that no tour covers yet (their store has no tour in the
    /// relevant date window).
    pub without_tour: Vec<IngredientWithoutTour>,
}

#[derive(Serialize)]
pub struct TourWithItems {
    #[serde(flatten)]
    pub tour: ShoppingTour,
    pub items: Vec<ShoppingListItem>,
    /// Sum of the item prices; `None` when any item has no priced source.
    pub total_price: Option<Money>,
    /// Completed tours are locked: no edits, no item changes (reopen first).
    pub completed: bool,
}

/// Tour summary for `GET /tours` (no item list — that's the shopping-list
/// use-case's job), plus the lifecycle flag.
#[derive(Serialize)]
pub struct TourSummaryDto {
    #[serde(flatten)]
    pub tour: ShoppingTour,
    pub completed: bool,
}

/// List all shopping tours of an event, with their items and the lifecycle flag.
/// Requires `ViewEvent` permission. Ingredients without a tour are listed separately.
pub async fn get_shopping_list(
    state: &MyAppState,
    actor: &AppCtx,
    id: EventId,
) -> Result<ShoppingListDto> {
    // Permission check: the caller must be able to view the event to see its shopping list.
    actor.ctx.assert_event_action(id, EventPerm::ViewEvent).await?;

    // Action
    let event = state.events().get(id).await?;
    let can_view_prices = actor.ctx.has_event_permission(
        id,
        event.group_id,
        Permission::Event(EventPerm::ViewCalculation),
    );

    let tours = state.events().get_shopping_tours(id).await?;
    let items = state.events().get_event_shopping_list(id).await?;
    let without_tour = state.events().get_ingredients_without_tour(id).await?;
    let pins = state
        .events()
        .get_shopping_tour_pins(id)
        .await
        .unwrap_or_default();
    let completed_ids = state
        .events()
        .get_completed_tour_ids(id)
        .await
        .unwrap_or_default();

    // Manual pins: ingredient -> tour. A pinned ingredient is routed to its
    // tour regardless of what the view derived.
    let pinned_tour: HashMap<IngredientId, TourId> = pins
        .iter()
        .map(|pin| (pin.ingredient_id, pin.tour_id))
        .collect();

    let mut grouped: HashMap<TourId, Vec<ShoppingListItem>> = HashMap::new();
    for mut item in items {
        if !can_view_prices {
            item.price = None;
        }
        // Pin wins over the view-derived tour_id (which may be `None`).
        let target_tour = pinned_tour
            .get(&item.ingredient_id)
            .copied()
            .or(item.tour_id);
        if let Some(tour_id) = target_tour {
            grouped.entry(tour_id).or_default().push(item);
        }
    }

    let tours = tours
        .into_iter()
        .map(|tour| {
            let items = grouped.remove(&tour.id).unwrap_or_default();
            let total_price = if can_view_prices {
                items.iter().try_fold(
                    Money::from(0),
                    |sum: Money, item| match &item.price {
                        Some(price) => Some(sum + price),
                        // Missing price on any item -> no reliable tour total.
                        None => None,
                    },
                )
            } else {
                None
            };
            TourWithItems {
                completed: completed_ids.contains(&tour.id),
                tour,
                items,
                total_price,
            }
        })
        .collect();

    // Pinned ingredients are assigned by definition — never "without tour".
    let without_tour = without_tour
        .into_iter()
        .filter(|item| !pinned_tour.contains_key(&item.ingredient_id))
        .collect();

    Ok(ShoppingListDto {
        tours,
        without_tour,
    })
}

/// List all shopping tours of an event, with the lifecycle flag.
/// Requires `ViewEvent` permission.
pub async fn list_shopping_tours_for_event(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: EventId,
) -> Result<Vec<TourSummaryDto>> {
    // Permission check
    ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;

    // Action
    let tours = state.events().get_shopping_tours(event_id).await?;
    let completed_ids = state
        .events()
        .get_completed_tour_ids(event_id)
        .await
        .unwrap_or_default();

    Ok(tours
        .into_iter()
        .map(|tour| TourSummaryDto {
            completed: completed_ids.contains(&tour.id),
            tour,
        })
        .collect())
}

// -------------------------------------------------------- shopping tours

#[derive(Debug, Deserialize)]
pub struct TourInput {
    /// RFC 3339 timestamp of the tour date.
    pub tour_date: String,
    pub store_id: StoreId,
}

/// Add a new shopping tour to an event.
/// Requires `ShoppingAddTour` permission. The tour is initially empty (no items).
pub async fn add_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    input: TourInput,
) -> Result<ShoppingTour> {
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingTourCreate).await?;

    let tour = ShoppingTour {
        id: TourId::default(),
        event_id,
        tour_date: parse_time(&input.tour_date)?,
        store_id: input.store_id,
        store_name: None,
    };
    let created = state.events().add_shopping_tour(tour).await?;

    audit::record(
        state,
        actor,
        "event.tour.added",
        Some("shopping_tour"),
        Some(created.id.into()),
        None,
        Some(event_id),
        Some(json!({ "store_id": created.store_id })),
    )
        .await;

    Ok(created)
}

/// Allows to reschedule or change the store of a shopping tour.
/// Rejected for completed tours. Requires `ShoppingEditTour` permission.
pub async fn update_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
    input: TourInput,
) -> Result<ShoppingTour> {
    // Permission check
    // actor.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingTourEdit).await?;

    let current = get_editable_tour(state, actor, event_id, tour_id).await?;

    // Action
    let tour = ShoppingTour {
        id: tour_id,
        event_id: current.event_id,
        tour_date: parse_time(&input.tour_date)?,
        store_id: input.store_id,
        store_name: current.store_name,
    };
    let updated = state.events().update_shopping_tour(tour).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour.updated",
        Some("shopping_tour"),
        Some(tour_id.into()),
        None,
        Some(event_id),
        None,
    )
        .await;
    Ok(updated)
}

/// Remove a shopping tour from an event.
/// Rejected for completed tours. Requires `EditShoppingTour` permission.
pub async fn delete_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
) -> Result<()> {
    // Permission & semantic checks
    // actor.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingTourEdit).await?;
    // reopen must use the event-validating lookup: completed tours may be
    // reopened (completed: false), only re-completing a completed tour is an error.
    let tour = get_event_tour(state, actor, event_id, tour_id).await?;
    // sanity check: the tour must belong to this event
    if tour.id != tour_id {
        return Err(Error::Validation {
            message: "tour does not belong to this event".into(),
        });
    }

    // Action
    state.events().delete_shopping_tour(tour_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour.deleted",
        Some("shopping_tour"),
        Some(tour_id.into()),
        None,
        Some(event_id),
        None,
    )
        .await;
    Ok(())
}

/// Mark a tour as completed (locked) or reopen it. Completed tours are read-only: no edits, no item changes.
/// completed tours can be reopened. Completing a completed tour is an error.
/// Requires `ShoppingTourComplete` permission.
pub async fn set_tour_completed(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
    completed: bool,
) -> Result<()> {
    // Permission & semantic checks
    actor.ctx.assert_event_actions(event_id, [
        EventPerm::ShoppingTourComplete
    ]).await?;
    let tour = get_event_tour(state, actor, event_id, tour_id).await?;
    // sanity check: the tour must belong to this event
    if tour.id != tour_id {
        return Err(Error::Validation {
            message: "tour does not belong to this event".into(),
        });
    }
    if completed && state.events().is_shopping_tour_completed(tour_id).await.unwrap_or(false) {
        return Err(Error::Validation {
            message: "tour is already completed".into(),
        });
    }

    // Action
    state.events().set_shopping_tour_completed(tour_id, completed).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour.completed_changed",
        Some("shopping_tour"),
        Some(tour_id.into()),
        None,
        Some(event_id),
        Some(json!({ "completed": completed })),
    )
        .await;
    Ok(())
}

/// Add one ingredient to a tour (manual pin). Rejected for completed tours.
/// Requires `ShoppingEditTour` permission.
pub async fn add_tour_item(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
    ingredient_id: IngredientId,
) -> Result<()> {
    // Permission & semantic checks
    actor.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingTourEdit).await?;
    // completing a completed tour is an error
    let tour = get_editable_tour(state, actor, event_id, tour_id).await?;
    // sanity check: the tour must belong to this event
    if tour.id != tour_id {
        return Err(Error::Validation {
            message: "tour does not belong to this event".into(),
        });
    }

    // Action
    state.events().add_shopping_tour_item(tour_id, ingredient_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour_item.added",
        Some("shopping_tour_item"),
        Some(tour_id.into()),
        None,
        Some(event_id),
        Some(json!({ "ingredient_id": ingredient_id })),
    )
        .await;
    Ok(())
}

/// Remove one ingredient from a tour (manual unpin). Rejected for completed tours.
/// Requires `ShoppingEditTour` permission.
pub async fn remove_tour_item(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
    ingredient_id: IngredientId,
) -> Result<()> {
    // Permission & semantic checks
    actor.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingTourEdit).await?;
    // completing a completed tour is an error
    let tour = get_editable_tour(state, actor, event_id, tour_id).await?;
    // sanity check: the tour must belong to this event
    if tour.id != tour_id {
        return Err(Error::Validation {
            message: "tour does not belong to this event".into(),
        });
    }

    // Action
    state.events().remove_shopping_tour_item(tour_id, ingredient_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour_item.removed",
        Some("shopping_tour_item"),
        Some(tour_id.into()),
        None,
        Some(event_id),
        Some(json!({ "ingredient_id": ingredient_id })),
    )
        .await;
    Ok(())
}

/// Fetches a tour for an event.
/// Ensures the tour belongs to `event_id` and checks `ViewEvent` permission.
async fn get_event_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
) -> Result<ShoppingTour> {
    // Permission & semantic checks
    actor.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;

    // Action - fetch the tour regardless of its final state (completed or not)
    let tour = state.events().get_shopping_tour(tour_id).await?;

    // sanity check: the tour must belong to this event
    if tour.id != tour_id {
        return Err(Error::Validation {
            message: "tour does not belong to this event".into(),
        });
    }

    Ok(tour)
}

/// Fetches a tour which is currently not completed.
/// Returns an error if the tour is already completed.
/// Checks that the tour belongs to `event_id` and checks `ViewEvent` permission.
async fn get_editable_tour(
    state: &MyAppState,
    app_ctx: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
) -> Result<ShoppingTour> {

    let tour = get_event_tour(state, &app_ctx, event_id, tour_id).await?;
    if state
        .events()
        .is_shopping_tour_completed(tour_id)
        .await
        .unwrap_or(false)
    {
        return Err(Error::Validation {
            message: "tour is completed — reopen it first".into(),
        });
    }
    Ok(tour)
}