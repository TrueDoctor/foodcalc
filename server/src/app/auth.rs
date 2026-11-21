//! Authentication use-cases.
//!
//! The session-cookie transport itself (login/logout via `AuthSession`) is
//! transport-specific and stays in `api::auth`; the user-facing business
//! logic — self-registration, resolving the current user's profile, and the
//! auth audit trail — lives here so other transports can reuse it.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::types::ids::{GroupId, UserId};
use foodlib_new::entities::user::{GroupMembership, User};
use foodlib_new::ops::audit::{action, AuditRecord};
use foodlib_new::policy::{GlobalPermission, Permission};

use crate::app::{Error, MyAppState, Result};

/// Group as exposed to clients (with the session user's roles in it).
#[derive(Debug, Serialize)]
pub struct ApiGroup {
    pub id: GroupId,
    pub name: String,
    pub is_personal: bool,
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
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub global_permissions: Vec<GlobalPermission>,
    pub groups: Vec<ApiGroup>,
}


#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

/// Verifies credentials and records the authentication audit trail
/// (`LOGIN_SUCCESS` or `LOGIN_FAILED`). Transport layers handle persisting
/// the session cookie or issuing a token.
pub async fn authenticate(state: &MyAppState, input: LoginInput) -> Result<User> {
    let attempted_username = input.username.trim().to_string();

    if attempted_username.is_empty() || input.password.is_empty() {
        audit_auth(
            state,
            None,
            &attempted_username,
            action::LOGIN_FAILED,
            serde_json::json!({}),
        )
            .await;
        return Err(Error::Unauthorized("wrong credentials".into()));
    }

    let user = state
        .users()
        .get_user_by_string_reference(attempted_username.clone())
        .await;

    let valid_user = user.filter(|u| {
        bcrypt::verify(&input.password, &u.password_hash).unwrap_or(false)
    });

    match valid_user {
        Some(user) => {
            audit_auth(
                state,
                Some(user.id),
                &user.username,
                action::LOGIN_SUCCESS,
                serde_json::json!({}),
            )
                .await;
            Ok(user)
        }
        None => {
            audit_auth(
                state,
                None,
                &attempted_username,
                action::LOGIN_FAILED,
                serde_json::json!({}),
            )
                .await;
            Err(Error::Unauthorized("wrong credentials".into()))
        }
    }
}

/// Input payload for self-registration.
#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Open self-registration: creates the user together with their personal
/// group and default global permissions atomically. Returns the created user
/// (for the transport's session login) and the resolved profile.
pub async fn register(
    state: &MyAppState,
    input: RegisterInput
) -> Result<(User, ApiUser)> {
    let username = input.username.trim().to_string();
    let email = input.email.trim().to_string();

    if username.len() < 3 {
        return Err(Error::Validation {
            message: "username must be at least 3 characters".into(),
        });
    }
    if !email.contains('@') || email.len() < 5 {
        return Err(Error::Validation {
            message: "invalid email address".into(),
        });
    }
    if input.password.len() < 8 {
        return Err(Error::Validation {
            message: "password must be at least 8 characters".into(),
        });
    }
    // Uniqueness pre-check (the DB constraints are the final arbiter).
    if state
        .users()
        .get_user_by_string_reference(username.clone())
        .await
        .is_some()
    {
        return Err(Error::Validation {
            message: "username or email already taken".into(),
        });
    }

    let password_hash = bcrypt::hash(&input.password, 12)
        .map_err(|err| Error::Misc(err.to_string()))?;

    let (user, _personal_group) = state
        .users()
        .create_user_with_personal_group(User {
            id: UserId(0),
            username,
            email,
            password_hash,
            is_admin: false,
            created_at: OffsetDateTime::now_utc(),
        })
        .await?;

    audit_auth(
        state,
        Some(user.id),
        &user.username,
        action::REGISTER,
        serde_json::json!({ "email": user.email }),
    )
    .await;

    let ctx = AuthCtx::load(&state.db, user.clone()).await?;
    let profile = current_user(state, &ctx).await?;
    Ok((user, profile))
}

/// The authenticated user plus their groups (the `AuthCtx` already carries
/// the resolved policy; only the memberships are fetched).
pub async fn current_user(state: &MyAppState, ctx: &AuthCtx) -> Result<ApiUser> {
    let memberships = state.users().get_user_group_memberships(ctx.user.id).await?;
    let mut global_permissions: Vec<GlobalPermission> =
        ctx.policy.global_permissions.iter().copied().collect();
    global_permissions.sort_by_key(|p| p.as_str());

    Ok(ApiUser {
        id: ctx.user.id,
        username: ctx.user.username.clone(),
        email: ctx.user.email.clone(),
        is_admin: ctx.user.is_admin,
        global_permissions,
        groups: memberships
            .into_iter()
            .map(ApiGroup::from_membership)
            .collect(),
    })
}

/// Best-effort audit for authentication events; failures are logged but do
/// not change the response. Used for login/logout in `api::auth` and for
/// registration above.
pub(crate) async fn audit_auth(
    state: &MyAppState,
    actor_user_id: Option<UserId>,
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