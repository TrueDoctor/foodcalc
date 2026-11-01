use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{api::map_api_error, MyAppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCreateInput {
    pub name: String,
    pub comment: Option<String>,
    pub group_id: i32,
    pub weighed_in: Option<bool>,
}

pub fn recipes_router() -> Router<MyAppState> {
    Router::new()
        .route("", get(list_recipes).post(create_recipe))
        .route("/:id", get(get_recipe))
}

async fn list_recipes(State(state): State<MyAppState>) -> impl IntoResponse {
    let recipes = state.recipes().list().await;
    match recipes {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn get_recipe(
    State(state): State<MyAppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let recipe = state.recipes().get(id).await;
    match recipe {
        Ok(item) => (StatusCode::OK, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

async fn create_recipe(
    State(state): State<MyAppState>,
    Json(payload): Json<RecipeCreateInput>,
) -> impl IntoResponse {
    let recipe = foodlib_new::entities::recipe::Recipe {
        id: 0,
        name: payload.name,
        comment: payload.comment,
        group_id: payload.group_id,
        weighed_in: payload.weighed_in.unwrap_or(false),
    };

    match state.recipes().create(recipe).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
