use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/", get(list_places))
        .route("/", put(add_places))
        .route("/:place_id", get(get_place))
        .route("/:place_id", post(update_place))
        .route("/:place_id", delete(delete_place))
}

async fn list_places() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn add_places() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn get_place() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn update_place() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

async fn delete_place() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}
