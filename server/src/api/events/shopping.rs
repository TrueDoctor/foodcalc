//! Shopping-tour endpoints for events — thin handlers over
//! `crate::app::events` and `crate::app::events::shopping` (source overrides).

use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use foodlib_new::entities::types::ids::{EventId, IngredientId, IngredientSourceId, TourId};
use serde::Deserialize;

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::events::shopping;
use crate::app::{events, AppCtx};

// DTOs shared with the app layer.
pub use crate::app::events::TourInput;

#[derive(Debug, Deserialize)]
pub struct TourCompletedInput {
    pub completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct TourItemInput {
    pub ingredient_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct SourceOverrideInput {
    pub source_id: i32,
}

/// `GET /api/events/{id}/shopping-list` — tours with their items plus
/// ingredients that are not covered by any tour yet.
pub async fn shopping_list(
    State(state): State<crate::MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(events::get_shopping_list(&state, &actor, EventId(id)).await)
}

/// `GET /api/events/{id}/tours` — all shopping tours of the event with
/// their lifecycle flag.
pub async fn list_tours(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(events::list_shopping_tours_for_event(&state, &ctx.ctx, EventId(id)).await)
}

/// `POST /api/events/{id}/tours` — schedule a shopping tour for a store.
pub async fn add_tour(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<TourInput>,
) -> Response {
    respond_created(events::add_tour(&state, &ctx, EventId(id), payload).await)
}

/// `PUT /api/events/{id}/tours/{tour_id}` — reschedule or move a tour.
/// Rejected for completed tours (reopen first).
pub async fn update_tour(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourInput>,
) -> Response {
    respond(events::update_tour(&state, &ctx, EventId(event_id), TourId(tour_id), payload).await)
}

/// `DELETE /api/events/{id}/tours/{tour_id}` — remove the tour. Its items
/// fall back to "without tour". Rejected for completed tours.
pub async fn delete_tour(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(events::delete_tour(&state, &ctx, EventId(event_id), TourId(tour_id)).await)
}

/// `PUT /api/events/{id}/tours/{tour_id}/completed` — mark the tour done
/// (locked) or reopen it.
pub async fn set_tour_completed(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourCompletedInput>,
) -> Response {
    respond_no_content(
        events::set_tour_completed(&state, &ctx, EventId(event_id), TourId(tour_id), payload.completed).await,
    )
}

/// `POST /api/events/{id}/tours/{tour_id}/items` — pin an ingredient onto
/// this tour, overriding the automatic store/window assignment. Rejected for
/// completed tours.
pub async fn add_tour_item(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourItemInput>,
) -> Response {
    respond_no_content(
        events::add_tour_item(&state, &ctx, EventId(event_id), TourId(tour_id), IngredientId(payload.ingredient_id)).await,
    )
}

/// `DELETE /api/events/{id}/tours/{tour_id}/items/{ingredient_id}` — unpin a
/// manually assigned ingredient (falls back to the automatic assignment).
pub async fn remove_tour_item(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, tour_id, ingredient_id)): Path<(i32, i32, i32)>,
) -> Response {
    respond_no_content(
        events::remove_tour_item(&state, &ctx, EventId(event_id), TourId(tour_id), IngredientId(ingredient_id)).await,
    )
}

// -------------------------------------------------------- source overrides

/// `GET /api/events/{id}/source-overrides` — per-event reroutes of
/// ingredients to a different source/store.
pub async fn list_overrides(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(shopping::list_overrides(&state, &ctx, EventId(id)).await)
}

/// `POST /api/events/{id}/source-overrides` — reroute an ingredient source
/// for this event (idempotent upsert).
pub async fn add_override(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<SourceOverrideInput>,
) -> Response {
    respond_created(shopping::add_default_override(&state, &ctx, EventId(id), IngredientSourceId(payload.source_id)).await)
}

/// `DELETE /api/events/{id}/source-overrides/{source_id}` — remove a reroute.
pub async fn delete_override(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(shopping::delete_override(&state, &ctx, EventId(event_id), IngredientSourceId(source_id)).await)
}
