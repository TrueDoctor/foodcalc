use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use foodlib::Place;
use serde::{Deserialize, Serialize};

use crate::ApiState;

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/", get(list_places))
        .route("/", put(add_places))
        .route("/:place_id", get(get_place))
        .route("/:place_id", post(update_place))
}

async fn list_places(State(state): State<ApiState>) -> impl IntoResponse {
    match state.food_base.get_places().await {
        Ok(place_list) => (StatusCode::OK, Json(place_list)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn add_places() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn get_place(State(state): State<ApiState>, Path(place_id): Path<i32>) -> impl IntoResponse {
    match state.food_base.get_place(place_id).await {
        Ok(place) => (StatusCode::OK, Json(place)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND).into_response(),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct PlaceDataBody {
    name: String,
    comment: Option<String>,
}

async fn update_place(
    State(state): State<ApiState>,
    Path(place_id): Path<i32>,
    Json(body): Json<PlaceDataBody>,
) -> impl IntoResponse {
    let place = Place {
        place_id,
        name: body.name,
        comment: body.comment,
    };
    match state.food_base.update_place(&place).await {
        Ok(place) => (StatusCode::OK, Json(place)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}
