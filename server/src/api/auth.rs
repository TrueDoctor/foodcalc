//! Session-cookie authentication endpoints for the SPA.
//!
//! These live under `/api/auth/*` and are separate from the HTML login
//! flow (`/auth/*`, maud frontend) but share the same session backend,
//! so a login in either UI is valid in the other.
//!
//! The session mechanics (`AuthSession`) are transport-specific and stay
//! here; registration, profile resolution and the auth audit trail live in
//! `crate::app::auth`.

use crate::api::map_api_error;
use crate::app::auth;
use crate::app::auth::LoginInput;
// Input DTOs shared with the app layer (the profile DTOs live in `app::auth`
// and are consumed by other transports too).
pub use crate::app::auth::RegisterInput;
use crate::MyAppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use foodlib_new::auth::AuthSession;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::ops::audit::action;


fn unauthorized(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": message })),
    )
}

pub async fn login(
    State(state): State<MyAppState>,
    mut auth_session: AuthSession,
    Json(payload): Json<LoginInput>,
) -> Response {
    let user = match auth::authenticate(&state, payload).await {
        Ok(user) => user,
        Err(foodlib_new::Error::Unauthorized(message)) => {
            return unauthorized(&message).into_response()
        }
        Err(err) => return map_api_error(err).into_response(),
    };

    if let Err(err) = auth_session.login(&user).await {
        return map_api_error(err.into()).into_response();
    }
    // Success is only audited once the session really persisted.
    auth::audit_auth(
        &state,
        Some(user.id),
        &user.username,
        action::LOGIN_SUCCESS,
        serde_json::json!({}),
    )
    .await;

    StatusCode::NO_CONTENT.into_response()
}

/// Open self-registration: creates the user together with their personal
/// group atomically (validation and audit in `app::auth`) and logs them in.
/// Public endpoint — no session required.
pub async fn register(
    State(state): State<MyAppState>,
    mut auth_session: AuthSession,
    Json(payload): Json<RegisterInput>,
) -> Response {
    let (user, profile) = match auth::register(&state, payload).await {
        Ok(pair) => pair,
        Err(err) => return map_api_error(err).into_response(),
    };

    // Log the freshly created user in (same session backend as login).
    if let Err(err) = auth_session.login(&user).await {
        return map_api_error(err.into()).into_response();
    }

    (StatusCode::CREATED, Json(profile)).into_response()
}

/// Invalidate the current session.
pub async fn logout(State(state): State<MyAppState>, mut auth_session: AuthSession) -> Response {
    let actor = auth_session.user.clone();
    if let Err(err) = auth_session.logout().await {
        return map_api_error(err.into()).into_response();
    }
    if let Some(user) = actor {
        auth::audit_auth(
            &state,
            Some(user.id),
            &user.username,
            action::LOGOUT,
            serde_json::json!({}),
        )
        .await;
    }
    StatusCode::NO_CONTENT.into_response()
}

/// The session user plus their groups. Unauthenticated requests are already
/// rejected by `require_api_auth`; the `AuthCtx` extractor resolves the
/// policy for the profile.
pub async fn me(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    match auth::current_user(&state, &ctx).await {
        Ok(profile) => (StatusCode::OK, Json(profile)).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
