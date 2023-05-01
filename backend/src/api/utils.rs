use std::sync::Arc;

use axum::{routing::{MethodRouter, get}, Json, http::StatusCode};

use crate::db::FoodBase;

pub fn get_places(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    get(move || async move {
        let result = db.get_places().await;
        match result {
            Ok(places) => (StatusCode::OK, Json(places)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn get_units(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    get(move || async move {
        let result = db.get_units().await;
        match result {
            Ok(units) => (StatusCode::OK, Json(units)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}