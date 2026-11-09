//! Ingredient endpoints: filtered list, composed detail (sources, unit
//! weights, properties), CRUD and source/weight/property management.

use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
    Json, Router,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::ingredient::{
    Ingredient, IngredientSource, IngredientWeight, IngredientWithSource,
};
use foodlib_new::entities::property::Property;
use foodlib_new::error::Error;

use crate::{
    api::{bad_request, json_response, map_api_error, respond, respond_created},
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

#[derive(Debug, Deserialize)]
pub struct IngredientListQuery {
    /// Case-insensitive substring search on the name.
    pub search: Option<String>,
}

async fn list_ingredients(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<IngredientListQuery>,
) -> Response {
    let ingredients = match state.ingredients().list_with_sources().await {
        Ok(ingredients) => ingredients,
        Err(err) => return map_api_error(err).into_response(),
    };

    let search = query.search.unwrap_or_default().to_lowercase();
    let items: Vec<&IngredientWithSource> = ingredients
        .iter()
        .filter(|ingredient| {
            ctx.user.is_admin || ctx.group_ids.contains(&ingredient.group_id)
        })
        .filter(|ingredient| {
            search.is_empty() || ingredient.name.to_lowercase().contains(&search)
        })
        .collect();

    json_response(StatusCode::OK, items)
}

// ---------------------------------------------------------------- create

#[derive(Debug, Deserialize)]
pub struct IngredientCreateInput {
    pub name: String,
    /// Energy density in kJ/g.
    pub energy: String,
    pub comment: Option<String>,
    pub group_id: i32,
}

fn parse_energy(value: &str) -> Result<BigDecimal, Response> {
    BigDecimal::from_str(value)
        .map_err(|_| bad_request(format!("invalid energy value: {value}")))
}

async fn create_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<IngredientCreateInput>,
) -> Response {
    if let Err(err) = ctx.assert_group(payload.group_id) {
        return map_api_error(err).into_response();
    }
    let energy = match parse_energy(&payload.energy) {
        Ok(energy) => energy,
        Err(response) => return response,
    };

    let ingredient = Ingredient {
        id: 0,
        name: payload.name,
        energy,
        comment: payload.comment,
        group_id: payload.group_id,
    };

    respond_created(state.ingredients().create(ingredient).await)
}

// ---------------------------------------------------------------- detail

#[derive(Debug, Serialize)]
pub struct IngredientDetailDto {
    pub ingredient: Ingredient,
    pub sources: Vec<IngredientSource>,
    /// Per-unit gram weights for amount conversions.
    pub weights: Vec<IngredientWeight>,
    /// Allergen / dietary properties aggregated for this ingredient.
    pub properties: Vec<Property>,
}

async fn get_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    let ingredient = match state.ingredients().get(id).await {
        Ok(ingredient) => ingredient,
        Err(err) => return map_api_error(err).into_response(),
    };
    // Group-private reads: non-members get a plain 404.
    if !ctx.can_access_group(ingredient.group_id) {
        return map_api_error(Error::NotFound {
            entity: "Ingredient",
            id: id.to_string(),
        })
        .into_response();
    }

    let sources = state
        .ingredients()
        .get_sources(id)
        .await
        .unwrap_or_default();
    let weights = state
        .ingredients()
        .get_weights(id)
        .await
        .unwrap_or_default();
    let properties = state
        .properties()
        .get_ingredient_properties(id)
        .await
        .map(|summary| summary.properties)
        .unwrap_or_default();

    json_response(
        StatusCode::OK,
        IngredientDetailDto {
            ingredient,
            sources,
            weights,
            properties,
        },
    )
}

// ------------------------------------------------------- update / delete

#[derive(Debug, Deserialize)]
pub struct IngredientUpdateInput {
    pub name: Option<String>,
    pub energy: Option<String>,
    /// `null` clears the comment, absent keeps it.
    pub comment: Option<Option<String>>,
    pub group_id: Option<i32>,
}

