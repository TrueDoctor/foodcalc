use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use foodlib::Recipe;
use serde::{Deserialize, Serialize};

use crate::ApiState;

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/", get(list_reciepes))
        .route("/", put(add_reciepe))
        .route("/:recipe_id/", get(get_recipe))
        .route("/:recipe_id/", post(update_reciepe))
        .route("/:recipe_id/", delete(update_reciepe))
        .route("/:recipe_id/steps/", get(update_reciepe))
        .route("/:recipe_id/steps/", post(update_reciepe))
        .route("/:recipe_id/ingredients/", get(calc_reciepe))
        .route("/:recipe_id/ingredients/", post(calc_reciepe))
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

async fn update_recipe(
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
        })
        .await
    {
        Ok(recipe) => (StatusCode::OK, Json(recipe_to_body(recipe))).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn calc_reciepe() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn update_reciepe() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}
