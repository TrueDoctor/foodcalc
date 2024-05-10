use axum::{http::StatusCode, response::IntoResponse, routing::{delete, get, post, put}, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list))
        .route("/", post(update_event))
        .route("/:event_id/", get(show_event))
        .route("/:event_id/", delete(delete_event))
        .route("/:event_id/", post(update_event))
        .route("/:event_id/meals/", put(meal_add))
        .route("/:event_id/meals/", delete(meal_delete))
        .route("/:event_id/meals/", post(meal_update))
}

async fn list() -> impl IntoResponse {
    StatusCode::OK
}

async fn show_event() -> impl IntoResponse {
    StatusCode::OK
}

async fn delete_event() -> impl IntoResponse {
    StatusCode::OK
}

async fn update_event() -> impl IntoResponse {
    StatusCode::OK
}

async fn meal_add() -> impl IntoResponse {
    StatusCode::OK
}

async fn meal_delete() -> impl IntoResponse {
    StatusCode::OK
}

async fn meal_update() -> impl IntoResponse {
    StatusCode::OK
}
