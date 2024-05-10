use axum::{http::StatusCode, response::IntoResponse, routing::{delete, get, post, put}, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list))
        .route("/", put(add))
        .route("/:ingredient_id/", get(show_ingredient))
        .route("/:ingredient_id/", delete(delete_ingredient))
        .route("/:ingredient_id/", post(show_ingredient))
}

async fn add() -> impl IntoResponse {
    StatusCode::OK
}

async fn list() -> impl IntoResponse {
    StatusCode::OK
}

async fn show_ingredient() -> impl IntoResponse {
    StatusCode::OK
}

async fn delete_ingredient() -> impl IntoResponse {
    StatusCode::OK
}