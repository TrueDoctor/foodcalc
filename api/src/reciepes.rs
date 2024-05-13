use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/", get(list_reciepes))
        .route("/", put(add_reciepe))
        .route("/:recipe_id/", get(calc_reciepe))
        .route("/:recipe_id/", post(update_reciepe))
        .route("/:recipe_id/", delete(update_reciepe))
        .route("/:recipe_id/steps/", get(update_reciepe))
        .route("/:recipe_id/steps/", post(update_reciepe))
        .route("/:recipe_id/ingredients/", get(calc_reciepe))
        .route("/:recipe_id/ingredients/", post(calc_reciepe))
}

async fn list_reciepes() -> impl IntoResponse {
    StatusCode::OK
}

async fn add_reciepe() -> impl IntoResponse {
    StatusCode::OK
}

async fn calc_reciepe() -> impl IntoResponse {
    StatusCode::OK
}

async fn update_reciepe() -> impl IntoResponse {
    StatusCode::OK
}
