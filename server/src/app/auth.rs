//! Authentication use-cases.
//!
//! The session-cookie transport itself (login/logout via `AuthSession`) is
//! transport-specific and stays in `api::auth`; the user-facing business
//! logic — self-registration, resolving the current user's profile, and the
//! auth audit trail — lives here so other transports can reuse it.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::user::{Group, GroupMembership, User};
use foodlib_new::ops::audit::{action, AuditRecord};
use foodlib_new::policy::{EventPerm, GlobalPermission, Permission};

use crate::app::{Error, MyAppState, Result};

/// Group as exposed to clients (with the session user's roles in it).
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
    /// Freshly registered profile: no groups beyond the personal one, the
    /// global create-permission and `manager` in the personal group.
    pub fn fresh_registration(user: User, personal_group: Group) -> Self {
        Self {
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
        }
    }
}

/// `POST /auth/register` payload.
#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Open self-registration: creates the user together with their personal
/// group atomically. Returns the created user (for the transport's session
/// login) and the profile as clients see it.
pub async fn register(state: &MyAppState, input: RegisterInput) -> Result<(User, ApiUser)> {
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

    let (user, personal_group) = state
        .users()
        .create_user_with_personal_group(User {
            id: 0,
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

    let profile = ApiUser::fresh_registration(user.clone(), personal_group);
    Ok((user, profile))
}

/// The authenticated user plus their groups (the `AuthCtx` already carries
/// the resolved policy; only the memberships are fetched).
pub async fn current_user(state: &MyAppState, ctx: &AuthCtx) -> Result<ApiUser> {
    let memberships = state.users().get_user_group_memberships(ctx.user.id).await?;
    Ok(ApiUser {
        id: ctx.user.id,
        username: ctx.user.username.clone(),
        email: ctx.user.email.clone(),
        is_admin: ctx.user.is_admin,
        global_permissions: ctx.policy.global_permissions.iter().cloned().collect(),
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
