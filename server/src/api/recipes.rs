//! Recipe endpoints: filtered list, full detail (tree), CRUD, deep clone
//! and the ingredient / step / sub-recipe editors — thin handlers over
//! `crate::app::recipes`. Permission checks, validation and serialization
//! DTOs live in the app layer.

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use foodlib_new::entities::recipe::RecipeKind;
use serde::Deserialize;
use serde_json::json;

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::recipes;
use crate::app::AppCtx;
use crate::MyAppState;

pub fn recipes_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_recipes).post(create_recipe))
        .route(
            "/{id}",
            get(get_recipe).put(update_recipe).delete(delete_recipe),
        )
        .route("/{id}/tree", get(get_recipe_tree))
        .route("/{id}/export", get(export_recipe_pdf))
        .route("/{id}/clone", post(clone_recipe))
        .route("/{id}/owner", put(transfer_owner))
        .route("/{id}/ingredients", post(add_ingredient))
        .route(
            "/{id}/ingredients", put(update_ingredient).delete(remove_ingredient),
        )
        .route("/{id}/steps", post(add_step))
        .route(
            "/{id}/steps/{step_id}",
            put(update_step).delete(remove_step),
        )
        .route("/{id}/sub-recipes", post(add_sub_recipe))
        .route(
            "/{id}/sub-recipes/{child_id}",
            put(update_sub_recipe).delete(remove_sub_recipe),
        )
}

use crate::app::recipes::RecipeUpdateInput;
// DTOs shared with the app layer.
pub use crate::app::recipes::{
    RecipeCloneInput, RecipeExportQuery, RecipeIngredientInput, RecipeListQuery, RecipeStepInput,
    SubRecipeInput, SubRecipeWeightInput,
};

#[derive(Debug, Deserialize)]
pub struct RecipeCreateInput {
    pub name: String,
    pub comment: Option<String>,
    pub kind: RecipeKind,
    pub group_id: i32,
    pub weighed_in: Option<bool>,
}

async fn list_recipes(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Query(query): Query<RecipeListQuery>,
) -> Response {
    respond(recipes::list_recipes(&state, &actor.ctx, query).await)
}

async fn create_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Json(payload): Json<RecipeCreateInput>,
) -> Response {
    respond_created(recipes::create_new_recipe(
        &state,
        &actor,
        payload.name,
        payload.comment,
        payload.kind,
        payload.group_id,
        payload.weighed_in,
    ).await)
}

async fn update_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeUpdateInput>,
) -> Response {
    respond(recipes::update_recipe(&state, &actor, id, payload).await)
}

async fn delete_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond_no_content(recipes::delete_recipe(&state, &actor, id).await)
}

async fn get_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(recipes::get_recipe(&state, &actor, id).await)
}

/// `GET /api/recipes/{id}/tree` — the recursion tree with resolved
/// ingredient weights per node (view mode visualization).
async fn get_recipe_tree(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(recipes::tree(&state, &actor, id).await)
}

/// `GET /api/recipes/{id}/export` — cooking instructions PDF, optionally
/// scaled to `servings × energy_per_serving` like an event meal.
async fn export_recipe_pdf(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Query(query): Query<RecipeExportQuery>,
) -> Response {
    match recipes::export_pdf(&state, &actor, id, query).await {
        Ok(file) => (
            [
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}.pdf\"", file.filename),
                ),
                (axum::http::header::CONTENT_TYPE, file.mime.to_string()),
            ],
            file.bytes,
        )
            .into_response(),
        Err(err) => crate::api::map_api_error(err).into_response(),
    }
}

async fn clone_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeCloneInput>,
) -> Response {
    respond_created(
        recipes::clone_recipe(&state, &actor, id, payload)
            .await
            .map(|new_id| json!({ "id": new_id })),
    )
}

// ------------------------------------------------------------ ingredients

async fn add_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeIngredientInput>,
) -> Response {
    respond_created(recipes::add_ingredient(&state, &actor, id, payload).await)
}

async fn update_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeIngredientInput>,
) -> Response {
    respond(recipes::update_ingredient(&state, &actor, id, payload).await)
}

async fn remove_ingredient(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(recipes::remove_ingredient(&state, &actor, id, ingredient_id).await)
}

// ------------------------------------------------------------------ steps

async fn add_step(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeStepInput>,
) -> Response {
    respond_created(recipes::add_step(&state, &actor, id, payload).await)
}

async fn update_step(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, step_id)): Path<(i32, i32)>,
    Json(payload): Json<RecipeStepInput>,
) -> Response {
    respond(recipes::update_step(&state, &actor, id, step_id, payload).await)
}

async fn remove_step(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, step_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(recipes::remove_step(&state, &actor, id, step_id).await)
}

// ------------------------------------------------------------- sub-recipes

async fn add_sub_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<SubRecipeInput>,
) -> Response {
    respond_created(recipes::add_sub_recipe(&state, &actor, id, payload).await)
}

async fn update_sub_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, child_id)): Path<(i32, i32)>,
    Json(payload): Json<SubRecipeWeightInput>,
) -> Response {
    respond(recipes::update_sub_recipe(&state, &actor, id, child_id, payload).await)
}

async fn remove_sub_recipe(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, child_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(recipes::remove_sub_recipe(&state, &actor, id, child_id).await)
}

// ---------------------------------------------------------------- ownership

#[derive(Debug, Deserialize)]
pub struct OwnerTransferInput {
    pub group_id: i32,
}

/// `PUT /api/recipes/{id}/owner` — explicit Besitz-Übertragung (siehe
/// `events::transfer_owner`): Besitzer oder Manager der aktuellen Gruppe;
/// Zielgruppe muss zugänglich sein.
async fn transfer_owner(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<OwnerTransferInput>,
) -> Response {
    respond_no_content(
        recipes::transfer_owner(&state, &actor, id, payload.group_id).await,
    )
}
