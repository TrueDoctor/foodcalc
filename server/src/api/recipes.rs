//! Recipe endpoints: filtered list, full detail (tree), CRUD, deep clone
//! and the ingredient / step / sub-recipe editors.

use std::collections::HashMap;
use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::types::PgInterval;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::recipe::{
    Recipe, RecipeIngredient, RecipeKind, RecipeMetaIngredient, RecipeStep,
};
use foodlib_new::error::Error;

use crate::{
    api::{bad_request, json_response, map_api_error, respond, respond_created},
    MyAppState,
};

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
            "/{id}/ingredients/{ingredient_id}",
            put(update_ingredient).delete(delete_ingredient),
        )
        .route("/{id}/steps", post(add_step))
        .route(
            "/{id}/steps/{step_id}",
            put(update_step).delete(delete_step),
        )
        .route("/{id}/sub-recipes", post(add_sub_recipe))
        .route(
            "/{id}/sub-recipes/{child_id}",
            put(update_sub_recipe).delete(delete_sub_recipe),
        )
}

// ------------------------------------------------------------------ list

#[derive(Debug, Deserialize)]
pub struct RecipeListQuery {
    /// Filter by kind: "meal" | "component" (absent = all).
    pub r#type: Option<String>,
    /// Case-insensitive substring search on the name.
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecipeListItemDto {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub group_id: i32,
    pub group_name: String,
    pub kind: RecipeKind,
    pub weighed_in: bool,
    /// Energy density in kJ/g — the scaling basis for meal planning.
    /// `None` for recipes without any resolvable ingredients.
    pub energy_kj_per_g: Option<f64>,
}

async fn list_recipes(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<RecipeListQuery>,
) -> Response {
    let recipes = match state.recipes().list_with_stats().await {
        Ok(recipes) => recipes,
        Err(err) => return map_api_error(err).into_response(),
    };
    let groups = match state.users().get_user_groups(ctx.user.id).await {
        Ok(groups) => groups,
        Err(err) => return map_api_error(err).into_response(),
    };

    let group_name_by_id: HashMap<i32, String> = groups
        .into_iter()
        .map(|group| (group.id, group.name))
        .collect();
    let search = query.search.unwrap_or_default().to_lowercase();
    let kind_filter: Option<RecipeKind> = match query.r#type.as_deref() {
        None | Some("") => None,
        Some("meal") => Some(RecipeKind::Meal),
        Some("component") => Some(RecipeKind::Component),
        Some(other) => return bad_request(format!("unknown recipe kind filter: {other}")),
    };

    let items: Vec<RecipeListItemDto> = recipes
        .into_iter()
        // Group visibility: members see their groups' recipes, admins see all.
        .filter(|recipe| ctx.user.is_admin || ctx.group_ids.contains(&recipe.group_id))
        .filter(|recipe| {
            kind_filter
                .as_ref()
                .map_or(true, |kind| &recipe.kind == kind)
        })
        .filter(|recipe| search.is_empty() || recipe.name.to_lowercase().contains(&search))
        .map(|recipe| {
            let weight_g = recipe.weight.to_f64().unwrap_or(0.0) * 1000.0;
            let energy_kj_per_g = (weight_g > 0.0)
                .then(|| recipe.energy.to_f64().unwrap_or(0.0) / weight_g);
            RecipeListItemDto {
                group_name: group_name_by_id
                    .get(&recipe.group_id)
                    .cloned()
                    .unwrap_or_default(),
                energy_kj_per_g,
                id: recipe.id,
                name: recipe.name,
                comment: recipe.comment,
                group_id: recipe.group_id,
                kind: recipe.kind,
                weighed_in: recipe.weighed_in,
            }
        })
        .collect();

    json_response(StatusCode::OK, items)
}

// ---------------------------------------------------------------- create

#[derive(Debug, Deserialize)]
pub struct RecipeCreateInput {
    pub name: String,
    pub comment: Option<String>,
    pub kind: RecipeKind,
    pub group_id: i32,
    pub weighed_in: Option<bool>,
}

