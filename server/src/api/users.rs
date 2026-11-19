//! User and group management for the SPA (global permissions + group policies).

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
    error::Error,
    policy::{GlobalPermission, GroupPerm, Permission},
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
        .route(
            "/{id}/members/{user_id}",
            put(set_member_roles).delete(remove_member),
        )
}

// ------------------------------------------------------------- permission

fn require_global(ctx: &AuthCtx, perm: GlobalPermission) -> Result<(), Response> {
    ctx.assert_global(perm)
        .map_err(|err| map_api_error(err).into_response())
}

/// Ensures the group exists, is NOT a personal workspace, and the caller holds
/// `group:manage_members` in that group.
async fn require_group_member_manager(
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
    if let Err(err) = ctx.assert_group_perm(group_id, Permission::Group(GroupPerm::ManageMembers)) {
        return Err(map_api_error(err).into_response());
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

/// `GET /api/users` — users with their group memberships.
async fn list_users(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<UserListQuery>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
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

/// `PUT /api/users/{id}` — change username/email/admin flag.
async fn update_user(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
    Json(payload): Json<UserUpdateInput>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
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

/// `DELETE /api/users/{id}` — cascade delete; refuses when the user's
/// personal group still owns entities.
async fn delete_user(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
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
/// or user-manager reset (without).
async fn change_password(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
    Json(payload): Json<PasswordChangeInput>,
) -> Response {
    let is_self = ctx.user.id == id;
    if !is_self {
        if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
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

/// `GET /api/groups` — user managers see every group (for admin assignment),
/// regular members see their own.
async fn list_groups(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    let groups = if ctx.can_global(GlobalPermission::ManageUsers) {
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

/// `POST /api/groups` — create a non-personal group and assign the creator
/// as `manager`.
async fn create_group(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<GroupCreateInput>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::AllowedToCreateNewDomainObjects) {
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

    // `create_group_with_owner(name, user_id)` creates the group and inserts
    // `(user_id, group_id, "manager")` into `user_group_roles` in one tx.
    respond_created(
        state
            .users()
            .create_group_with_manager(&name, ctx.user.id)
            .await,
    )
}

/// `DELETE /api/groups/{id}` — requires `group:delete` on the group; refuses
/// personal and non-empty groups.
async fn delete_group(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_group_perm(id, Permission::Group(GroupPerm::Delete)) {
        return map_api_error(err).into_response();
    }
    let group = match state.users().get_group(id).await {
        Ok(group) => group,
        Err(err) => return map_api_error(err).into_response(),
    };
    if group.is_personal {
        return bad_request("personal groups are deleted with their owning user".into());
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

/// `GET /api/groups/{id}/members` — members (or user admins) see member list
/// with all assigned role slugs.
async fn list_members(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if !ctx.can_global(GlobalPermission::ManageUsers) && !ctx.can_access_group(id) {
        return map_api_error(Error::Forbidden(
            "you are not a member of this group".into(),
        ))
            .into_response();
    }
    let members = match state.users().get_group_members_with_roles(id).await {
        Ok(members) => members,
        Err(err) => return map_api_error(err).into_response(),
    };
    let items = members
        .into_iter()
        .map(|(user, roles)| ApiMemberWithRoles {
            member: ApiMember::from(user),
            roles,
        })
        .collect::<Vec<_>>();
    (StatusCode::OK, Json(items)).into_response()
}

/// Member projection plus all assigned role slugs in the group.
#[derive(Debug, Serialize)]
pub struct ApiMemberWithRoles {
    #[serde(flatten)]
    pub member: ApiMember,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetMemberRolesInput {
    /// Non-empty list of role slugs (e.g. `["viewer"]` or `["editor", "kitchen"]`).
    pub roles: Vec<String>,
}

/// `PUT /api/groups/{id}/members/{user_id}` — replaces the member's role set.
/// Refuses if the change would leave the group with zero members holding
/// `group:manage_members`.
async fn set_member_roles(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, user_id)): Path<(i32, i64)>,
    Json(payload): Json<SetMemberRolesInput>,
) -> Response {
    if let Err(response) = require_group_member_manager(&state, &ctx, id).await {
        return response;
    }

    let mut roles: Vec<String> = payload
        .roles
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    roles.sort();
    roles.dedup();

    if roles.is_empty() {
        return bad_request("a group member must have at least one role".into());
    }

    // Validate all role slugs exist in DB `roles` table
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    // Ensure at least one member in the group retains `group:manage_members`
    let would_leave_unmanaged = match state
        .users()
        .would_leave_group_without_permission(
            id,
            user_id,
            &roles,
            Permission::Group(GroupPerm::ManageMembers),
        )
        .await
    {
        Ok(unmanaged) => unmanaged,
        Err(err) => return map_api_error(err).into_response(),
    };
    if would_leave_unmanaged {
        return bad_request(
            "cannot remove the last member able to manage group members".into(),
        );
    }

    match state.users().set_member_roles(user_id, id, &roles).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddMemberInput {
    /// Username, email or numeric id of the user to add.
    pub user: String,
    /// Optional initial role slugs (defaults to `["viewer"]`).
    pub roles: Option<Vec<String>>,
}

/// `POST /api/groups/{id}/members` — adds a user to the group with initial roles.
async fn add_member(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<AddMemberInput>,
) -> Response {
    if let Err(response) = require_group_member_manager(&state, &ctx, id).await {
        return response;
    }

    let user = match state
        .users()
        .get_user_by_string_reference(payload.user.trim().to_string())
        .await
    {
        Some(user) => user,
        None => return bad_request(format!("unknown user: {}", payload.user)),
    };

    let roles = payload.roles.unwrap_or_else(|| vec!["viewer".to_string()]);
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    match state
        .users()
        .add_user_to_group_with_roles(user.id, id, &roles)
        .await
    {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!(ApiMemberWithRoles {
                member: ApiMember::from(user),
                roles,
            })),
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/groups/{id}/members/{user_id}` — refuses to remove the last
/// member holding `group:manage_members`.
async fn remove_member(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, user_id)): Path<(i32, i64)>,
) -> Response {
    if let Err(response) = require_group_member_manager(&state, &ctx, id).await {
        return response;
    }

    let would_leave_unmanaged = match state
        .users()
        .would_leave_group_without_permission(
            id,
            user_id,
            &[],
            Permission::Group(GroupPerm::ManageMembers),
        )
        .await
    {
        Ok(unmanaged) => unmanaged,
        Err(err) => return map_api_error(err).into_response(),
    };

    if would_leave_unmanaged {
        return bad_request(
            "cannot remove the last member able to manage group members".into(),
        );
    }

    match state.users().remove_user_from_group(user_id, id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------- admin self-grant

#[derive(Debug, Deserialize)]
pub struct AdminSelfGrantInput {
    pub group_id: i32,
    pub role: String,
}

/// `PUT /api/admin/me/groups` — a global user-manager may grant **themselves**
/// a role in any group; afterwards standard group policies apply.
pub async fn admin_self_grant(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<AdminSelfGrantInput>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
        return response;
    }
    let role = payload.role.trim();

    if !state.users().validate_role_slugs(&[role.to_string()]) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    match state
        .users()
        .grant_membership(ctx.user.id, payload.group_id, role)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}