//! Logistics endpoints beyond shopping tours — thin handlers over
//! `crate::app::events::logistics`: food prep read access and event-inventory
//! links with per-ingredient coverage.

use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Router,
};

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::events::logistics;
use crate::app::AppCtx;
use crate::MyAppState;

pub fn logistics_router() -> Router<MyAppState> {
    Router::new()
        .route("/{id}/preps", get(list_preps))
        .route("/{id}/inventories", get(list_event_inventories))
        .route(
            "/{id}/inventories/{inventory_id}",
            post(add_event_inventory).delete(remove_event_inventory),
        )
        .route("/{id}/inventory-coverage", get(inventory_coverage))
}

/// `GET /api/events/{id}/preps` — food prep tasks with recipe names
/// (timeline feed).
async fn list_preps(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(logistics::list_food_preps(&state, &ctx.ctx, id).await)
}

/// `GET /api/events/{id}/inventories` — the event's linked inventories.
async fn list_event_inventories(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(logistics::list_event_inventories(&state, &ctx.ctx, id).await)
}

/// `POST /api/events/{id}/inventories/{inventory_id}` — link an inventory to
/// the event. Requires event write access and read access to the inventory.
async fn add_event_inventory(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path((id, inventory_id)): Path<(i32, i32)>,
) -> Response {
    respond_created(logistics::link_inventory(&state, &ctx, id, inventory_id).await)
}

/// `DELETE /api/events/{id}/inventories/{inventory_id}` — unlink.
async fn remove_event_inventory(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path((id, inventory_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(logistics::unlink_inventory(&state, &ctx, id, inventory_id).await)
}

/// `GET /api/events/{id}/inventory-coverage` — per-ingredient demand vs. the
/// stock of the event's linked inventories.
async fn inventory_coverage(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(logistics::inventory_coverage(&state, &ctx.ctx, id).await)
}
