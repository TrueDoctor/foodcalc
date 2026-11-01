//! Session-cookie authentication endpoints for the SPA.
//!
//! These live under `/api/auth/*` and are separate from the HTML login
//! flow (`/auth/*`, maud frontend) but share the same session backend,
//! so a login in either UI is valid in the other.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use foodlib_new::auth::{AuthSession, Credentials};
use foodlib_new::entities::user::{Group, User};
use serde::{Deserialize, Serialize};

use crate::{api::map_api_error, MyAppState};

/// `POST /api/auth/login` payload.
#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

/// Group as exposed to the SPA.
#[derive(Debug, Serialize)]
pub struct ApiGroup {
    pub id: i32,
    pub name: String,
    pub is_personal: bool,
}

impl From<Group> for ApiGroup {
    fn from(group: Group) -> Self {
        Self {
            id: group.id,
            name: group.name,
            is_personal: group.is_personal,
        }
    }
}

/// The authenticated user together with the groups they belong to.
#[derive(Debug, Serialize)]
pub struct ApiUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub groups: Vec<ApiGroup>,
}

impl ApiUser {
    fn new(user: User, groups: Vec<Group>) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
            groups: groups.into_iter().map(Into::into).collect(),
        }
    }
}

fn unauthorized(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": message })),
    )
}

/// Routes that require an authenticated session. Mounted behind
/// `require_api_auth` (see `api::api_router`).
pub fn protected_router() -> Router<MyAppState> {
    Router::new()
        .route("/logout", post(logout))
        .route("/me", get(me))
}

/// Login with username and password. On success the session cookie is
/// set; the SPA resolves the current user via `/api/auth/me` afterwards.
pub async fn login(auth: AuthSession, Json(payload): Json<LoginInput>) -> Response {
    let credentials = Credentials {
        username: payload.username,
        password: payload.password,
    };

    match foodlib_new::auth::login(auth, credentials).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(foodlib_new::Error::UserNotFound { .. }) => {
            unauthorized("wrong credentials").into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

/// Invalidate the current session.
pub async fn logout(mut auth: AuthSession) -> Response {
    if let Err(err) = auth.logout().await {
        return map_api_error(err.into()).into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

/// The session user plus their groups; 401 when not logged in.
pub async fn me(State(state): State<MyAppState>, auth: AuthSession) -> Response {
    let Some(user) = auth.user else {
        return unauthorized("authentication required").into_response();
    };

    match state.users().get_user_groups(user.id).await {
        Ok(groups) => (StatusCode::OK, Json(ApiUser::new(user, groups))).into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
