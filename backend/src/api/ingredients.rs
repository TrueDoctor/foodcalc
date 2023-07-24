use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::types::BigDecimal;

use crate::MyAppState;

#[derive(Clone, Deserialize)]
pub struct Ingredient {
    name: String,
    energy: BigDecimal,
    comment: Option<String>,
}

pub async fn create(
    State(state): State<MyAppState>,
    Json(ingredient): Json<Ingredient>,
) -> impl IntoResponse {
    let id = state
        .db_connection
        .add_ingredient(ingredient.name, ingredient.energy, ingredient.comment)
        .await;
    match id {
        Ok(id) => (StatusCode::OK, Json(id)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(-1)),
    }
}

pub async fn update(
    State(state): State<MyAppState>,
    Path(ingredient_id): Path<i32>,
    Json(ingredient): Json<Ingredient>,
) -> impl IntoResponse {
    let db = state.db_connection.clone();
    let ingredient = crate::db::Ingredient {
        ingredient_id,
        name: ingredient.name,
        energy: ingredient.energy,
        comment: ingredient.comment,
    };
    db.update_ingredient(ingredient).await.unwrap();

    StatusCode::OK
}

pub async fn list(State(state): State<MyAppState>) -> impl IntoResponse {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();
    Json(ingredients)
}
