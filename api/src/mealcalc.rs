use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};

use crate::ApiState;

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/meal/{meal_id}", get(calc_meal))
        .route("/recipe/{recipe_id}", get(calc_recipe))
}

async fn calc_meal(State(_state): State<ApiState>, Path(_meal_id): Path<i32>) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn calc_recipe(
    State(state): State<ApiState>,
    Path(recipe_id): Path<i32>,
) -> impl IntoResponse {
    let people = 1.;
    let calories = 2600;

    let _recipe_data = state
        .food_base
        .get_recipe_from_string_reference(recipe_id.to_string())
        .await
        .unwrap();

    let subrecipes = state
        .food_base
        .fetch_subrecipes_from_user_input(recipe_id, people, calories)
        .await
        .unwrap();

    (
        StatusCode::OK,
        state.food_base.format_subrecipes_markdown(subrecipes).await,
    )
}