async fn create_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<RecipeCreateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_group(payload.group_id) {
        return map_api_error(err).into_response();
    }

    let recipe = Recipe {
        id: 0,
        name: payload.name,
        comment: payload.comment,
        group_id: payload.group_id,
        weighed_in: payload.weighed_in.unwrap_or(false),
        kind: payload.kind,
    };

    respond_created(state.recipes().create(recipe).await)
}

// ------------------------------------------------------- update / delete

#[derive(Debug, Deserialize)]
pub struct RecipeUpdateInput {
    pub name: Option<String>,
    /// `null` clears the comment, absent keeps it.
    pub comment: Option<Option<String>>,
    pub kind: Option<RecipeKind>,
    pub weighed_in: Option<bool>,
}

async fn update_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let current = match state.recipes().get(id).await {
        Ok(recipe) => recipe,
        Err(err) => return map_api_error(err).into_response(),
    };

    let recipe = Recipe {
        id: current.id,
        name: payload.name.unwrap_or(current.name),
        comment: payload.comment.unwrap_or(current.comment),
        group_id: current.group_id,
        weighed_in: payload.weighed_in.unwrap_or(current.weighed_in),
        kind: payload.kind.unwrap_or(current.kind),
    };

    respond(state.recipes().update(recipe).await)
}

async fn delete_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    match state.recipes().delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ---------------------------------------------------------------- detail

#[derive(Debug, Serialize)]
pub struct RecipeStatsDto {
    /// Total resolved ingredient weight in kg.
    pub weight_kg: BigDecimal,
    /// Total resolved energy in kJ.
    pub energy_kj: BigDecimal,
    /// Energy density — the basis for scaling portions.
    pub energy_kj_per_g: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RecipeIngredientDto {
    pub ingredient_id: i32,
    pub name: Option<String>,
    pub amount: BigDecimal,
    pub unit_id: i32,
}

#[derive(Debug, Serialize)]
pub struct SubRecipeDto {
    pub recipe_id: i32,
    pub name: String,
    pub weight: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct RecipeStepDto {
    pub id: i32,
    pub order: f64,
    pub name: String,
    pub description: String,
    pub fixed_duration_seconds: i64,
    pub duration_per_kg_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct PropertyDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RecipeDetailDto {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub group_id: i32,
    pub kind: RecipeKind,
    pub weighed_in: bool,
    pub stats: Option<RecipeStatsDto>,
    pub ingredients: Vec<RecipeIngredientDto>,
    pub sub_recipes: Vec<SubRecipeDto>,
    pub steps: Vec<RecipeStepDto>,
    pub properties: Vec<PropertyDto>,
}

/// `PgInterval` (months/days/microseconds — hours and minutes are part of
/// the microseconds field) → total seconds.
fn interval_seconds(interval: &PgInterval) -> i64 {
    interval.months as i64 * 2_592_000
        + interval.days as i64 * 86_400
        + interval.microseconds.div_euclid(1_000_000)
}

/// Total seconds → `PgInterval` (all remainder goes to microseconds).
fn interval_from_seconds(seconds: i64) -> PgInterval {
    PgInterval {
        months: 0,
        days: (seconds / 86_400) as i32,
        microseconds: (seconds % 86_400) * 1_000_000,
    }
}

fn energy_kj_per_g(weight_kg: &BigDecimal, energy_kj: &BigDecimal) -> Option<f64> {
    let weight_g = weight_kg.to_f64()? * 1000.0;
    (weight_g > 0.0).then(|| energy_kj.to_f64().unwrap_or(0.0) / weight_g)
}

async fn get_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    let recipe = match state.recipes().get(id).await {
        Ok(recipe) => recipe,
        Err(err) => return map_api_error(err).into_response(),
    };
    // Group-private reads: non-members get a plain 404.
    if !ctx.can_access_group(recipe.group_id) {
        return map_api_error(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        })
        .into_response();
    }