async fn update_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    let current = match state.ingredients().get(id).await {
        Ok(ingredient) => ingredient,
        Err(err) => return map_api_error(err).into_response(),
    };
    if let Some(group_id) = payload.group_id {
        if let Err(err) = ctx.assert_group(group_id) {
            return map_api_error(err).into_response();
        }
    }

    let energy = match &payload.energy {
        Some(value) => match parse_energy(value) {
            Ok(energy) => energy,
            Err(response) => return response,
        },
        None => current.energy,
    };

    let ingredient = Ingredient {
        id: current.id,
        name: payload.name.unwrap_or(current.name),
        energy,
        comment: payload.comment.unwrap_or(current.comment),
        group_id: payload.group_id.unwrap_or(current.group_id),
    };

    respond(state.ingredients().update(ingredient).await)
}

async fn delete_ingredient(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    match state.ingredients().delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ---------------------------------------------------------------- sources

#[derive(Debug, Deserialize)]
pub struct IngredientSourceInput {
    pub store_id: i32,
    pub package_size: String,
    pub unit_id: i32,
    pub price: String,
    pub url: Option<String>,
    pub comment: Option<String>,
}

async fn list_sources(State(state): State<MyAppState>, Path(id): Path<i32>) -> Response {
    respond(state.ingredients().get_sources(id).await)
}

async fn add_source(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientSourceInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    let package_size = match parse_amount(&payload.package_size) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let price = match parse_amount(&payload.price) {
        Ok(value) => value,
        Err(response) => return response,
    };

    let source = IngredientSource {
        id: 0,
        ingredient_id: id,
        store_id: payload.store_id,
        package_size,
        unit_id: payload.unit_id,
        price,
        url: payload.url,
        comment: payload.comment,
    };
    respond_created(state.ingredients().add_source(source).await)
}

async fn update_source(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, source_id)): Path<(i32, i32)>,
    Json(payload): Json<IngredientSourceInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    let package_size = match parse_amount(&payload.package_size) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let price = match parse_amount(&payload.price) {
        Ok(value) => value,
        Err(response) => return response,
    };

    let source = IngredientSource {
        id: source_id,
        ingredient_id: id,
        store_id: payload.store_id,
        package_size,
        unit_id: payload.unit_id,
        price,
        url: payload.url,
        comment: payload.comment,
    };
    respond(state.ingredients().update_source(source).await)
}

async fn delete_source(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, source_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    match state.ingredients().delete_source(source_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ---------------------------------------------------------------- weights

/// Parses a decimal amount ("1.5") into a `BigDecimal`.
fn parse_amount(value: &str) -> Result<BigDecimal, Response> {
    BigDecimal::from_str(value)
        .map_err(|_| bad_request(format!("invalid decimal amount: {value}")))
}

#[derive(Debug, Deserialize)]
pub struct IngredientWeightInput {
    pub unit_id: i32,
    /// Weight in **kg** per unit of `unit_id` (e.g. 1 piece = 0.12 kg).
    pub weight: String,
}

#[derive(Debug, Deserialize)]
pub struct IngredientWeightUpdateInput {
    pub weight: String,
}

async fn list_weights(State(state): State<MyAppState>, Path(id): Path<i32>) -> Response {
    respond(state.ingredients().get_weights(id).await)
}

async fn add_weight(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientWeightInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    let weight = match parse_amount(&payload.weight) {
        Ok(weight) => weight,
        Err(response) => return response,
    };

    let input = IngredientWeight {
        ingredient_id: id,
        unit_id: payload.unit_id,
        weight: weight.clone(),
    };
    respond_created(state.ingredients().add_weight(input).await)
}

async fn update_weight(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, unit_id)): Path<(i32, i32)>,
    Json(payload): Json<IngredientWeightUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    let weight = match parse_amount(&payload.weight) {
        Ok(weight) => weight,
        Err(response) => return response,
    };

    let input = IngredientWeight {
        ingredient_id: id,
        unit_id,
        weight: weight.clone(),
    };
    respond(state.ingredients().update_weight(input).await)
}

async fn delete_weight(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, unit_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    match state.ingredients().delete_weight(id, unit_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------------- properties

#[derive(Debug, Deserialize)]
pub struct IngredientPropertyInput {
    pub property_id: i32,
}

async fn get_ingredient_properties(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
) -> Response {
    respond(state.properties().get_ingredient_properties(id).await)
}

async fn add_property(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<IngredientPropertyInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    match state
        .properties()
        .assign_to_ingredient(id, payload.property_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn delete_property(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, property_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_ingredient(id).await {
        return map_api_error(err).into_response();
    }
    match state
        .properties()
        .remove_from_ingredient(id, property_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
