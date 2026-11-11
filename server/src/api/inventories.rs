//! Inventory endpoints: group-scoped list, CRUD and stock item management.

use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::inventory::{Inventory, InventoryItem};
use foodlib_new::error::Error;

use crate::api::{bad_request, map_api_error, respond, respond_created, MyAppState};

pub fn inventories_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_inventories).post(create_inventory))
        .route(
            "/{id}",
            get(get_inventory).put(update_inventory).delete(delete_inventory),
        )
        .route("/{id}/owner", put(transfer_owner))
        .route("/{id}/items", get(list_items).post(add_item))
        .route(
            "/{id}/items/{ingredient_id}",
            put(update_item).delete(remove_item),
        )
}

/// `GET /api/inventories` — inventories of the user's groups (admins: all).
async fn list_inventories(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    let inventories = match state.inventories().list().await {
        Ok(items) => items,
        Err(err) => return map_api_error(err).into_response(),
    };
    let visible: Vec<&Inventory> = inventories
        .iter()
        .filter(|inventory| ctx.can_access_group(inventory.group_id))
        .collect();
    respond(Ok(visible))
}

#[derive(Debug, Deserialize)]
pub struct InventoryCreateInput {
    pub name: String,
    pub group_id: i32,
}

/// `POST /api/inventories` — create an inventory owned by one of the user's
/// groups.
async fn create_inventory(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<InventoryCreateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_group(payload.group_id) {
        return map_api_error(err).into_response();
    }
    let inventory = Inventory {
        id: 0,
        name: payload.name,
        group_id: payload.group_id,
    };
    respond_created(state.inventories().create(inventory).await)
}

/// Loads the inventory and enforces group-private reads (404 otherwise).
async fn load_inventory(
    state: &MyAppState,
    ctx: &AuthCtx,
    id: i32,
) -> Result<Inventory, Response> {
    let inventory = state
        .inventories()
        .get(id)
        .await
        .map_err(|err| map_api_error(err).into_response())?;
    if !ctx.can_access_group(inventory.group_id) {
        return Err(map_api_error(Error::NotFound {
            entity: "Inventory",
            id: id.to_string(),
        })
        .into_response());
    }
    Ok(inventory)
}

/// `GET /api/inventories/{id}` — frame data.
async fn get_inventory(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    match load_inventory(&state, &ctx, id).await {
        Ok(inventory) => (StatusCode::OK, Json(inventory)).into_response(),
        Err(response) => response,
    }
}

#[derive(Debug, Deserialize)]
pub struct InventoryUpdateInput {
    pub name: Option<String>,
}

/// `PUT /api/inventories/{id}` — rename / move to another owned group.
async fn update_inventory(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<InventoryUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_inventory(id).await {
        return map_api_error(err).into_response();
    }
    let current = match state.inventories().get(id).await {
        Ok(inventory) => inventory,
        Err(err) => return map_api_error(err).into_response(),
    };
    let name = match payload.name {
        Some(name) if name.trim().is_empty() => {
            return bad_request("inventory name must not be empty".to_string())
        }
        Some(name) => name,
        None => current.name,
    };
    let inventory = Inventory {
        id: current.id,
        name,
        group_id: current.group_id,
    };
    respond(state.inventories().update(inventory).await)
}

/// `DELETE /api/inventories/{id}` — deletes the inventory and its items.
async fn delete_inventory(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_inventory(id).await {
        return map_api_error(err).into_response();
    }
    match state.inventories().delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/inventories/{id}/items` — stocked items with ingredient names.
async fn list_items(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_inventory(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.inventories().get_items(id).await)
}

#[derive(Debug, Deserialize)]
pub struct InventoryItemAddInput {
    pub ingredient_id: i32,
    /// Decimal string, amount in kg.
    pub amount: String,
}

/// Parses a decimal amount ("1.5") into a `BigDecimal`.
fn parse_amount(value: &str) -> Result<BigDecimal, Response> {
    BigDecimal::from_str(value)
        .map_err(|_| bad_request(format!("invalid decimal amount: {value}")))
}

/// `POST /api/inventories/{id}/items` — stock an ingredient.
async fn add_item(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<InventoryItemAddInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_inventory(id).await {
        return map_api_error(err).into_response();
    }
    let amount = match parse_amount(&payload.amount) {
        Ok(amount) => amount,
        Err(response) => return response,
    };
    let item = InventoryItem {
        inventory_id: id,
        ingredient_id: payload.ingredient_id,
        amount,
    };
    respond_created(state.inventories().add_item(item).await)
}

#[derive(Debug, Deserialize)]
pub struct InventoryItemUpdateInput {
    /// Decimal string, amount in kg.
    pub amount: String,
}

/// `PUT /api/inventories/{id}/items/{ingredient_id}` — adjust a stocked amount.
async fn update_item(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    Json(payload): Json<InventoryItemUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_inventory(id).await {
        return map_api_error(err).into_response();
    }
    let amount = match parse_amount(&payload.amount) {
        Ok(amount) => amount,
        Err(response) => return response,
    };
    let item = InventoryItem {
        inventory_id: id,
        ingredient_id,
        amount,
    };
    respond(state.inventories().update_item(item).await)
}

/// `DELETE /api/inventories/{id}/items/{ingredient_id}` — remove the stock.
async fn remove_item(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_inventory(id).await {
        return map_api_error(err).into_response();
    }
    match state.inventories().delete_item(id, ingredient_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// --------------------------------------------------------------- ownership

#[derive(Debug, Deserialize)]
pub struct OwnerTransferInput {
    pub group_id: i32,
}

/// `PUT /api/inventories/{id}/owner` — explicit Besitz-Übertragung (siehe
/// `events::transfer_owner`).
async fn transfer_owner(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<OwnerTransferInput>,
) -> Response {
    let current = match state.inventories().get(id).await {
        Ok(inventory) => inventory,
        Err(err) => return map_api_error(err).into_response(),
    };
    if !ctx.can_edit_group(current.group_id) {
        return map_api_error(foodlib_new::Error::Forbidden(
            "transferring ownership requires edit rights on the current owner group".into(),
        ))
        .into_response();
    }
    if !ctx.can_access_group(payload.group_id) {
        return map_api_error(foodlib_new::Error::Forbidden(
            "you can only transfer into groups you belong to".into(),
        ))
        .into_response();
    }
    match state
        .inventories()
        .set_owner_group(id, payload.group_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
