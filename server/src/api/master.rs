//! Master data read endpoints (stores, units, properties, places) used
//! by the pickers in the UI. Groups live in `api::users` (admin-aware).
//! Handlers are thin: delegation to `crate::app::master`.

use axum::{extract::State, response::Response};

use crate::api::respond;
use crate::app::master;
use crate::MyAppState;

pub async fn stores(State(state): State<MyAppState>) -> Response {
    respond(master::get_available_stores(&state).await)
}

pub async fn units(State(state): State<MyAppState>) -> Response {
    respond(master::get_available_units(&state).await)
}

pub async fn properties(State(state): State<MyAppState>) -> Response {
    respond(master::get_food_properties(&state).await)
}

/// Places where meals are served (event meal picker).
pub async fn places(State(state): State<MyAppState>) -> Response {
    respond(master::get_available_event_places(&state).await)
}