    let stats = state.recipes().get_recipe_stats(id).await.ok();
    let ingredients = match state.recipes().get_ingredients(id).await {
        Ok(ingredients) => ingredients,
        Err(err) => return map_api_error(err).into_response(),
    };
    let sub_recipes = match state.recipes().get_meta_ingredients(id).await {
        Ok(sub_recipes) => sub_recipes,
        Err(err) => return map_api_error(err).into_response(),
    };
    let steps = match state.recipes().get_steps(id).await {
        Ok(steps) => steps,
        Err(err) => return map_api_error(err).into_response(),
    };
    let properties = state
        .properties()
        .get_recipe_properties(id)
        .await
        .map(|summary| summary.properties)
        .unwrap_or_default();

    json_response(
        StatusCode::OK,
        RecipeDetailDto {
            stats: stats.map(|stats| RecipeStatsDto {
                energy_kj_per_g: energy_kj_per_g(&stats.weight, &stats.energy),
                weight_kg: stats.weight,
                energy_kj: stats.energy,
            }),
            ingredients: ingredients
                .into_iter()
                .map(|ingredient| RecipeIngredientDto {
                    ingredient_id: ingredient.ingredient_id,
                    name: ingredient.name,
                    amount: ingredient.amount,
                    unit_id: ingredient.unit_id,
                })
                .collect(),
            sub_recipes: sub_recipes
                .into_iter()
                .map(|meta| SubRecipeDto {
                    recipe_id: meta.child_id,
                    name: meta.name.unwrap_or_default(),
                    weight: meta.weight,
                })
                .collect(),
            steps: steps
                .into_iter()
                .map(|step| RecipeStepDto {
                    id: step.id,
                    order: step.order,
                    name: step.name,
                    description: step.description,
                    fixed_duration_seconds: step.fixed_duration.to_f64().unwrap() as i64,
                    duration_per_kg_seconds: step.duration_per_kg.to_f64().unwrap() as i64,
                })
                .collect(),
            properties: properties
                .into_iter()
                .map(|property| PropertyDto {
                    id: property.id,
                    name: property.name,
                })
                .collect(),
            id: recipe.id,
            name: recipe.name,
            comment: recipe.comment,
            group_id: recipe.group_id,
            kind: recipe.kind,
            weighed_in: recipe.weighed_in,
        },
    )
}

// ------------------------------------------------------------------ tree

/// `GET /api/recipes/{id}/tree` — the recursion tree with resolved
/// ingredient weights per node (view mode visualization).
async fn get_recipe_tree(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    let recipe = match state.recipes().get(id).await {
        Ok(recipe) => recipe,
        Err(err) => return map_api_error(err).into_response(),
    };
    if !ctx.can_access_group(recipe.group_id) {
        return map_api_error(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        })
        .into_response();
    }
    respond(state.recipes().get_recipe_tree(id).await)
}

// ----------------------------------------------------------- recipe export

#[derive(Debug, Deserialize)]
pub struct RecipeExportQuery {
    /// Target portions for the PDF (default: the recipe as-is).
    pub servings: Option<u32>,
    /// Target energy per portion in kJ (default: the recipe's own density).
    pub energy_per_serving: Option<u32>,
}

