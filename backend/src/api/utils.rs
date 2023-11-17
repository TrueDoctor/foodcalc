use axum::{extract::State, http::StatusCode, Json};

use crate::MyAppState;

pub async fn get_places(State(state): State<MyAppState>) -> impl axum::response::IntoResponse {
    let result = state.db_connection.get_places().await;
    match result {
        Ok(places) => (StatusCode::OK, Json(places)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn get_units(State(state): State<MyAppState>) -> impl axum::response::IntoResponse {
    let result = state.db_connection.get_units().await;
    match result {
        Ok(units) => (StatusCode::OK, Json(units)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}
