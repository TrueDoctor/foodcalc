use axum::{http::StatusCode, response::IntoResponse, routing::{delete, get, post, put}, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_places))
        .route("/", put(add_places))
        .route("/:place_id", get(get_place))
        .route("/:place_id", post(update_place))
        .route("/:place_id", delete(delete_place))
}

async fn list_places() -> impl IntoResponse {
    StatusCode::OK
}

async fn add_places() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_place() -> impl IntoResponse {
    StatusCode::OK
}

async fn update_place() -> impl IntoResponse {
    StatusCode::OK
}

async fn delete_place() -> impl IntoResponse {
    StatusCode::OK
}