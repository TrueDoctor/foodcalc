//! Logistics endpoints beyond shopping tours: food prep read access and
//! event-inventory links with per-ingredient coverage.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use foodlib_new::auth_context::AuthCtx;

use crate::api::{map_api_error, respond, MyAppState};

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
async fn list_preps(State(state): State<MyAppState>, ctx: AuthCtx, Path(id): Path<i32>) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.events().list_food_preps(id).await)
}

/// `GET /api/events/{id}/inventories` — the event's linked inventories.
async fn list_event_inventories(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.events().list_event_inventories(id).await)
}

/// `POST /api/events/{id}/inventories/{inventory_id}` — link an inventory to
/// the event. Requires event write access and read access to the inventory.
async fn add_event_inventory(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, inventory_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    match state.inventories().get(inventory_id).await {
        Ok(inventory) => {
            if !ctx.can_access_group(inventory.group_id) {
                return map_api_error(foodlib_new::Error::NotFound {
                    entity: "Inventory",
                    id: inventory_id.to_string(),
                })
                .into_response();
            }
        }
        Err(err) => return map_api_error(err).into_response(),
    }
    match state.events().add_inventory(id, inventory_id).await {
        Ok(link) => (
            StatusCode::CREATED,
            Json(link),
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/events/{id}/inventories/{inventory_id}` — unlink.
async fn remove_event_inventory(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, inventory_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    match state.events().remove_inventory(id, inventory_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/events/{id}/inventory-coverage` — per-ingredient demand vs. the
/// stock of the event's linked inventories.
async fn inventory_coverage(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.events().get_event_inventory_coverage(id).await)
}
