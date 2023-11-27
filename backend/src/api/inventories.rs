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
pub struct Inventory {
    name: String,
    energy: BigDecimal,
    comment: Option<String>,
}

pub async fn create(
    State(state): State<MyAppState>,
    Json(inventory): Json<Inventory>,
) -> impl IntoResponse {
    let id = state
        .db_connection
        .add_inventory(inventory.name)
        .await;
    match id {
        Ok(id) => (StatusCode::OK, Json(id)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(-1)),
    }
}

pub async fn update(
    State(state): State<MyAppState>,
    Path(inventory_id): Path<i32>,
    Json(inventory): Json<Inventory>,
) -> impl IntoResponse {
    let db = state.db_connection.clone();
    let inventory = foodlib::Inventory {
        inventory_id,
        name: inventory.name,
    };
    db.update_inventory(inventory).await.unwrap();

    StatusCode::OK
}

pub async fn list(State(state): State<MyAppState>) -> impl IntoResponse {
    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();
    Json(inventories)
}
