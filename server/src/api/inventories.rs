use std::str::FromStr;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use foodlib_new::{auth_context::AuthCtx, inventory::Inventory, inventory::InventoryItem, inventory::InventoryItemWithName};

use crate::{api::map_api_error, MyAppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryCreateInput {
    pub name: String,
    pub group_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemAddInput {
    pub inventory_id: i32,
    pub ingredient_id: i32,
    pub amount: String,
}


pub fn inventories_router() -> Router<MyAppState> {
    Router::new()
    .route("/", get(list_available_inventories ).post(create_inventory))
    .route("/{id}", get(get_inventory))
    .route("/{id}/items", post(add_item_to_inventory))
}

async fn list_available_inventories(State(state): State<MyAppState>, ctx: AuthCtx) -> impl IntoResponse {
    // Group visibility: members see their groups' inventories, admins all.
    let inventories = state.inventories().list().await;
    match inventories {
        Ok(items) => {
            let visible: Vec<Inventory> = items
                .into_iter()
                .filter(|inventory| {
                    ctx.user.is_admin || ctx.group_ids.contains(&inventory.group_id)
                })
                .collect();
            (StatusCode::OK, Json(visible)).into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn create_inventory(
    State(state): State<MyAppState>,
    Json(payload): Json<InventoryCreateInput>,
) -> impl IntoResponse {
    let inventory = foodlib_new::entities::inventory::Inventory {
        id: 0, // TODO: id creation so that it creates a new inventory with a unique id, or let the database handle it
        name: payload.name,
        group_id: payload.group_id,
    };
    match state.inventories().create(inventory).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn get_inventory(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.inventories().get(id).await {
        Ok(item) => (StatusCode::OK, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn add_item_to_inventory(
    State(state): State<MyAppState>,
    Json(payload): Json<InventoryItemAddInput>,
) -> impl IntoResponse {

let parsed_amount: BigDecimal = match bigdecimal::BigDecimal::from_str(&payload.amount) {
        Ok(value) => value,
        Err(_) => {
            return map_api_error(foodlib_new::Error::Validation {
                message: format!("invalid amount: {}", payload.amount),
            })
            .into_response();
        }
    };

 let item = InventoryItem {
        inventory_id: payload.inventory_id,
        ingredient_id: payload.ingredient_id,
        amount: parsed_amount,
    };

    match state.inventories().add_item(item).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}