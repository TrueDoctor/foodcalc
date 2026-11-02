//! Master data read endpoints (stores, units, properties, places) used
//! by the pickers in the UI. Groups live in `api::users` (admin-aware).

use axum::{extract::State, response::Response};

use crate::{api::respond, MyAppState};

pub async fn stores(State(state): State<MyAppState>) -> Response {
    respond(state.stores().list().await)
}

pub async fn units(State(state): State<MyAppState>) -> Response {
    respond(state.units().list().await)
}

pub async fn properties(State(state): State<MyAppState>) -> Response {
    respond(state.properties().list().await)
}

/// Places where meals are served (event meal picker).
pub async fn places(State(state): State<MyAppState>) -> Response {
    respond(state.events().get_places().await)
}
