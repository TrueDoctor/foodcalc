//! User, group and settings endpoints — thin handlers over
//! `crate::app::users`. Permission checks, validation and audit logging live
//! in the app layer.

use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{delete, get, put},
    Json, Router,
};

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::users;
use crate::app::AppCtx;
use crate::MyAppState;

// DTOs shared with the app layer.
pub use crate::app::users::{
    AddMemberInput, AdminSelfGrantInput, GlobalPermissionsInput, GroupCreateInput,
    GroupDefaultRolesInput, PasswordChangeInput,
    SetMemberRolesInput, UserListQuery, UserUpdateInput,
};

pub fn users_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", put(update_user).delete(delete_user))
        .route("/{id}/password", put(change_password))
        .route("/{id}/global-permissions", put(set_user_global_permissions))
}

/// Role catalog — served to the SPA; role UIs are rendered from this.
pub fn roles_router() -> Router<MyAppState> {
    Router::new().route("/", get(list_roles))
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
        .route(
            "/{id}/default-roles",
            get(get_group_default_roles)
                .put(put_group_default_roles)
                .delete(clear_group_default_roles),
        )
}

// ------------------------------------------------------------------ users

/// `GET /api/roles` — the role catalog. The SPA renders all role UIs
/// (pills, pickers, matrix legend) purely from this response.
async fn list_roles() -> Response {
    respond(Ok(users::role_catalog()))
}

/// `GET /api/users` — users with group roles and global permissions.
async fn list_users(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Query(query): Query<UserListQuery>,
) -> Response {
    respond(users::list_filtered_users(&state, &actor, query).await)
}

/// `PUT /api/users/{id}` — change username/email/admin flag.
async fn update_user(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i64>,
    Json(payload): Json<UserUpdateInput>,
) -> Response {
    respond(users::update_user(&state, &actor, id, payload).await)
}

/// `DELETE /api/users/{id}` — cascade delete; refuses when the user's
/// personal group still owns entities.
async fn delete_user(State(state): State<MyAppState>, actor: AppCtx, Path(id): Path<i64>) -> Response {
    respond_no_content(users::delete_user(&state, &actor, id).await)
}

/// `PUT /api/users/{id}/password` — self-service (with current password) or
/// user-manager reset (without).
async fn change_password(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i64>,
    Json(payload): Json<PasswordChangeInput>,
) -> Response {
    respond_no_content(users::change_password(&state, &actor, id, payload).await)
}

/// `PUT /api/users/{id}/global-permissions` — replaces the user's global
/// permissions (admin access drawer).
async fn set_user_global_permissions(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i64>,
    Json(payload): Json<GlobalPermissionsInput>,
) -> Response {
    respond_no_content(users::set_global_permissions(&state, &actor, id, payload).await)
}

// ----------------------------------------------------------------- groups

/// `GET /api/groups` — user managers see every group, regular members see
/// their memberships.
async fn list_groups(State(state): State<MyAppState>, actor: AppCtx) -> Response {
    respond(users::list_groups(&state, &actor).await)
}

/// `POST /api/groups` — create a non-personal group and assign the creator
/// as `manager`.
async fn create_group(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Json(payload): Json<GroupCreateInput>,
) -> Response {
    respond_created(users::create_group(&state, &actor, payload).await)
}

/// `DELETE /api/groups/{id}` — requires `group:delete` on the group; refuses
/// personal and non-empty groups.
async fn delete_group(State(state): State<MyAppState>, actor: AppCtx, Path(id): Path<i32>) -> Response {
    respond_no_content(users::delete_group(&state, &actor, id).await)
}

/// `GET /api/groups/{id}/members` — members (or user admins) see member list
/// with all assigned role slugs.
async fn list_members(State(state): State<MyAppState>, actor: AppCtx, Path(id): Path<i32>) -> Response {
    respond(users::list_members(&state, &actor, id).await)
}

/// `POST /api/groups/{id}/members` — adds a user to the group with initial roles.
async fn add_member(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<AddMemberInput>,
) -> Response {
    respond_created(users::add_member(&state, &actor, id, payload).await)
}

/// `PUT /api/groups/{id}/members/{user_id}` — replaces the member's role set.
async fn set_member_roles(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, user_id)): Path<(i32, i64)>,
    Json(payload): Json<SetMemberRolesInput>,
) -> Response {
    respond_no_content(users::set_member_roles(&state, &actor, id, user_id, payload).await)
}

/// `DELETE /api/groups/{id}/members/{user_id}` — refuses to remove the last
/// member holding `group:manage_members`.
async fn remove_member(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path((id, user_id)): Path<(i32, i64)>,
) -> Response {
    respond_no_content(users::remove_member(&state, &actor, id, user_id).await)
}

// ------------------------------------------------------- group defaults

/// `GET /api/groups/{id}/default-roles` — effective default roles.
async fn get_group_default_roles(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(users::get_default_roles(&state, &actor, id).await)
}

/// `PUT /api/groups/{id}/default-roles` — set the group's own default roles.
async fn put_group_default_roles(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<GroupDefaultRolesInput>,
) -> Response {
    respond_no_content(users::put_default_roles(&state, &actor, id, payload).await)
}

/// `DELETE /api/groups/{id}/default-roles` — reset to "inherit the global
/// default". Guard `group:manage_settings`.
async fn clear_group_default_roles(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond_no_content(users::clear_default_roles(&state, &actor, id).await)
}

// ------------------------------------------------------ system settings

/// `GET/PUT /api/settings/member-defaults` — global default roles for new
/// members of groups without their own default. Guard `admin:manage_settings`.
pub fn settings_router() -> Router<MyAppState> {
    Router::new().route(
        "/member-defaults",
        get(get_member_defaults).put(put_member_defaults),
    )
}

#[derive(Debug, serde::Deserialize)]
pub struct MemberDefaultsInput {
    pub roles: Vec<String>,
}

async fn get_member_defaults(State(state): State<MyAppState>, actor: AppCtx) -> Response {
    respond(users::get_member_defaults(&state, &actor).await)
}

async fn put_member_defaults(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Json(payload): Json<MemberDefaultsInput>,
) -> Response {
    respond_no_content(users::set_member_defaults(&state, &actor, payload.roles).await)
}

// ------------------------------------------------------ admin self-grant

/// `PUT /api/admin/me/groups` — a global user-manager may grant **themselves**
/// roles in any group; afterwards standard group policies apply.
pub async fn admin_self_grant(
    State(state): State<MyAppState>,
    actor: AppCtx,
    Json(payload): Json<AdminSelfGrantInput>,
) -> Response {
    respond_no_content(users::admin_self_grant(&state, &actor, payload).await)
}
