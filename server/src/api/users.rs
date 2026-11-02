//! User and group management for the SPA (admin + group members).
//!
//! Permission model (intentionally simple today, designed to be role-ready):
//! `is_admin` grants global access; group membership grants access to that
//! group. Every decision funnels through the two choke points below —
//! [`require_admin`] and [`require_group_manager`] — plus the existing
//! `AuthCtx::assert_*` methods. A future per-group role model only replaces
//! the internals of these helpers (plus one schema migration); nothing else
//! in the API layer may decide permissions inline.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use foodlib_new::{
    auth_context::AuthCtx,
    entities::user::{Group, User},
    Error,
};

use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

pub fn users_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", put(update_user).delete(delete_user))
        .route("/{id}/password", put(change_password))
}

pub fn groups_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_groups).post(create_group))
        .route("/{id}", delete(delete_group))
        .route("/{id}/members", get(list_members).post(add_member))
        .route("/{id}/members/{user_id}", delete(remove_member))
}

// ------------------------------------------------------------- permission

async fn require_admin(ctx: &AuthCtx) -> Result<(), Response> {
    if ctx.user.is_admin {
        Ok(())
    } else {
        Err(map_api_error(Error::Forbidden(
            "admin privileges required".into(),
        ))
        .into_response())
    }
}

/// Admin or member of the group; personal groups can never be managed this
/// way (their membership follows the owning user account).
async fn require_group_manager(
    state: &MyAppState,
    ctx: &AuthCtx,
    group_id: i32,
) -> Result<Group, Response> {
    let group = match state.users().get_group(group_id).await {
        Ok(group) => group,
        Err(err) => return Err(map_api_error(err).into_response()),
    };
    if group.is_personal {
        return Err(bad_request(
            "personal groups are managed through the owning user account".into(),
        ));
    }
    if !ctx.user.is_admin && !ctx.group_ids.contains(&group_id) {
        return Err(map_api_error(Error::Forbidden(
            "you are not a member of this group".into(),
        ))
        .into_response());
    }
    Ok(group)
}

// ------------------------------------------------------------------ users

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserWithGroups {
    #[serde(flatten)]
    pub member: ApiMember,
    pub groups: Vec<Group>,
}

/// `GET /api/users` (admin) — users with their group memberships.
async fn list_users(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<UserListQuery>,
) -> Response {
    if let Err(response) = require_admin(&ctx).await {
        return response;
    }
    let rows = match state.users().list_users_with_groups().await {
        Ok(rows) => rows,
        Err(err) => return map_api_error(err).into_response(),
    };

    let search = query.search.unwrap_or_default().to_lowercase();
    let items: Vec<UserWithGroups> = rows
        .into_iter()
        .filter(|(user, _)| {
            search.is_empty()
                || user.username.to_lowercase().contains(&search)
                || user.email.to_lowercase().contains(&search)
        })
        .map(|(user, groups)| UserWithGroups {
            member: ApiMember::from(user),
            groups,
        })
        .collect();

    (StatusCode::OK, Json(items)).into_response()
}

#[derive(Debug, Deserialize)]
pub struct UserUpdateInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_admin: Option<bool>,
}

