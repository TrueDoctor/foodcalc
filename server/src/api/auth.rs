//! Session-cookie authentication endpoints for the SPA.
//!
//! These live under `/api/auth/*` and are separate from the HTML login
//! flow (`/auth/*`, maud frontend) but share the same session backend,
//! so a login in either UI is valid in the other.

use crate::{api::map_api_error, MyAppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use foodlib_new::auth::{AuthSession, Credentials};
use foodlib_new::entities::user::{GroupMembership, User};
use foodlib_new::ops::audit::{action, AuditRecord};
use foodlib_new::policy::{EventPerm, GlobalPermission, Permission};
use foodlib_new::Error;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// `POST /api/auth/login` payload.
#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

/// Group as exposed to the SPA (with the session user's role in it).
#[derive(Debug, Serialize)]
pub struct ApiGroup {
    pub id: i32,
    pub name: String,
    pub is_personal: bool,
    /// `viewer` | `editor` | `manager` (personal groups: `manager`).
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>,
}

impl ApiGroup {
    pub fn from_membership(membership: GroupMembership) -> Self {
        Self {
            id: membership.id,
            name: membership.name,
            is_personal: membership.is_personal,
            roles: membership.roles,
            permissions: membership.permissions,
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
    pub global_permissions: Vec<GlobalPermission>,
    pub groups: Vec<ApiGroup>,
}

impl ApiUser {
    fn new(user: User, global_permissions: Vec<GlobalPermission>, memberships: Vec<GroupMembership>) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
            global_permissions: global_permissions,
            groups: memberships
                .into_iter()
                .map(ApiGroup::from_membership)
                .collect(),
        }
    }
}

fn unauthorized(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": message })),
    )
}

fn bad_input(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": message })),
    )
}

/// Login with username and password. On success the session cookie is
/// set; the SPA resolves the current user via `/api/auth/me` afterwards.
pub async fn login(
    State(state): State<MyAppState>,
    auth: AuthSession,
    Json(payload): Json<LoginInput>,
) -> Response {
    let attempted_username = payload.username.clone();
    let credentials = Credentials {
        username: payload.username,
        password: payload.password,
    };

    let result = foodlib_new::auth::login(auth, credentials).await;
    match result {
        Ok(_) => {
            let actor = state
                .users()
                .get_user_by_string_reference(attempted_username.clone())
                .await;
            audit_auth(
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
            audit_auth(
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
            audit_auth(
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

/// Best-effort audit for authentication events; failures are logged but do
/// not change the response.
async fn audit_auth(
    state: &MyAppState,
    actor_user_id: Option<i64>,
    actor_username: &str,
    action: &'static str,
    details: serde_json::Value,
) {
    if let Err(err) = state
        .audit()
        .record(AuditRecord {
            actor_user_id,
            actor_username,
            action,
            details: Some(details),
            ..Default::default()
        })
        .await
    {
        log::error!("auth event not audited: {err}");
    }
}

/// `POST /api/auth/register` payload.
#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Open self-registration: creates the user together with their personal
/// group atomically and logs them in. Public endpoint — validation happens
/// here (username/email/password policy, uniqueness pre-check).
pub async fn register(
    State(state): State<MyAppState>,
    mut auth: AuthSession,
    Json(payload): Json<RegisterInput>,
) -> Response {
    let username = payload.username.trim().to_string();
    let email = payload.email.trim().to_string();

    if username.len() < 3 {
        return bad_input("username must be at least 3 characters").into_response();
    }
    if !email.contains('@') || email.len() < 5 {
        return bad_input("invalid email address").into_response();
    }
    if payload.password.len() < 8 {
        return bad_input("password must be at least 8 characters").into_response();
    }
    // Uniqueness pre-check (the DB constraints are the final arbiter).
    if state
        .users()
        .get_user_by_string_reference(username.clone())
        .await
        .is_some()
    {
        return bad_input("username or email already taken").into_response();
    }

    let password_hash = match bcrypt::hash(&payload.password, 12) {
        Ok(hash) => hash,
        Err(err) => return map_api_error(Error::Misc(err.to_string())).into_response(),
    };

    let created = state
        .users()
        .create_user_with_personal_group(User {
            id: 0,
            username,
            email,
            password_hash,
            is_admin: false,
            created_at: OffsetDateTime::now_utc(),
        })
        .await;

    let (user, personal_group) = match created {
        Ok(pair) => pair,
        Err(err) => return map_api_error(err).into_response(),
    };

    // Log the freshly created user in (same session backend as login).
    if let Err(err) = auth.login(&user).await {
        return map_api_error(err.into()).into_response();
    }

    audit_auth(
        &state,
        Some(user.id),
        &user.username,
        action::REGISTER,
        serde_json::json!({ "email": user.email }),
    )
    .await;

    (
        StatusCode::CREATED,
        Json(ApiUser {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
            global_permissions: vec![GlobalPermission::AllowedToCreateNewDomainObjects],
            groups: vec![ApiGroup {
                id: personal_group.id,
                name: personal_group.name,
                is_personal: personal_group.is_personal,
                roles: vec!["manager".into()],
                permissions: vec![Permission::Event(EventPerm::Edit)],
            }],
        }),
    )
        .into_response()
}

/// Invalidate the current session.
pub async fn logout(State(state): State<MyAppState>, mut auth: AuthSession) -> Response {
    let actor = auth.user.clone();
    if let Err(err) = auth.logout().await {
        return map_api_error(err.into()).into_response();
    }
    if let Some(user) = actor {
        audit_auth(
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

/// The session user plus their groups; 401 when not logged in.
pub async fn me(State(state): State<MyAppState>, auth: AuthSession) -> Response {
    let Some(user) = auth.user else {
        return unauthorized("authentication required").into_response();
    };

    let policy = match state.users().get_user_policy(user.id).await {
        Ok(policy) => policy,
        Err(err) => return map_api_error(err).into_response(),
    };
    let global_permissions = policy.global_permissions.into_iter().collect();
    let memberships = match state.users().get_user_group_memberships(user.id).await {
        Ok(memberships) => memberships,
        Err(err) => return map_api_error(err).into_response(),
    };
    (StatusCode::OK, Json(ApiUser::new(user, global_permissions, memberships))).into_response()
}
