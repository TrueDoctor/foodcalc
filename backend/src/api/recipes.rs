use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;

use crate::MyAppState;
use foodlib::{RecipeIngredient, RecipeStep};

#[derive(Clone, Serialize, Deserialize)]
pub struct Recipe {
    name: String,
    comment: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SerializableRecipeStep {
    step_id: i32,
    step_order: f64,
    step_name: String,
    step_description: String,
    recipe_id: i32,
    fixed_duration: Duration,
    duration_per_kg: Duration,
}

pub async fn create(
    State(state): State<MyAppState>,
    Json(recipe): Json<Recipe>,
) -> impl axum::response::IntoResponse {
    let recipe = foodlib::Recipe {
        recipe_id: 0,
        name: recipe.name.to_owned(),
        comment: recipe.comment.to_owned(),
    };
    let result = state.db_connection.insert_recipe(&recipe).await;
    match result {
        Ok(r) => (StatusCode::CREATED, Json(Some(r.recipe_id))),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}

pub async fn update(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Json(recipe): Json<Recipe>,
) -> impl axum::response::IntoResponse {
    let recipe = foodlib::Recipe {
        recipe_id,
        name: recipe.name.to_owned(),
        comment: recipe.comment.to_owned(),
    };
    let result = state.db_connection.update_recipe(&recipe).await;
    match result {
        Ok(r) => (StatusCode::CREATED, Json(Some(r.recipe_id))),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}

pub async fn list(State(state): State<MyAppState>) -> impl axum::response::IntoResponse {
    let recipes = state.db_connection.get_recipes().await;
    match recipes {
        Ok(recipes) => (StatusCode::OK, Json(recipes)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn list_meta_ingredients(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> impl axum::response::IntoResponse {
    let subrecipes = state.db_connection.get_meta_ingredients(recipe_id).await;
    match subrecipes {
        Ok(subrecipes) => (StatusCode::OK, Json(subrecipes)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn list_all_meta_ingredients(
    State(state): State<MyAppState>,
) -> impl axum::response::IntoResponse {
    let subrecipes = state.db_connection.get_all_meta_ingredients().await;
    match subrecipes {
        Ok(subrecipes) => (StatusCode::OK, Json(subrecipes)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn list_subrecipes(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> impl axum::response::IntoResponse {
    let subrecipes = state
        .db_connection
        .get_recipe_meta_ingredients(recipe_id)
        .await;
    match subrecipes {
        Ok(subrecipes) => (StatusCode::OK, Json(subrecipes)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn list_ingredients(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> impl axum::response::IntoResponse {
    let ingredients = state.db_connection.get_recipe_ingredients(recipe_id).await;
    match ingredients {
        Ok(ingredients) => (StatusCode::OK, Json(ingredients)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn update_entry(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Json(entries): Json<Vec<RecipeIngredient>>,
) -> impl axum::response::IntoResponse {
    let result = state
        .db_connection
        .update_recipe_entries(recipe_id, entries.into_iter())
        .await;
    match result {
        Ok(_) => StatusCode::CREATED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn update_steps(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Json(steps): Json<Vec<SerializableRecipeStep>>,
) -> impl axum::response::IntoResponse {
    let steps = steps.into_iter().map(deserialize_step).collect::<Vec<_>>();
    let result = state
        .db_connection
        .update_recipe_steps(recipe_id, steps.into_iter())
        .await;
    match result {
        Ok(_) => StatusCode::CREATED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn deserialize_step(step: SerializableRecipeStep) -> RecipeStep {
    log::debug!("step: {:?}", step);
    RecipeStep {
        step_id: step.step_id,
        step_order: step.step_order,
        step_name: step.step_name,
        step_description: step.step_description,
        recipe_id: step.recipe_id,
        fixed_duration: PgInterval::try_from(step.fixed_duration).unwrap(),
        duration_per_kg: PgInterval::try_from(step.duration_per_kg).unwrap(),
    }
}

fn serialize_step(step: RecipeStep) -> SerializableRecipeStep {
    SerializableRecipeStep {
        step_id: step.step_id,
        step_order: step.step_order,
        step_name: step.step_name,
        step_description: step.step_description,
        recipe_id: step.recipe_id,
        fixed_duration: Duration::default(),
        duration_per_kg: Duration::default(),
    }
}

pub async fn get_steps(
    Path(recipe_id): Path<i32>,
    State(state): State<MyAppState>,
) -> impl axum::response::IntoResponse {
    let steps = state.db_connection.get_recipe_steps(recipe_id).await;
    match steps {
        Ok(steps) => {
            let serialized_steps: Vec<_> = steps.into_iter().map(serialize_step).collect();
            (StatusCode::OK, Json(serialized_steps))
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}
