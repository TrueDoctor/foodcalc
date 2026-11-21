//! Ingredient endpoints: filtered list, composed detail (sources, unit
//! weights, properties), CRUD and source/weight/property management.

use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{delete, get, put},
    Json, Router,
};
use serde::Deserialize;

use crate::app::ingredients;
use crate::app::ingredients::{IngredientCreateInput, IngredientPropertyInput, IngredientSourceInput, IngredientUpdateInput, IngredientWeightInput, IngredientWeightUpdateInput};
use crate::{
    api::{respond, respond_created, respond_no_content},
    app::AppCtx,
    MyAppState,
};

pub fn ingredients_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_ingredients).post(create_ingredient))
        .route(
            "/{id}",
            get(get_ingredient)
                .put(update_ingredient)
                .delete(delete_ingredient),
        )
        .route("/{id}/owner", put(transfer_owner))
        .route("/{id}/usages", get(list_usages))
        .route("/{id}/sources", get(list_sources).post(add_source))
        .route(
            "/{id}/sources/{source_id}",
            put(update_source).delete(delete_source),
        )
        .route("/{id}/weights", get(list_weights).post(add_weight))
        .route(
            "/{id}/weights/{unit_id}",
            put(update_weight).delete(delete_weight),
        )
        .route(
            "/{id}/properties",
            get(get_ingredient_properties).post(add_property),
        )
        .route(
            "/{id}/properties/{property_id}",
            delete(delete_property),
        )
}

// ------------------------------------------------------------------ list

// List/search DTOs live in the application layer.
pub use crate::app::ingredients::IngredientListQuery;

async fn list_ingredients(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Query(query): Query<IngredientListQuery>,
) -> Response {
    respond(ingredients::list_ingredients(&state, &ctx, query).await)
}

// ---------------------------------------------------------------- create



async fn create_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Json(payload): Json<IngredientCreateInput>,
) -> Response {
    respond_created(ingredients::create_ingredient(&state, &actor, payload).await)
}

// ---------------------------------------------------------------- detail



async fn get_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(ingredients::get_ingredient(&state, &actor, id).await)
}

// ------------------------------------------------------- update / delete



async fn update_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientUpdateInput>,
) -> Response {
    respond(ingredients::update_ingredient(&state, &actor, id, &payload).await)
}

async fn delete_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond_no_content(ingredients::delete_ingredient(&state, &actor, id).await)
}

// ---------------------------------------------------------------- sources

async fn list_sources(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>
) -> Response {
    respond(ingredients::list_sources(&state, &actor, id).await)
}

async fn add_source(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientSourceInput>,
) -> Response {
   respond_created(ingredients::add_source(&state, &actor, id, payload).await)
}

async fn update_source(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, source_id)): Path<(i32, i32)>,
    Json(payload): Json<IngredientSourceInput>,
) -> Response {
   respond(ingredients::update_source(&state, &actor, id, source_id, payload).await)
}

async fn delete_source(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, source_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(ingredients::delete_source(&state, &actor, id, source_id).await)
}



async fn list_weights(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>) -> Response {
    respond(ingredients::list_weights(&state, &actor, id).await)
}

async fn add_weight(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientWeightInput>,
) -> Response {
    respond_created(ingredients::add_weight(&state, &actor, id, payload).await)
}

async fn update_weight(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, unit_id)): Path<(i32, i32)>,
    Json(payload): Json<IngredientWeightUpdateInput>,
) -> Response {
    respond(ingredients::update_weight(&state, &actor, id, unit_id, payload).await)
}

async fn delete_weight(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, unit_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(ingredients::delete_weight(&state, &actor, id, unit_id).await)
}

// ------------------------------------------------------------- properties

async fn get_ingredient_properties(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(ingredients::list_ingredient_properties(&state, &actor, id).await)
}

async fn add_property(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientPropertyInput>,
) -> Response {
    respond_created(ingredients::add_property(&state, &actor, id, payload).await)
}

async fn delete_property(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, property_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(ingredients::delete_property(&state, &actor, id, property_id).await)
}

// --------------------------------------------------------------- ownership

#[derive(Debug, Deserialize)]
pub struct OwnerTransferInput {
    pub group_id: i32,
}

/// `PUT /api/ingredients/{id}/owner` — explicit Besitz-Übertragung (siehe
/// `events::transfer_owner`).
async fn transfer_owner(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<OwnerTransferInput>,
) -> Response {
    respond_no_content(
        ingredients::transfer_owner(&state, &ctx, id, payload.group_id).await,
    )
}

// ----------------------------------------------------------------- usages

/// Rezepte, die diese Zutat verwenden — nur Ressourcen sichtbarer Gruppen
/// (kein Inline-Bypass). Grundlage für „Verwendungen“ in der Zutaten-Ansicht
/// und die Lösch-Warnung.
async fn list_usages(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(ingredients::usages(&state, &ctx.ctx, id).await)
}
