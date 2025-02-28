use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use foodlib::{Recipe, RecipeIngredient, RecipeMetaIngredient, RecipeStep, Unit};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgInterval, types::BigDecimal};

use crate::ApiState;

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/", get(list_reciepes))
        .route("/", put(add_reciepe))
        .route("/{recipe_id}/", get(get_recipe))
        .route("/{recipe_id}/", post(update_reciepe))
        .route("/{recipe_id}/", delete(delete_reciepe))
        .route("/{recipe_id}/steps/", get(get_steps))
        .route("/{recipe_id}/steps/", post(update_steps))
        .route("/{recipe_id}/subrecipes/", get(get_subrecipes))
        .route("/{recipe_id}/subrecipes/", post(update_subrecipes))
        .route("/{recipe_id}/ingredients/", get(get_ingredients))
        .route("/{recipe_id}/ingredients/", post(update_ingredients))
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct RecipeBody {
    id: Option<i32>,
    name: String,
    comment: Option<String>,
}

fn recipe_to_body(recipe: Recipe) -> RecipeBody {
    RecipeBody {
        id: Some(recipe.recipe_id),
        name: recipe.name,
        comment: recipe.comment,
    }
}

pub(crate) fn serialize_interval<S>(interval: &PgInterval, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration = interval.microseconds / 1000_0000;
    serializer.serialize_str(&duration.to_string())
}

pub(crate) fn deserialize_interval<'de, D>(deserializer: D) -> Result<PgInterval, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let microseconds: i64 = s
        .parse()
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse interval: {}", e)))?;
    let interval = PgInterval {
        microseconds: microseconds * 1000_1000,
        days: 0,
        months: 0,
    };
    Ok(interval)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RecipeStepBody {
    name: String,
    description: String,
    #[serde(
        serialize_with = "serialize_interval",
        deserialize_with = "deserialize_interval"
    )]
    pub duration_fixed: PgInterval,
    #[serde(
        serialize_with = "serialize_interval",
        deserialize_with = "deserialize_interval"
    )]
    pub duration_per_kg: PgInterval,
}
fn step_to_body(step: RecipeStep) -> RecipeStepBody {
    RecipeStepBody {
        name: step.step_name,
        description: step.step_description,
        duration_fixed: step.fixed_duration,
        duration_per_kg: step.duration_per_kg,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RecipeIngredientBody {
    id: i32,
    name: String,
    amount: BigDecimal,
    unit: Unit,
}
fn ingredient_to_body(ingredient: RecipeIngredient) -> Option<RecipeIngredientBody> {
    match ingredient.ingredient {
        RecipeMetaIngredient::Ingredient(ingredient_data) => Some(RecipeIngredientBody {
            id: ingredient_data.ingredient_id,
            name: ingredient_data.name,
            amount: ingredient.amount,
            unit: ingredient.unit,
        }),
        _ => None,
    }
}

fn subrecipe_to_body(ingredient: RecipeIngredient) -> Option<RecipeIngredientBody> {
    match ingredient.ingredient {
        RecipeMetaIngredient::MetaRecipe(ingredient_data) => Some(RecipeIngredientBody {
            id: ingredient_data.recipe_id,
            name: ingredient_data.name,
            amount: ingredient.amount,
            unit: ingredient.unit,
        }),
        _ => None,
    }
}

async fn list_reciepes(State(state): State<ApiState>) -> impl IntoResponse {
    match state.food_base.get_recipes().await {
        Ok(recipes) => (
            StatusCode::OK,
            Json(recipes.into_iter().map(recipe_to_body).collect::<Vec<_>>()),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn add_reciepe(
    State(state): State<ApiState>,
    Json(body): Json<RecipeBody>,
) -> impl IntoResponse {
    match state.food_base.add_recipe(&body.name, &body.comment).await {
        Ok(recipe) => (StatusCode::OK, Json(recipe_to_body(recipe))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    }
}

async fn get_recipe(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.get_recipe(recipe_id).await {
        Ok(recipe) => (StatusCode::OK, Json(recipe_to_body(recipe))).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn update_reciepe(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
    Json(body): Json<RecipeBody>,
) -> impl IntoResponse {
    match state
        .food_base
        .update_recipe(&Recipe {
            recipe_id,
            name: body.name,
            comment: body.comment,
            owner_id: -1,
        })
        .await
    {
        Ok(recipe) => (StatusCode::OK, Json(recipe_to_body(recipe))).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn delete_reciepe(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.delete_recipe(recipe_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn update_steps(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
    Json(body): Json<Vec<RecipeStepBody>>,
) -> impl IntoResponse {
    match state
        .food_base
        .update_recipe_steps(
            recipe_id,
            body.into_iter().enumerate().map(|(i, step)| {
                return RecipeStep {
                    step_id: 0,
                    step_order: i as f64,
                    step_name: step.name,
                    step_description: step.description,
                    fixed_duration: step.duration_fixed,
                    duration_per_kg: step.duration_per_kg,
                    recipe_id: recipe_id,
                };
            }),
        )
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_steps(State(state): State<ApiState>, Path(recipe_id): Path<i32>) -> impl IntoResponse {
    match state.food_base.get_recipe_steps(recipe_id).await {
        Ok(steps) => (
            StatusCode::OK,
            Json(
                steps
                    .into_iter()
                    .map(step_to_body)
                    .collect::<Vec<RecipeStepBody>>(),
            ),
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_ingredients(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.get_recipe_ingredients(recipe_id).await {
        Ok(ingredients) => (
            StatusCode::OK,
            Json(
                ingredients
                    .into_iter()
                    .flat_map(ingredient_to_body)
                    .collect::<Vec<RecipeIngredientBody>>(),
            ),
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn update_ingredients(
    State(_state): State<ApiState>,
    Path(_recipe_id): Path<i32>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn get_subrecipes(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.get_recipe_meta_ingredients(recipe_id).await {
        Ok(ingredients) => (
            StatusCode::OK,
            Json(
                ingredients
                    .into_iter()
                    .flat_map(subrecipe_to_body)
                    .collect::<Vec<RecipeIngredientBody>>(),
            ),
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn update_subrecipes(
    State(_state): State<ApiState>,
    Path(_recipe_id): Path<i32>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}
