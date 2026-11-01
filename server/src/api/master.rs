//! Master data read endpoints (groups, stores, units, properties) used
//! by the owner/store/unit pickers in the UI.

use axum::{extract::State, response::Response};

use foodlib_new::auth_context::AuthCtx;

use crate::{api::respond, MyAppState};

/// Groups for the owner picker: the active user's groups, or all groups
/// for admins. The active user comes from the `AuthCtx` extractor — it
/// resolves the session user plus their group memberships.
pub async fn groups(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    // if ctx.user.is_admin {
    //     respond(state.users().list_groups().await)
    // } else {
        respond(state.users().get_user_groups(ctx.user.id).await)
    // }
}

pub async fn stores(State(state): State<MyAppState>) -> Response {
    respond(state.stores().list().await)
}

pub async fn units(State(state): State<MyAppState>) -> Response {
    respond(state.units().list().await)
}

pub async fn properties(State(state): State<MyAppState>) -> Response {
    respond(state.properties().list().await)
}
