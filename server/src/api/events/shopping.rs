use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use foodlib_new::{
    auth_context::AuthCtx,
    entities::event::{IngredientWithoutTour, ShoppingListItem, ShoppingTour},
};

use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

/// Shopping list of an event, grouped by tour.
#[derive(serde::Serialize)]
pub struct ShoppingListDto {
    pub tours: Vec<TourWithItems>,
    /// Ingredients that no tour covers yet (their store has no tour in the
    /// relevant date window).
    pub without_tour: Vec<IngredientWithoutTour>,
}

#[derive(serde::Serialize)]
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
/// endpoint's job), plus the lifecycle flag.
#[derive(serde::Serialize)]
pub struct TourSummaryDto {
    #[serde(flatten)]
    pub tour: ShoppingTour,
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct TourInput {
    /// RFC 3339 timestamp of the tour date.
    pub tour_date: String,
    pub store_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct SourceOverrideInput {
    pub source_id: i32,
}

/// `GET /api/events/{id}/shopping-list` — tours with their items plus
/// ingredients that are not covered by any tour yet. Manually pinned items
/// (`shopping_tour_items`) override the view-derived assignment: their
/// ingredient is forced onto its tour and never appears as "without tour".
pub async fn shopping_list(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let tours = match state.events().get_shopping_tours(id).await {
        Ok(tours) => tours,
        Err(err) => return map_api_error(err).into_response(),
    };
    let items = match state.events().get_event_shopping_list(id).await {
        Ok(items) => items,
        Err(err) => return map_api_error(err).into_response(),
    };
    let without_tour = match state.events().get_ingredients_without_tour(id).await {
        Ok(items) => items,
        Err(err) => return map_api_error(err).into_response(),
    };
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
    let mut pinned_tour: std::collections::HashMap<i32, i32> =
        std::collections::HashMap::new();
    for pin in &pins {
        pinned_tour.insert(pin.ingredient_id, pin.tour_id);
    }

    let mut grouped: std::collections::HashMap<i32, Vec<ShoppingListItem>> =
        std::collections::HashMap::new();
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

    respond(Ok(ShoppingListDto {
        tours,
        without_tour,
    }))
}

/// `GET /api/events/{id}/tours` — all shopping tours of the event with
/// their lifecycle flag.
pub async fn list_tours(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let tours = match state.events().get_shopping_tours(id).await {
        Ok(tours) => tours,
        Err(err) => return map_api_error(err).into_response(),
    };
    let completed_ids = state
        .events()
        .get_completed_tour_ids(id)
        .await
        .unwrap_or_default();
    let summaries = tours
        .into_iter()
        .map(|tour| TourSummaryDto {
            completed: completed_ids.contains(&tour.id),
            tour,
        })
        .collect::<Vec<_>>();
    respond(Ok(summaries))
}

/// `POST /api/events/{id}/tours` — schedule a shopping tour for a store.
/// Ingredients sourced from that store whose `buy_by` falls into this tour's
/// date window are assigned automatically by the `shopping_tour_ingredients`
/// view — no per-item assignment needed. Extra/manual items can be pinned
/// afterwards (`POST …/tours/{tour_id}/items`).
pub async fn add_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<TourInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let tour_date = match parse_time(&payload.tour_date) {
        Ok(time) => time,
        Err(response) => return response,
    };
    let tour = ShoppingTour {
        id: 0,
        event_id: id,
        tour_date,
        store_id: payload.store_id,
        store_name: None,
    };
    respond_created(state.events().add_shopping_tour(tour).await)
}

/// `PUT /api/events/{id}/tours/{tour_id}` — reschedule or move a tour.
/// Rejected for completed tours (reopen first).
pub async fn update_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    if tour_is_completed(&state, tour_id).await {
        return bad_request("tour is completed — reopen it first".into());
    }
    let current = match state.events().get_shopping_tour(tour_id).await {
        Ok(tour) => tour,
        Err(err) => return map_api_error(err).into_response(),
    };
    if current.event_id != event_id {
        return bad_request("tour does not belong to this event".into());
    }
    let tour_date = match parse_time(&payload.tour_date) {
        Ok(time) => time,
        Err(response) => return response,
    };

    let tour = ShoppingTour {
        id: tour_id,
        event_id: current.event_id,
        tour_date,
        store_id: payload.store_id,
        store_name: current.store_name,
    };
    respond(state.events().update_shopping_tour(tour).await)
}

/// `DELETE /api/events/{id}/tours/{tour_id}` — remove the tour. Its items
/// fall back to "without tour". Rejected for completed tours.
pub async fn delete_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, tour_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    if tour_is_completed(&state, tour_id).await {
        return bad_request("tour is completed — reopen it first".into());
    }
    match state.events().delete_shopping_tour(tour_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// Body for the tour-lifecycle toggle.
#[derive(Debug, Deserialize)]
pub struct TourCompletedInput {
    pub completed: bool,
}

/// Body for pinning an ingredient onto a tour.
#[derive(Debug, Deserialize)]
pub struct TourItemInput {
    pub ingredient_id: i32,
}

/// `PUT /api/events/{id}/tours/{tour_id}/completed` — mark the tour done
/// (locked) or reopen it.
pub async fn set_tour_completed(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourCompletedInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    match state
        .events()
        .set_shopping_tour_completed(tour_id, payload.completed)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `POST /api/events/{id}/tours/{tour_id}/items` — pin an ingredient onto
/// this tour, overriding the automatic store/window assignment. Rejected for
/// completed tours.
pub async fn add_tour_item(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourItemInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    if tour_is_completed(&state, tour_id).await {
        return bad_request("tour is completed — reopen it first".into());
    }
    match state
        .events()
        .add_shopping_tour_item(tour_id, payload.ingredient_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/events/{id}/tours/{tour_id}/items/{ingredient_id}` — unpin a
/// manually assigned ingredient (falls back to the automatic assignment).
pub async fn remove_tour_item(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, tour_id, ingredient_id)): Path<(i32, i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    if tour_is_completed(&state, tour_id).await {
        return bad_request("tour is completed — reopen it first".into());
    }
    match state
        .events()
        .remove_shopping_tour_item(tour_id, ingredient_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// Whether the tour is completed (locked) — `false` when the status row is
/// missing or unreadable (a locked tour must never become editable by
/// accident, but a broken status table should not lock everything either).
async fn tour_is_completed(state: &MyAppState, tour_id: i32) -> bool {
    state
        .events()
        .is_shopping_tour_completed(tour_id)
        .await
        .unwrap_or(false)
}

/// `GET /api/events/{id}/source-overrides` — per-event reroutes of
/// ingredients to a different source/store.
pub async fn list_overrides(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.events().get_source_overrides(id).await)
}

/// `POST /api/events/{id}/source-overrides` — reroute an ingredient source
/// for this event (idempotent upsert).
pub async fn add_override(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<SourceOverrideInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond_created(
        state
            .events()
            .set_source_override(id, payload.source_id)
            .await,
    )
}

/// `DELETE /api/events/{id}/source-overrides/{source_id}` — remove a reroute.
pub async fn delete_override(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(event_id).await {
        return map_api_error(err).into_response();
    }
    match state
        .events()
        .delete_source_override(event_id, source_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

fn parse_time(value: &str) -> Result<OffsetDateTime, Response> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| bad_request(format!("invalid RFC 3339 timestamp: {value}")))
}
