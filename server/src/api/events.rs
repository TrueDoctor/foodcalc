use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{api::map_api_error, MyAppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCreateInput {
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<String>,
    pub group_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventUpdateInput {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub budget: Option<String>,
}

pub fn events_router() -> Router<MyAppState> {
    Router::new()
        .route("", get(list_events).post(create_event))
        .route("/:id", get(get_event).put(update_event).delete(delete_event))
        .route("/:id/clone", post(clone_event))
}

async fn list_events(State(state): State<MyAppState>) -> impl IntoResponse {
    let events = state.events().list().await;
    match events {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn get_event(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let event = state.events().get(id).await;
    match event {
        Ok(item) => (StatusCode::OK, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn create_event(
    State(state): State<MyAppState>,
    Json(payload): Json<EventCreateInput>,
) -> impl IntoResponse {
    let budget = match payload.budget {
        Some(value) => {
            let parsed = bigdecimal::BigDecimal::from_str(&value).map_err(|_| {
                foodlib_new::Error::Validation {
                    message: format!("invalid budget: {value}"),
                }
            });
            match parsed {
                Ok(v) => Some(v),
                Err(err) => return map_api_error(err).into_response(),
            }
        }
        None => None,
    };

    let event = foodlib_new::entities::event::Event {
        id: 0,
        name: payload.name,
        comment: payload.comment,
        budget,
        group_id: payload.group_id,
    };

    match state.events().create(event).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn update_event(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
    Json(payload): Json<EventUpdateInput>,
) -> impl IntoResponse {
    let current = match state.events().get(id).await {
        Ok(item) => item,
        Err(err) => return map_api_error(err).into_response(),
    };

    let budget = match payload.budget {
        Some(value) => {
            let parsed = bigdecimal::BigDecimal::from_str(&value).map_err(|_| {
                foodlib_new::Error::Validation {
                    message: format!("invalid budget: {value}"),
                }
            });
            match parsed {
                Ok(v) => Some(v),
                Err(err) => return map_api_error(err).into_response(),
            }
        }
        None => current.budget,
    };

    let updated = foodlib_new::entities::event::Event {
        id: current.id,
        name: payload.name.unwrap_or(current.name),
        comment: payload.comment.or(current.comment),
        budget,
        group_id: current.group_id,
    };

    match state.events().update(updated).await {
        Ok(item) => (StatusCode::OK, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn delete_event(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.events().delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn clone_event(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let event = match state.events().get(id).await {
        Ok(event) => event,
        Err(err) => return map_api_error(err).into_response(),
    };

    match state.events().duplicate(id, event.group_id).await {
        Ok(new_id) => {
            (StatusCode::CREATED, Json(serde_json::json!({ "id": new_id }))).into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}