/// `GET /api/recipes/{id}/export` — cooking instructions PDF, optionally
/// scaled to `servings × energy_per_serving` like an event meal.
async fn export_recipe_pdf(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Query(query): Query<RecipeExportQuery>,
) -> Response {
    let recipe = match state.recipes().get(id).await {
        Ok(recipe) => recipe,
        Err(err) => return map_api_error(err).into_response(),
    };
    if !ctx.can_access_group(recipe.group_id) {
        return map_api_error(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        })
        .into_response();
    }

    // Defaults: render the base recipe (1 portion at its own total energy).
    let stats = state.recipes().get_recipe_stats(id).await.ok();
    let servings = query.servings.unwrap_or(1) as f64;
    let energy_per_serving = match query.energy_per_serving {
        Some(energy) => energy,
        None => {
            let total = stats
                .as_ref()
                .map(|stats| stats.energy.to_f64().unwrap_or(0.0))
                .unwrap_or(0.0);
            (total / servings.max(1.0)).round() as u32
        }
    };

    let recipe_info = match state
        .export()
        .fetch_user_input_meal(id, servings, energy_per_serving, String::new())
        .await
    {
        Ok(info) => info,
        Err(err) => return map_api_error(err).into_response(),
    };
    let title = recipe_info.name.to_owned();

    #[cfg(feature = "typst")]
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let renderer = pdf_export::DocumentRenderer::new();
    #[cfg(feature = "typst")]
    let result = renderer.export_recipes(vec![recipe_info])
        .map_err(|e| foodlib_new::Error::Misc(e.to_string()));
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>, Error> =
        Err(Error::Misc("Server compiled without typst support".into()));

    match result {
        Ok(bytes) => (
            [
                (
                    axum::http::header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{title}.pdf\""),
                ),
                (
                    axum::http::header::CONTENT_TYPE,
                    "application/pdf".to_string(),
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------------------ clone

#[derive(Debug, Deserialize)]
pub struct RecipeCloneInput {
    /// Optional new name; defaults to `"<name> (Kopie)"`.
    pub name: Option<String>,
    /// Optional target group; defaults to the source recipe's group.
    pub group_id: Option<i32>,
}

async fn clone_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeCloneInput>,
) -> Response {
    let source = match state.recipes().get(id).await {
        Ok(recipe) => recipe,
        Err(err) => return map_api_error(err).into_response(),
    };
    // Read access to the source is enough — the copy is owned by the
    // target group, which requires write access.
    if !ctx.can_access_group(source.group_id) {
        return map_api_error(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        })
        .into_response();
    }
    let target_group = payload.group_id.unwrap_or(source.group_id);
    if let Err(err) = ctx.assert_can_edit_group(target_group) {
        return map_api_error(err).into_response();
    }

    match state
        .recipes()
        .duplicate(id, payload.name, target_group)
        .await
    {
        Ok(new_id) => json_response(StatusCode::CREATED, json!({ "id": new_id })),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------------ ingredients

#[derive(Debug, Deserialize)]
pub struct RecipeIngredientInput {
    pub ingredient_id: i32,
    pub amount: String,
    pub unit_id: i32,
}

fn parse_amount(value: &str) -> Result<BigDecimal, Response> {
    BigDecimal::from_str(value)
        .map_err(|_| bad_request(format!("invalid decimal amount: {value}")))
}

async fn add_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeIngredientInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let amount = match parse_amount(&payload.amount) {
        Ok(amount) => amount,
        Err(response) => return response,
    };

    let input = RecipeIngredient {
        recipe_id: id,
        ingredient_id: payload.ingredient_id,
        amount: amount.clone(),
        unit_id: payload.unit_id,
        name: None,
    };
    match state.recipes().add_ingredient(input).await {
        Ok(()) => json_response(
            StatusCode::CREATED,
            RecipeIngredientDto {
                ingredient_id: payload.ingredient_id,
                name: None,
                amount,
                unit_id: payload.unit_id,
            },
        ),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn update_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    Json(payload): Json<RecipeIngredientInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let amount = match parse_amount(&payload.amount) {
        Ok(amount) => amount,
        Err(response) => return response,
    };

    let input = RecipeIngredient {
        recipe_id: id,
        ingredient_id: payload.ingredient_id,
        amount: amount.clone(),
        unit_id: payload.unit_id,
        name: None,
    };
    match state.recipes().update_ingredient(input).await {
        Ok(updated) => json_response(
            StatusCode::OK,
            RecipeIngredientDto {
                ingredient_id: updated.ingredient_id,
                name: updated.name,
                amount,
                unit_id: updated.unit_id,
            },
        ),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn delete_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    match state.recipes().delete_ingredient(id, ingredient_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------------------ steps

#[derive(Debug, Deserialize)]
pub struct RecipeStepInput {
    pub order: f64,
    pub name: String,
    pub description: String,
    pub fixed_duration_seconds: i64,
    pub duration_per_kg_seconds: i64,
}

fn step_input_to_entity(recipe_id: i32, id: i32, payload: &RecipeStepInput) -> RecipeStep {
    RecipeStep {
        id,
        recipe_id,
        order: payload.order,
        name: payload.name.clone(),
        description: payload.description.clone(),
        fixed_duration: payload.fixed_duration_seconds.to_f64().unwrap(),
        duration_per_kg: payload.duration_per_kg_seconds.to_f64().unwrap(),
    }
}

fn step_to_dto(step: RecipeStep) -> RecipeStepDto {
    RecipeStepDto {
        id: step.id,
        order: step.order,
        name: step.name,
        description: step.description,
        fixed_duration_seconds: step.fixed_duration.to_i64().unwrap(),
        duration_per_kg_seconds: step.duration_per_kg.to_i64().unwrap(),
    }
}

async fn add_step(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<RecipeStepInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let step = step_input_to_entity(id, 0, &payload);
    match state.recipes().add_step(step).await {
        Ok(record) => json_response(StatusCode::CREATED, step_to_dto(record)),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn update_step(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, step_id)): Path<(i32, i32)>,
    Json(payload): Json<RecipeStepInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let step = step_input_to_entity(id, step_id, &payload);
    match state.recipes().update_step(step).await {
        Ok(record) => json_response(StatusCode::OK, step_to_dto(record)),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn delete_step(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, step_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    match state.recipes().delete_step(id, step_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------------ sub-recipes

#[derive(Debug, Deserialize)]
pub struct SubRecipeInput {
    pub child_recipe_id: i32,
    /// Weight of the child recipe used by the parent, in kg.
    pub weight: String,
}

#[derive(Debug, Deserialize)]
pub struct SubRecipeWeightInput {
    pub weight: String,
}

async fn add_sub_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<SubRecipeInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let weight = match parse_amount(&payload.weight) {
        Ok(weight) => weight,
        Err(response) => return response,
    };

    let meta = RecipeMetaIngredient {
        parent_id: id,
        child_id: payload.child_recipe_id,
        weight: weight.clone(),
        name: None,
    };
    if let Err(err) = state.recipes().add_meta_ingredient(meta).await {
        return map_api_error(err).into_response();
    }
    // Respond with the child name for instant UI feedback.
    let name = state
        .recipes()
        .get(payload.child_recipe_id)
        .await
        .map(|recipe| recipe.name)
        .unwrap_or_default();
    json_response(
        StatusCode::CREATED,
        SubRecipeDto {
            recipe_id: payload.child_recipe_id,
            name,
            weight,
        },
    )
}

async fn update_sub_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, child_id)): Path<(i32, i32)>,
    Json(payload): Json<SubRecipeWeightInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    let weight = match parse_amount(&payload.weight) {
        Ok(weight) => weight,
        Err(response) => return response,
    };

    let meta = RecipeMetaIngredient {
        parent_id: id,
        child_id,
        weight: weight.clone(),
        name: None,
    };
    match state.recipes().update_meta_ingredient(meta).await {
        Ok(_) => {
            let name = state
                .recipes()
                .get(child_id)
                .await
                .map(|recipe| recipe.name)
                .unwrap_or_default();
            json_response(
                StatusCode::OK,
                SubRecipeDto {
                    recipe_id: child_id,
                    name,
                    weight,
                },
            )
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn delete_sub_recipe(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, child_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_recipe(id).await {
        return map_api_error(err).into_response();
    }
    match state.recipes().delete_meta_ingredient(id, child_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
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
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<OwnerTransferInput>,
) -> Response {
    let current = match state.recipes().get(id).await {
        Ok(recipe) => recipe,
        Err(err) => return map_api_error(err).into_response(),
    };
    if !ctx.can_edit_group(current.group_id) {
        return map_api_error(Error::Forbidden(
            "transferring ownership requires edit rights on the current owner group".into(),
        ))
        .into_response();
    }
    if !ctx.can_access_group(payload.group_id) {
        return map_api_error(Error::Forbidden(
            "you can only transfer into groups you belong to".into(),
        ))
        .into_response();
    }
    match state.recipes().set_owner_group(id, payload.group_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
