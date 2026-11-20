//! User and group management for the SPA (global permissions + group policies).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

use foodlib_new::{
    auth_context::AuthCtx,
    entities::user::{Group, GroupMembership, User},
    error::Error,
    ops::audit::{action, AuditRecord},
    policy::{all_definitions, GlobalPermission, GroupPerm, Permission},
};

use crate::api::auth::ApiGroup;
use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

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

/// Best-effort audit hook for security-relevant mutations; failures are
/// logged but never block the API response (the change already committed).
async fn audit(
    state: &MyAppState,
    ctx: &AuthCtx,
    action: &'static str,
    entity_type: Option<&str>,
    entity_id: Option<i32>,
    group_id: Option<i32>,
    target_user_id: Option<i64>,
    details: serde_json::Value,
) {
    if let Err(err) = state
        .audit()
        .record(AuditRecord {
            actor_user_id: Some(ctx.user.id),
            actor_username: &ctx.user.username,
            action,
            entity_type,
            entity_id,
            group_id,
            event_id: None,
            target_user_id,
            details: Some(details),
        })
        .await
    {
        log::error!("audit write failed: {err}");
    }
}

// ------------------------------------------------------------------ users

/// `GET /api/roles` — the role catalog. The SPA renders all role UIs
/// (pills, pickers, matrix legend) purely from this response.
async fn list_roles(_ctx: AuthCtx) -> Response {
    respond(Ok(all_definitions()))
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserWithGroups {
    #[serde(flatten)]
    pub member: ApiMember,
    pub global_permissions: Vec<GlobalPermission>,
    pub groups: Vec<GroupMembership>,
}

/// `GET /api/users` — users with group roles and global permissions.
async fn list_users(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<UserListQuery>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
        return response;
    }
    let rows = match state.users().list_users_detailed().await {
        Ok(rows) => rows,
        Err(err) => return map_api_error(err).into_response(),
    };

    let search = query.search.unwrap_or_default().to_lowercase();
    let items: Vec<UserWithGroups> = rows
        .into_iter()
        .filter(|(user, _, _)| {
            search.is_empty()
                || user.username.to_lowercase().contains(&search)
                || user.email.to_lowercase().contains(&search)
        })
        .map(|(user, groups, global_permissions)| UserWithGroups {
            member: ApiMember::from(user),
            global_permissions,
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
        Ok(user) => {
            audit(
                &state,
                &ctx,
                action::USER_UPDATED,
                None,
                None,
                None,
                Some(user.id),
                serde_json::json!({
                    "username": user.username,
                    "email": user.email,
                    "is_admin": user.is_admin,
                }),
            )
            .await;
            (StatusCode::OK, Json(serde_json::json!(ApiMember::from(user))))
                .into_response()
        }
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
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::USER_DELETED,
                None,
                None,
                None,
                Some(id),
                serde_json::json!({}),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct GlobalPermissionsInput {
    pub permissions: Vec<GlobalPermission>,
}

/// `PUT /api/users/{id}/global-permissions` — replaces the user's global
/// permissions (admin access drawer). Self-lockout is prevented by refusing
/// to drop `admin:manage_users` from the caller's own account.
async fn set_user_global_permissions(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i64>,
    Json(payload): Json<GlobalPermissionsInput>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
        return response;
    }
    if id == ctx.user.id && !payload.permissions.contains(&GlobalPermission::ManageUsers) {
        return bad_request("you cannot revoke your own admin:manage_users permission".into());
    }
    match state
        .users()
        .set_user_global_permissions(id, &payload.permissions)
        .await
    {
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::GLOBAL_PERMISSIONS_CHANGED,
                None,
                None,
                None,
                Some(id),
                serde_json::json!({ "permissions": payload.permissions }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
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
        Ok(_) => {
            audit(
                &state,
                &ctx,
                action::PASSWORD_CHANGED,
                None,
                None,
                None,
                Some(id),
                serde_json::json!({ "self_service": is_self }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
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

/// `GET /api/groups` — user managers see every group (with the session user's
/// own roles in it, possibly empty), regular members see their memberships.
/// Every entry carries `roles`, so the SPA can render them without falling
/// back to `/auth/me`.
async fn list_groups(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    let memberships = state.users().get_user_group_memberships(ctx.user.id).await;
    let result = match memberships {
        Ok(memberships) if ctx.can_global(GlobalPermission::ManageUsers) => {
            let mut by_id: HashMap<i32, GroupMembership> =
                memberships.into_iter().map(|m| (m.id, m)).collect();
            state
                .users()
                .get_all_groups()
                .await
                .map(|groups| {
                    groups
                        .into_iter()
                        .map(|group| {
                            by_id.remove(&group.id).unwrap_or(GroupMembership {
                                id: group.id,
                                name: group.name,
                                is_personal: group.is_personal,
                                roles: Vec::new(),
                                permissions: Vec::new(),
                            })
                        })
                        .map(ApiGroup::from_membership)
                        .collect::<Vec<ApiGroup>>()
                })
        }
        Ok(memberships) => Ok(memberships
            .into_iter()
            .map(ApiGroup::from_membership)
            .collect()),
        Err(err) => Err(err),
    };
    respond(result)
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
    match state
        .users()
        .create_group_with_manager(&name, ctx.user.id)
        .await
    {
        Ok((group, _creator)) => {
            audit(
                &state,
                &ctx,
                action::ENTITY_CREATED,
                Some("group"),
                Some(group.id),
                Some(group.id),
                None,
                serde_json::json!({ "name": group.name }),
            )
            .await;
            respond_created(Ok(group))
        }
        Err(err) => map_api_error(err).into_response(),
    }
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
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::ENTITY_DELETED,
                Some("group"),
                Some(id),
                Some(id),
                None,
                serde_json::json!({ "name": group.name }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
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
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::GROUP_MEMBER_ROLES_CHANGED,
                None,
                None,
                Some(id),
                Some(user_id),
                serde_json::json!({ "roles": roles }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
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

    let roles = match payload.roles {
        Some(roles) => roles,
        None => match state.users().resolve_member_default_roles(id).await {
            Ok(roles) => roles,
            Err(err) => return map_api_error(err).into_response(),
        },
    };
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    match state
        .users()
        .add_user_to_group_with_roles(user.id, id, &roles)
        .await
    {
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::GROUP_MEMBER_ADDED,
                None,
                None,
                Some(id),
                Some(user.id),
                serde_json::json!({ "roles": roles, "username": user.username }),
            )
            .await;
            (
                StatusCode::CREATED,
                Json(serde_json::json!(ApiMemberWithRoles {
                    member: ApiMember::from(user),
                    roles,
                })),
            )
                .into_response()
        }
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
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::GROUP_MEMBER_REMOVED,
                None,
                None,
                Some(id),
                Some(user_id),
                serde_json::json!({}),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------- group defaults

/// Effective default roles of a group as seen by the settings UI.
#[derive(Debug, Serialize)]
pub struct GroupDefaultRoles {
    /// Group-level setting; `None` = inherit the global default.
    pub roles: Option<Vec<String>>,
    /// What a new member would actually receive.
    pub resolved: Vec<String>,
    pub inherited: bool,
}

/// `GET /api/groups/{id}/default-roles` — guard `group:manage_settings`.
async fn get_group_default_roles(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_group_perm(id, Permission::Group(GroupPerm::ManageSettings)) {
        return map_api_error(err).into_response();
    }
    let roles = match state.users().get_group_default_roles(id).await {
        Ok(roles) => roles,
        Err(err) => return map_api_error(err).into_response(),
    };
    let resolved = match state.users().resolve_member_default_roles(id).await {
        Ok(resolved) => resolved,
        Err(err) => return map_api_error(err).into_response(),
    };
    respond(Ok(GroupDefaultRoles {
        inherited: roles.is_none(),
        roles,
        resolved,
    }))
}

#[derive(Debug, Deserialize)]
pub struct GroupDefaultRolesInput {
    pub roles: Vec<String>,
}

/// `PUT /api/groups/{id}/default-roles` — guard `group:manage_settings`.
async fn put_group_default_roles(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<GroupDefaultRolesInput>,
) -> Response {
    if let Err(err) = ctx.assert_group_perm(id, Permission::Group(GroupPerm::ManageSettings)) {
        return map_api_error(err).into_response();
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
        return bad_request("use DELETE to inherit the global default".into());
    }
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }
    match state.users().set_group_default_roles(id, &roles).await {
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::GROUP_DEFAULTS_CHANGED,
                Some("group"),
                Some(id),
                Some(id),
                None,
                serde_json::json!({ "roles": roles }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/groups/{id}/default-roles` — reset to "inherit the global
/// default". Guard `group:manage_settings`.
async fn clear_group_default_roles(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_group_perm(id, Permission::Group(GroupPerm::ManageSettings)) {
        return map_api_error(err).into_response();
    }
    match state.users().clear_group_default_roles(id).await {
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::GROUP_DEFAULTS_CHANGED,
                Some("group"),
                Some(id),
                Some(id),
                None,
                serde_json::json!({ "inherited": true }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------- system settings

/// `GET/PUT /api/settings/member-defaults` — global default roles for new
/// members of groups without their own default. Guard `admin:manage_settings`.
pub fn settings_router() -> Router<MyAppState> {
    Router::new().route(
        "/member-defaults",
        get(get_member_defaults).put(put_member_defaults),
    )
}

#[derive(Debug, Serialize)]
pub struct MemberDefaults {
    pub roles: Vec<String>,
    /// Organizational groups that override the global default.
    pub groups_overriding: i64,
}

async fn get_member_defaults(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageSettings) {
        return response;
    }
    let users = state.users();
    match tokio::try_join!(
        users.get_global_member_default_roles(),
        users.count_groups_with_own_defaults()
    ) {
        Ok((roles, groups_overriding)) => {
            respond(Ok(MemberDefaults { roles, groups_overriding }))
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct MemberDefaultsInput {
    pub roles: Vec<String>,
}

async fn put_member_defaults(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<MemberDefaultsInput>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageSettings) {
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
        return bad_request("at least one default role is required".into());
    }
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }
    match state.users().set_global_member_default_roles(&roles).await {
        Ok(()) => {
            audit(
                &state,
                &ctx,
                action::SETTINGS_MEMBER_DEFAULTS_CHANGED,
                None,
                None,
                None,
                None,
                serde_json::json!({ "roles": roles }),
            )
            .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

// ------------------------------------------------------- admin self-grant

#[derive(Debug, Deserialize)]
pub struct AdminSelfGrantInput {
    pub group_id: i32,
    /// Multi-role form; takes precedence over the legacy `role`.
    pub roles: Option<Vec<String>>,
    /// Legacy single-role form (kept for the maud admin UI).
    pub role: Option<String>,
}

/// `PUT /api/admin/me/groups` — a global user-manager may grant **themselves**
/// roles in any group; afterwards standard group policies apply.
pub async fn admin_self_grant(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<AdminSelfGrantInput>,
) -> Response {
    if let Err(response) = require_global(&ctx, GlobalPermission::ManageUsers) {
        return response;
    }
    let mut roles: Vec<String> = payload
        .roles
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    if let Some(role) = payload.role {
        roles.push(role.trim().to_string());
    }
    roles.sort();
    roles.dedup();

    if roles.is_empty() {
        return bad_request("at least one role is required".into());
    }
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    for role in &roles {
        match state
            .users()
            .grant_membership(ctx.user.id, payload.group_id, role)
            .await
        {
            Ok(()) => {}
            Err(err) => return map_api_error(err).into_response(),
        }
    }
    audit(
        &state,
        &ctx,
        action::ADMIN_SELF_GRANT,
        Some("group"),
        None,
        Some(payload.group_id),
        Some(ctx.user.id),
        serde_json::json!({ "roles": roles }),
    )
    .await;
    StatusCode::NO_CONTENT.into_response()
}