//! Status board endpoints — thin handlers over `crate::app::status_board`.
//! The status/urgency/availability validation and RBAC checks live in the app layer.

use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::status_board::{self, SetAvailabilityInput};
use crate::app::AppCtx;
use crate::MyAppState;
use foodlib_new::entities::types::ids::{EventId, MealId, ReorderId};
use foodlib_new::ops::status_board::CreateReorderInput;

#[derive(Debug, Deserialize)]
pub struct ReorderStatusBody {
    pub status: String,
}

pub fn status_board_router() -> Router<MyAppState> {
    Router::new()
        .route("/{id}/status-board", get(get_board))
        .route("/{id}/reorders", post(post_reorder))
        .route("/{id}/reorders/{reorder_id}/status", put(put_reorder_status))
        .route("/{id}/meals/{meal_id}/availability", put(put_availability))
}

async fn get_board(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<EventId>,
) -> Response {
    respond(status_board::get_board(&state, &actor, id).await)
}

async fn post_reorder(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<EventId>,
    Json(payload): Json<CreateReorderInput>,
) -> Response {
    respond_created(
        status_board::create_reorder(&state, &actor, id, payload)
            .await
            .map(|reorder_id| json!({ "id": reorder_id })),
    )
}

async fn put_reorder_status(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, reorder_id)): Path<(EventId, ReorderId)>,
    Json(payload): Json<ReorderStatusBody>,
) -> Response {
    respond_no_content(
        status_board::update_reorder_status(&state, &actor, id, reorder_id, &payload.status).await,
    )
}

async fn put_availability(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, meal_id)): Path<(EventId, MealId)>,
    Json(payload): Json<SetAvailabilityInput>,
) -> Response {
    respond_no_content(
        status_board::set_meal_availability(&state, &actor, id, meal_id, payload).await,
    )
}