//! Event use-cases: CRUD, ownership transfer, detail composition and
//! shopping-tour management.

use std::collections::HashMap;
use std::str::FromStr;

use bigdecimal::BigDecimal;
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
use foodlib_new::policy::{EventPerm, GlobalPermission, Permission};

pub mod exports;
pub mod logistics;
pub mod meals;
pub mod members;
pub mod shopping;
// ------------------------------------------------------------------ read

/// `GET /api/events` — all events of the user's groups with planner totals,
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
    pub total_cost: BigDecimal,
    pub permissions: Vec<Permission>,
}

pub async fn get_event_details(state: &MyAppState, ctx: &AuthCtx, id: i32) -> Result<EventDetails> {
    ctx.assert_event_action(id, EventPerm::View).await?;
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
    /// Budget as a decimal string (BigDecimal over the wire).
    pub budget: Option<String>,
    pub group_id: i32,
}

/// `POST /api/events` — create an event owned by one of the user's groups.
pub async fn create_new_event(state: &MyAppState, actor: &AppCtx, input: EventCreateInput) -> Result<Event> {
    let ctx = &actor.ctx;
    ctx.assert_global(GlobalPermission::AllowedToCreateNewDomainObjects)?;
    ctx.assert_group_perm(input.group_id, Permission::Event(EventPerm::Create))?;
    let budget = parse_budget(input.budget)?;

    let event = Event {
        id: 0,
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
        Some(created.id),
        Some(created.group_id),
        None,
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

/// `PUT /api/events/{id}` — update the event's frame data. Provided fields
/// win; omitted fields keep the current values.
pub async fn update_event(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
    input: EventUpdateInput,
) -> Result<Event> {
    let ctx = &actor.ctx;
    ctx.assert_event_action(id, EventPerm::Edit).await?;
    let current = state.events().get(id).await?;
    let budget = parse_budget(input.budget)?.or(current.budget);
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

/// `DELETE /api/events/{id}` — delete the event (cascades to meals/tours).
pub async fn delete_event(state: &MyAppState, actor: &AppCtx, id: i32) -> Result<()> {
    actor.ctx.assert_event_action(id, EventPerm::Delete).await?;
    state.events().delete(id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("event"),
        Some(id),
        None,
        Some(id),
        None,
    )
    .await;
    Ok(())
}

/// `POST /api/events/{id}/clone` — duplicate the event incl. meals and source
/// overrides. Returns the new event id.
pub async fn clone_created_event(state: &MyAppState, actor: &AppCtx, id: i32) -> Result<i32> {
    let ctx = &actor.ctx;
    ctx.assert_event_action(id, EventPerm::Clone).await?;
    let event = state.events().get(id).await?;
    let new_id = state.events().duplicate(id, event.group_id).await?;
    audit::record(
        state,
        actor,
        audit::action::ENTITY_CREATED,
        Some("event"),
        Some(new_id),
        Some(event.group_id),
        Some(id),
        Some(json!({ "cloned_from": id })),
    )
    .await;
    Ok(new_id)
}

/// `PUT /api/events/{id}/owner` — explicit Besitz-Übertragung: erlaubt für
/// den Besitzer (persönliche Gruppe) oder Manager der besitzenden Gruppe;
/// die Zielgruppe muss für den Aufrufer zugänglich sein (Admin oder
/// Mitgliedschaft). Danach regelt die Manager-Rolle der neuen Gruppe.
pub async fn transfer_event_owner(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
    target_group_id: i32,
) -> Result<()> {
    let ctx = &actor.ctx;
    let current = state.events().get(id).await?;
    assert_ownership_transfer(
        ctx,
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
        Some(id),
        Some(target_group_id),
        Some(id),
        Some(json!({ "from_group_id": current.group_id })),
    )
    .await;
    Ok(())
}

/// Parses a budget decimal string; an empty string clears the budget.
fn parse_budget(budget: Option<String>) -> Result<Option<BigDecimal>> {
    match budget.as_deref() {
        None | Some("") => Ok(None),
        Some(value) => BigDecimal::from_str(value)
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
    pub total_price: Option<BigDecimal>,
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

/// `GET /api/events/{id}/shopping-list` — tours with their items plus
/// ingredients that are not covered by any tour yet. Manually pinned items
/// (`shopping_tour_items`) override the view-derived assignment: their
/// ingredient is forced onto its tour and never appears as "without tour".
pub async fn get_shopping_list(state: &MyAppState, ctx: &AuthCtx, id: i32) -> Result<ShoppingListDto> {
    // Check Permissions
    ctx.assert_event_action(id, EventPerm::View).await?;

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

    // Manual pins: ingredient → tour. A pinned ingredient is routed to its
    // tour regardless of what the view derived.
    let mut pinned_tour: HashMap<i32, i32> = HashMap::new();
    for pin in &pins {
        pinned_tour.insert(pin.ingredient_id, pin.tour_id);
    }

    let mut grouped: HashMap<i32, Vec<ShoppingListItem>> = HashMap::new();
    for item in items {
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
            let total_price = items.iter().try_fold(
                BigDecimal::from(0),
                |sum, item| match &item.price {
                    Some(price) => Some(sum + price),
                    // Missing price on any item → no reliable tour total.
                    None => None,
                },
            );
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

/// `GET /api/events/{id}/tours` — all shopping tours of the event with
/// their lifecycle flag.
pub async fn list_shopping_tours_for_event(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: i32
) -> Result<Vec<TourSummaryDto>> {
    ctx.assert_event_action(event_id, EventPerm::View).await?;
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
    pub store_id: i32,
}

/// `POST /api/events/{id}/tours` — schedule a shopping tour for a store.
/// Ingredients sourced from that store whose `buy_by` falls into this tour's
/// date window are assigned automatically by the `shopping_tour_ingredients`
/// view — no per-item assignment needed. Extra/manual items can be pinned
/// afterwards (`add_tour_item`).
pub async fn add_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    input: TourInput,
) -> Result<ShoppingTour> {
    // Check permissions
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingAddTour).await?;

    // Create the tour
    let tour = ShoppingTour {
        id: 0,
        event_id,
        tour_date: parse_time(&input.tour_date)?,
        store_id: input.store_id,
        store_name: None,
    };
    let created = state.events().add_shopping_tour(tour).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour.added",
        Some("shopping_tour"),
        Some(created.id),
        None,
        Some(event_id),
        Some(json!({ "store_id": created.store_id })),
    )
    .await;

    // Response
    Ok(created)
}

/// `PUT /api/events/{id}/tours/{tour_id}` — reschedule or move a tour.
/// Rejected for completed tours (reopen first).
pub async fn update_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    tour_id: i32,
    input: TourInput,
) -> Result<ShoppingTour> {
    // Check permissions
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingEditTour).await?;

    // Check lifecycle
    if tour_is_completed(state, actor, tour_id).await {
        return Err(Error::Validation {
            message: "tour is completed — reopen it first".into(),
        });
    }
    let current = state.events().get_shopping_tour(tour_id).await?;
    if current.event_id != event_id {
        return Err(Error::Validation {
            message: "tour does not belong to this event".into(),
        });
    }

    // Update the tour
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
        Some(tour_id),
        None,
        Some(event_id),
        None,
    )
    .await;
    Ok(updated)
}

/// `DELETE /api/events/{id}/tours/{tour_id}` — remove the tour. Its items
/// fall back to "without tour". Rejected for completed tours.
pub async fn delete_tour(state: &MyAppState, actor: &AppCtx, event_id: i32, tour_id: i32) -> Result<()> {
    // Check permissions
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingEditTour).await?;

    // Check lifecycle
    if tour_is_completed(state, actor, tour_id).await {
        return Err(Error::Validation {
            message: "tour is completed — reopen it first".into(),
        });
    }

    // Delete the tour
    state.events().delete_shopping_tour(tour_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour.deleted",
        Some("shopping_tour"),
        Some(tour_id),
        None,
        Some(event_id),
        None,
    )
    .await;
    Ok(())
}

