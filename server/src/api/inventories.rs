//! Inventory endpoints: group-scoped list, CRUD and stock item management.
//! Handlers are thin: extraction + delegation to `crate::app::inventories`.

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::inventories::{
    InventoryCreateInput, InventoryItemAddInput, InventoryItemUpdateInput, InventoryUpdateInput,
};
use crate::app::{inventories, AppCtx};
use crate::MyAppState;
use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, put},
    Json, Router,
};
use foodlib_new::types::ids::{GroupId, IngredientId, InventoryId};
use serde::Deserialize;

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
async fn list_inventories(
    State(state): State<MyAppState>,
    actor: AppCtx,
) -> Response {
    respond(inventories::list_inventories(&state, &actor).await)
}

/// `POST /api/inventories` — create an inventory owned by one of the user's
/// groups.
async fn create_inventory(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Json(payload): Json<InventoryCreateInput>,
) -> Response {
    respond_created(inventories::create_inventory(&state, &actor, payload).await)
}

/// `GET /api/inventories/{id}` — frame data. Non-members get a plain 404.
async fn get_inventory(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<InventoryId>,
) -> Response {
    respond(inventories::get_inventory(&state, &actor, id).await)
}

/// `PUT /api/inventories/{id}` — rename / move to another owned group.
async fn update_inventory(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<InventoryId>,
    Json(payload): Json<InventoryUpdateInput>,
) -> Response {
    respond(inventories::update_inventory(&state, &actor, id, payload).await)
}

/// `DELETE /api/inventories/{id}` — deletes the inventory and its items.
async fn delete_inventory(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<InventoryId>,
) -> Response {
    respond_no_content(inventories::delete_inventory(&state, &actor, id).await)
}

/// `GET /api/inventories/{id}/items` — stocked items with ingredient names.
async fn list_items(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<InventoryId>,
) -> Response {
    respond(inventories::list_inventory_items(&state, &actor, id).await)
}

/// `POST /api/inventories/{id}/items` — stock an ingredient.
async fn add_item(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<InventoryId>,
    Json(payload): Json<InventoryItemAddInput>,
) -> Response {
    respond_created(inventories::add_inventory_item(&state, &actor, id, payload).await)
}

/// `PUT /api/inventories/{id}/items/{ingredient_id}` — adjust a stocked amount.
async fn update_item(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, ingredient_id)): Path<(InventoryId, IngredientId)>,
    Json(payload): Json<InventoryItemUpdateInput>,
) -> Response {
    respond(inventories::update_inventory_item(&state, &actor, id, ingredient_id, payload).await)
}

/// `DELETE /api/inventories/{id}/items/{ingredient_id}` — remove the stock.
async fn remove_item(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, ingredient_id)): Path<(InventoryId, IngredientId)>,
) -> Response {
    respond_no_content(inventories::remove_inventory_item(&state, &actor, id, ingredient_id).await)
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
    actor: AppCtx,
    Path(id): Path<InventoryId>,
    Json(payload): Json<OwnerTransferInput>,
) -> Response {
    let group_id = GroupId(payload.group_id);
    respond_no_content(
        inventories::transfer_inventory_ownership(&state, &actor, id, group_id).await,
    )
}
