//! Session-cookie authentication endpoints for the SPA.
//!
//! These live under `/api/auth/*` and are separate from the HTML login
//! flow (`/auth/*`, maud frontend) but share the same session backend,
//! so a login in either UI is valid in the other.
//!
//! The session mechanics (`AuthSession`) are transport-specific and stay
//! here; registration, profile resolution and the auth audit trail live in
//! `crate::app::auth`.

use crate::app::auth;
use crate::api::map_api_error;
use crate::MyAppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use foodlib_new::auth::{AuthSession, Credentials};
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::ops::audit::action;
use foodlib_new::Error;
use serde::Deserialize;

// Input DTOs shared with the app layer (the profile DTOs live in `app::auth`
// and are consumed by other transports too).
pub use crate::app::auth::RegisterInput;

/// `POST /api/auth/login` payload.
#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

fn unauthorized(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": message })),
    )
}

/// Login with username and password. On success the session cookie is
/// set; the SPA resolves the current user via `/api/auth/me` afterwards.
pub async fn login(
    State(state): State<MyAppState>,
    auth_session: AuthSession,
    Json(payload): Json<LoginInput>,
) -> Response {
    let attempted_username = payload.username.clone();
    let credentials = Credentials {
        username: payload.username,
        password: payload.password,
    };

    let result = foodlib_new::auth::login(auth_session, credentials).await;
    match result {
        Ok(_) => {
            let actor = state
                .users()
                .get_user_by_string_reference(attempted_username.clone())
                .await;
            auth::audit_auth(
                &state,
                actor.as_ref().map(|user| user.id),
                &attempted_username,
                action::LOGIN_SUCCESS,
                serde_json::json!({}),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(Error::UserNotFound { .. }) => {
            auth::audit_auth(
                &state,
                None,
                &attempted_username,
                action::LOGIN_FAILED,
                serde_json::json!({}),
            )
            .await;
            unauthorized("wrong credentials").into_response()
        }
        Err(err) => {
            auth::audit_auth(
                &state,
                None,
                &attempted_username,
                action::LOGIN_FAILED,
                serde_json::json!({ "error": err.to_string() }),
            )
            .await;
            map_api_error(err).into_response()
        }
    }
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
