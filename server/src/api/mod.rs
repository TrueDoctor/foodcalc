use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde_json::json;

use crate::MyAppState;

pub mod events;
pub mod recipes;

pub fn api_router() -> Router<MyAppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/events", events::events_router())
        .nest("/recipes", recipes::recipes_router())
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "foodcalc-api",
        })),
    )
}

pub fn map_api_error(err: foodlib_new::Error) -> (StatusCode, Json<serde_json::Value>) {
    let status = match err {
        foodlib_new::Error::NotFound { .. } => StatusCode::NOT_FOUND,
        foodlib_new::Error::Unauthorized(_) | foodlib_new::Error::Forbidden(_) => {
            StatusCode::UNAUTHORIZED
        }
        foodlib_new::Error::Validation { .. } | foodlib_new::Error::Conflict { .. } => {
            StatusCode::BAD_REQUEST
        }
        foodlib_new::Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };

    (
        status,
        Json(json!({
            "error": err.to_string(),
        })),
    )
}
