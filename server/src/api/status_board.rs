use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use foodlib_new::ops::status_board::CreateReorderInput;

use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

#[derive(Debug, Deserialize)]
pub struct ReorderStatusBody {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AvailabilityBody {
    pub availability: String,
    pub eta_minutes: Option<i32>,
}

pub fn status_board_router() -> Router<MyAppState> {
    Router::new()
        .route("/{id}/status-board", get(get_board))
        .route("/{id}/reorders", post(post_reorder))
        .route("/{id}/reorders/{reorder_id}/status", put(put_reorder_status))
        .route("/{id}/meals/{meal_id}/availability", put(put_availability))
}

async fn get_board(State(state): State<MyAppState>, Path(id): Path<i32>) -> Response {
    respond(state.status_board().get_board(id).await)
}

async fn post_reorder(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
    Json(payload): Json<CreateReorderInput>,
) -> Response {
    if payload.amount_label.trim().is_empty() {
        return bad_request("amount_label must not be empty".parse().unwrap());
    }
    if !matches!(payload.urgency.as_str(), "normal" | "urgent" | "critical") {
        return bad_request(format!("invalid urgency: {}", payload.urgency));
    }
    respond_created(
        state
            .status_board()
            .create_reorder(id, payload)
            .await
            .map(|reorder_id| json!({ "id": reorder_id })),
    )
}

async fn put_reorder_status(
    State(state): State<MyAppState>,
    Path((id, reorder_id)): Path<(i32, i32)>,
    Json(payload): Json<ReorderStatusBody>,
) -> Response {
    if !matches!(
        payload.status.as_str(),
        "requested" | "preparing" | "en_route" | "delivered"
    ) {
        return bad_request(format!("invalid status: {}", payload.status));
    }
    match state
        .status_board()
        .update_reorder_status(id, reorder_id, &payload.status)
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn put_availability(
    State(state): State<MyAppState>,
    Path((id, meal_id)): Path<(i32, i32)>,
    Json(payload): Json<AvailabilityBody>,
) -> Response {
    if !matches!(
        payload.availability.as_str(),
        "available" | "low" | "replenishing" | "sold_out"
    ) {
        return bad_request(format!("invalid availability: {}", payload.availability));
    }
    match state.status_board().set_meal_availability(
        id,
        meal_id,
        &payload.availability,
        payload.eta_minutes,
    ) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}