/// `PUT /api/events/{id}/tours/{tour_id}/completed` — mark the tour done
/// (locked) or reopen it.
pub async fn set_tour_completed(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    tour_id: i32,
    completed: bool,
) -> Result<()> {
    // Check permissions
    actor.ctx.assert_event_action(event_id, EventPerm::ManageShopping).await?;

    // Update the lifecycle flag
    state.events().set_shopping_tour_completed(tour_id, completed).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour.completed_changed",
        Some("shopping_tour"),
        Some(tour_id),
        None,
        Some(event_id),
        Some(json!({ "completed": completed })),
    )
    .await;
    Ok(())
}

/// `POST /api/events/{id}/tours/{tour_id}/items` — pin an ingredient onto
/// this tour, overriding the automatic store/window assignment. Rejected for
/// completed tours.
pub async fn add_tour_item(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    tour_id: i32,
    ingredient_id: i32,
) -> Result<()> {
    // Check permissions
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingEditTour).await?;

    // Check lifecycle
    if tour_is_completed(state, actor, tour_id).await {
        return Err(Error::Validation {
            message: "tour is completed — reopen it first".into(),
        });
    }

    // Add the item
    state.events().add_shopping_tour_item(tour_id, ingredient_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour_item.added",
        Some("shopping_tour_item"),
        Some(tour_id),
        None,
        Some(event_id),
        Some(json!({ "ingredient_id": ingredient_id })),
    )
    .await;
    Ok(())
}

/// `DELETE /api/events/{id}/tours/{tour_id}/items/{ingredient_id}` — unpin a
/// manually assigned ingredient (falls back to the automatic assignment).
/// Rejected for completed tours.
pub async fn remove_tour_item(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    tour_id: i32,
    ingredient_id: i32,
) -> Result<()> {
    // Check permissions
    actor.ctx.assert_event_action(event_id, EventPerm::ShoppingEditTour).await?;

    // Check lifecycle
    if tour_is_completed(state, actor, tour_id).await {
        return Err(Error::Validation {
            message: "tour is completed — reopen it first".into(),
        });
    }

    // Remove the item
    state.events().remove_shopping_tour_item(tour_id, ingredient_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.tour_item.removed",
        Some("shopping_tour_item"),
        Some(tour_id),
        None,
        Some(event_id),
        Some(json!({ "ingredient_id": ingredient_id })),
    )
    .await;
    Ok(())
}

/// Whether the tour is completed (locked) — `false` when the status row is
/// missing or unreadable (a locked tour must never become editable by
/// accident, but a broken status table should not lock everything either).
async fn tour_is_completed(
    state: &MyAppState,
    actor: &AppCtx,
    tour_id: i32
) -> bool {
    // Check permissions
    actor.ctx.assert_event_action(tour_id, EventPerm::View).await.ok();

    // Resolve and respond
    state.events().is_shopping_tour_completed(tour_id).await
        .unwrap_or(false)
}