/// `PUT /api/users/{id}` (admin) — change username/email/admin flag.
async fn update_user(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
    Json(payload): Json<UserUpdateInput>,
) -> Response {
    if let Err(response) = require_admin(&ctx).await {
        return response;
    }
    let current = match state.users().get_user(id).await {
        Ok(user) => user,
        Err(err) => return map_api_error(err).into_response(),
    };
    if current.id == ctx.user.id && payload.is_admin == Some(false) {
        return bad_request("you cannot revoke your own admin privileges".into());
    }

    let updated = User {
        id: current.id,
        username: payload.username.unwrap_or(current.username),
        email: payload.email.unwrap_or(current.email),
        password_hash: current.password_hash,
        is_admin: payload.is_admin.unwrap_or(current.is_admin),
        created_at: current.created_at,
    };
    match state.users().update_user(updated).await {
        Ok(user) => (StatusCode::OK, Json(serde_json::json!(ApiMember::from(user))))
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/users/{id}` (admin) — cascade delete; refuses when the
/// user's personal group still owns entities.
async fn delete_user(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_admin(&ctx).await {
        return response;
    }
    if id == ctx.user.id {
        return bad_request("you cannot delete your own account here".into());
    }
    match state.users().delete_user_cascade(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PasswordChangeInput {
    /// Required when a user changes their own password.
    pub current_password: Option<String>,
    pub new_password: String,
}

/// `PUT /api/users/{id}/password` — self-service (with current password)
/// or admin reset (without).
async fn change_password(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
    Json(payload): Json<PasswordChangeInput>,
) -> Response {
    let is_self = ctx.user.id == id;
    if !is_self {
        if let Err(response) = require_admin(&ctx).await {
            return response;
        }
    }

    if payload.new_password.len() < 8 {
        return bad_request("password must be at least 8 characters".into());
    }
    if is_self {
        let current_password = match &payload.current_password {
            Some(password) => password,
            None => {
                return bad_request(
                    "current_password is required to change your own password".into(),
                )
            }
        };
        if !current_password.is_empty()
            && !bcrypt::verify(current_password, &ctx.user.password_hash).unwrap_or(false)
        {
            return bad_request("current password is incorrect".into());
        }
    }

    match state
        .users()
        .update_password(id, payload.new_password)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ----------------------------------------------------------------- groups

/// User projection without the password hash — the entity must never be
/// serialized directly to API clients.
#[derive(Debug, Serialize)]
pub struct ApiMember {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<User> for ApiMember {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
            created_at: user.created_at,
        }
    }
}

/// `GET /api/groups` — admins see every group, members their own (picker
/// semantics stay compatible with `masterApi.groups`).
async fn list_groups(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    let groups = if ctx.user.is_admin {
        state.users().get_all_groups().await
    } else {
        state.users().get_user_groups(ctx.user.id).await
    };
    respond(groups)
}

#[derive(Debug, Deserialize)]
pub struct GroupCreateInput {
    pub name: String,
}

/// `POST /api/groups` (admin) — create a non-personal group.
async fn create_group(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<GroupCreateInput>,
) -> Response {
    if let Err(response) = require_admin(&ctx).await {
        return response;
    }
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return bad_request("group name must not be empty".into());
    }
    if state
        .users()
        .get_all_groups()
        .await
        .map(|groups| groups.iter().any(|group| group.name == name))
        .unwrap_or(false)
    {
        return bad_request(format!("group name already taken: {name}"));
    }

    respond_created(state.users().create_group(&name).await)
}

/// `DELETE /api/groups/{id}` (admin) — refuses personal and non-empty groups.
async fn delete_group(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(response) = require_admin(&ctx).await {
        return response;
    }
    let group = match state.users().get_group(id).await {
        Ok(group) => group,
        Err(err) => return map_api_error(err).into_response(),
    };
    if group.is_personal {
        return bad_request(
            "personal groups are deleted with their owning user".into(),
        );
    }
    match state.users().count_owned_entities(id).await {
        Ok(count) if count > 0 => {
            return bad_request(format!(
                "group still owns {count} entities — reassign or delete them first"
            ))
        }
        Ok(_) => {}
        Err(err) => return map_api_error(err).into_response(),
    }
    match state.users().delete_group(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/groups/{id}/members` — admin or group member.
async fn list_members(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if !ctx.user.is_admin && !ctx.group_ids.contains(&id) {
        return map_api_error(Error::Forbidden(
            "you are not a member of this group".into(),
        ))
        .into_response();
    }
    let members = match state.users().get_group_members(id).await {
        Ok(members) => members,
        Err(err) => return map_api_error(err).into_response(),
    };
    (StatusCode::OK, Json(members.into_iter().map(ApiMember::from).collect::<Vec<_>>()))
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct AddMemberInput {
    /// Username, email or numeric id of the user to add.
    pub user: String,
}

/// `POST /api/groups/{id}/members` — admin or group manager; personal
/// groups are excluded. Resolves the member by username/email/id.
async fn add_member(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<AddMemberInput>,
) -> Response {
    if let Err(response) = require_group_manager(&state, &ctx, id).await {
        return response;
    }

    let user = match state
        .users()
        .get_user_by_string_reference(payload.user.trim().to_string())
        .await
    {
        Some(user) => user,
        None => {
            return bad_request(format!("unknown user: {}", payload.user));
        }
    };
    match state.users().add_user_to_group(user.id, id).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!(ApiMember::from(user))),
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/groups/{id}/members/{user_id}` — admin or group manager.
async fn remove_member(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, user_id)): Path<(i32, i64)>,
) -> Response {
    if let Err(response) = require_group_manager(&state, &ctx, id).await {
        return response;
    }
    match state.users().remove_user_from_group(user_id, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}
