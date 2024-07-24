use std::{any::Any, borrow::BorrowMut};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use foodlib::Ingredient;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use tracing::Instrument;

use crate::ApiState;

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/", get(list))
        .route("/", put(add))
        .route("/:ingredient_id/", get(show_ingredient))
        .route("/:ingredient_id/", delete(delete_ingredient))
        .route("/:ingredient_id/", post(update_ingredient))
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct IngredientDataBody {
    name: String,
    energy: BigDecimal,
    comment: Option<String>,
}

// TODO Add Error Handling
async fn add(
    State(state): State<ApiState>,
    Json(body): Json<IngredientDataBody>,
) -> impl IntoResponse {
    match state
        .food_base
        .add_ingredient(body.name, body.energy, body.comment)
        .await
    {
        Ok(ingredient) => (StatusCode::OK, Json(ingredient)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn list(State(state): State<ApiState>) -> impl IntoResponse {
    match state.food_base.get_ingredients().await {
        Ok(ingredient_list) => (StatusCode::OK, Json(ingredient_list)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn show_ingredient(
    State(state): State<ApiState>,
    Path(ingredient_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.get_ingredient(ingredient_id).await {
        Ok(ingredient) => (StatusCode::OK, Json(ingredient)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn delete_ingredient(
    State(state): State<ApiState>,
    Path(ingredient_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.delete_ingredient(ingredient_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn update_ingredient(
    State(state): State<ApiState>,
    Path(ingredient_id): Path<i32>,
    Json(body): Json<IngredientDataBody>,
) -> impl IntoResponse {
    let ingredient = Ingredient {
        ingredient_id,
        name: body.name,
        energy: body.energy,
        comment: body.comment,
    };
    match state.food_base.update_ingredient(&ingredient).await {
        Ok(ingredient) => (StatusCode::OK, Json(ingredient)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
