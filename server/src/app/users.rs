//! User, group and settings use-cases: account management, global
//! permissions, group policies, member management and system defaults.
//! Reading the user list requires `admin:manage_users`; every mutation is
//! audited.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::types::ids::{GroupId, UserId};
use foodlib_new::entities::user::{Group, GroupMembership, User};
use foodlib_new::ops::audit::action;
use foodlib_new::policy::{all_definitions, GlobalPermission, GroupPerm, Permission};

use crate::app::audit;
use crate::app::auth::ApiGroup;
use crate::app::lib::normalize_roles;
use crate::app::{AppCtx, Error, MyAppState, Result};
// ------------------------------------------------------------------ DTOs

/// Member projection as exposed to clients.
#[derive(Debug, Serialize)]
pub struct Member {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<User> for Member {
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

/// Member projection plus all assigned role slugs in the group.
#[derive(Debug, Serialize)]
pub struct MemberWithRoles {
    #[serde(flatten)]
    pub member: Member,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub search: Option<String>,
}

/// User manager view: member projection plus global permissions and group
/// memberships.
#[derive(Debug, Serialize)]
pub struct UserWithGroups {
    #[serde(flatten)]
    pub member: Member,
    pub global_permissions: Vec<GlobalPermission>,
    pub groups: Vec<GroupMembership>,
}

#[derive(Debug, Deserialize)]
pub struct UserUpdateInput {
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordChangeInput {
    /// Required when a user changes their own password.
    pub current_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct GlobalPermissionsInput {
    pub permissions: Vec<GlobalPermission>,
}

#[derive(Debug, Deserialize)]
pub struct GroupCreateInput {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SetMemberRolesInput {
    /// Non-empty list of role slugs (e.g. `["viewer"]` or `["editor", "kitchen"]`).
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberInput {
    /// Username, email or numeric id of the user to add.
    pub user: String,
    /// Optional initial role slugs (defaults to the group's defaults).
    pub roles: Option<Vec<String>>,
}

/// Effective default roles of a group as seen by the settings UI.
#[derive(Debug, Serialize)]
pub struct GroupDefaultRoles {
    /// Group-level setting; `None` = inherit the global default.
    pub roles: Option<Vec<String>>,
    /// What a new member would actually receive.
    pub resolved: Vec<String>,
    pub inherited: bool,
}

#[derive(Debug, Deserialize)]
pub struct GroupDefaultRolesInput {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MemberDefaults {
    pub roles: Vec<String>,
    /// Organizational groups that override the global default.
    pub groups_overriding: i64,
}

#[derive(Debug, Deserialize)]
pub struct AdminSelfGrantInput {
    pub group_id: GroupId,
    /// Multi-role form; takes precedence over the legacy `role`.
    pub roles: Option<Vec<String>>,
    /// Legacy single-role form (kept for the maud admin UI).
    pub role: Option<String>,
}

/// One catalog entry as served to the SPA / admin UI. Unlike the static
/// `RoleDefinition`, the permission set comes from the DB (`roles` /
/// `role_permissions`) — the DB is the source of truth since the role matrix
/// became editable.
#[derive(Debug, Serialize)]
pub struct RoleCatalogEntry {
    pub slug: String,
    pub name: String,
    pub description: String,
    /// `"basic"` or `"domain"` (render grouping).
    pub category: String,
    pub is_system: bool,
    pub permissions: Vec<Permission>,
}

/// The role catalog. The SPA renders all role UIs (pills, pickers, matrix
/// legend) purely from this response. Built-in roles contribute name and
/// description; their *permissions* (and all custom roles) come from the DB.
pub async fn role_catalog(state: &MyAppState) -> Result<Vec<RoleCatalogEntry>> {
    let db_roles: HashMap<String, foodlib_new::ops::users::DbRole> = state
        .users()
        .list_roles()
        .await?
        .into_iter()
        .map(|r| (r.slug.clone(), r))
        .collect();
    let mut db_by_slug = db_roles;
    let mut out = Vec::new();

    for def in all_definitions() {
        let (name, permissions, is_system) = match db_by_slug.remove(def.slug) {
            Some(row) => (row.name, row.permissions, row.is_system),
            // No DB row (e.g. legacy alias like `editor`): keep the built-in.
            None => (def.name.to_string(), def.permissions.clone(), true),
        };
        out.push(RoleCatalogEntry {
            slug: def.slug.to_string(),
            name,
            description: def.description.to_string(),
            category: match def.category {
                foodlib_new::policy::RoleCategory::Basic => "basic",
                foodlib_new::policy::RoleCategory::Domain => "domain",
            }
            .to_string(),
            is_system,
            permissions,
        });
    }

    // Custom roles not present in the built-in catalog.
    let mut custom: Vec<_> = db_by_slug.into_values().collect();
    custom.sort_by(|a, b| a.slug.cmp(&b.slug));
    for row in custom {
        out.push(RoleCatalogEntry {
            slug: row.slug,
            name: row.name,
            description: String::new(),
            category: "domain".to_string(),
            is_system: row.is_system,
            permissions: row.permissions,
        });
    }
    Ok(out)
}

// --------------------------------------------------------------- helpers

/// Validates the role slugs against the DB `roles` table.
/// Validates the role slugs against the DB `roles` table (the source of
/// truth since the role matrix became editable; falls back to the built-in
/// catalog for legacy aliases like `editor`).
async fn validate_roles(state: &MyAppState, roles: &[String]) -> Result<()> {
    let known: std::collections::HashSet<String> = state
        .users()
        .list_roles()
        .await?
        .into_iter()
        .map(|r| r.slug)
        .collect();
    if !roles
        .iter()
        .all(|slug| known.contains(slug) || foodlib_new::policy::BuiltInRole::from_slug(slug).is_some())
    {
        return Err(Error::Validation {
            message: "one or more unknown role slugs provided".into(),
        });
    }
    Ok(())
}

// ----------------------------------------------------------- role matrix

/// All roles with their DB-backed permission sets — the columns of the
/// legacy permission matrix. Public read: the role catalog is not secret.
pub async fn list_roles(
    state: &MyAppState,
) -> Result<Vec<foodlib_new::ops::users::DbRole>> {
    state.users().list_roles().await
}

/// Replaces one role's permission set (matrix column save).
/// Guard `admin:manage_settings`.
///
/// Safety: system roles must keep `group:manage_members` — otherwise editing
/// the `manager` column could lock every group out of member management.
pub async fn set_role_permissions(
    state: &MyAppState,
    actor: &AppCtx,
    slug: &str,
    permissions: Vec<Permission>,
) -> Result<()> {
    actor.ctx.assert_global(GlobalPermission::ManageSettings)?;

    let role = state
        .users()
        .list_roles()
        .await?
        .into_iter()
        .find(|r| r.slug == slug)
        .ok_or(Error::NotFound {
            entity: "Role",
            id: slug.to_string(),
        })?;
    // Only roles that currently carry `group:manage_members` must keep it:
    // the migration grants it to `manager`, not to every system role.
    if role.is_system
        && role.permissions.contains(&Permission::Group(GroupPerm::ManageMembers))
        && !permissions.contains(&Permission::Group(GroupPerm::ManageMembers))
    {
        return Err(Error::Validation {
            message: "system roles must keep group:manage_members".into(),
        });
    }

    state
        .users()
        .set_role_permissions(slug, &permissions)
        .await?;
    audit::record(
        state,
        actor,
        action::ROLE_PERMISSIONS_CHANGED,
        Some("role"),
        None,
        None,
        None,
        Some(json!({
            "slug": slug,
            "permissions": permissions
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
        })),
    )
        .await;
    Ok(())
}

/// Creates a custom (non-system) role. Guard `admin:manage_settings`.
pub async fn create_role(
    state: &MyAppState,
    actor: &AppCtx,
    slug: &str,
    name: &str,
) -> Result<()> {
    actor.ctx.assert_global(GlobalPermission::ManageSettings)?;
    let slug = slug.trim();
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::Validation {
            message: "role name must not be empty".into(),
        });
    }
    state.users().create_role(slug, name).await?;
    audit::record(
        state,
        actor,
        action::ROLE_CREATED,
        Some("role"),
        None,
        None,
        None,
        Some(json!({ "slug": slug, "name": name })),
    )
        .await;
    Ok(())
}

/// Deletes a custom, unused role. Guard `admin:manage_settings`.
pub async fn delete_role(state: &MyAppState, actor: &AppCtx, slug: &str) -> Result<()> {
    actor.ctx.assert_global(GlobalPermission::ManageSettings)?;
    state.users().delete_role(slug).await?;
    audit::record(
        state,
        actor,
        action::ROLE_DELETED,
        Some("role"),
        None,
        None,
        None,
        Some(json!({ "slug": slug })),
    )
        .await;
    Ok(())
}
/// Ensures the group exists, is NOT a personal workspace, and the caller
/// holds `group:manage_members` in that group.
pub async fn assert_group_member_manager(
    state: &MyAppState,
    ctx: &AuthCtx,
    group_id: GroupId,
) -> Result<Group> {
    let group = state.users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Validation {
            message: "personal groups are managed through the owning user account".into(),
        });
    }
    ctx.assert_group_perm(group_id, Permission::Group(GroupPerm::ManageMembers))?;
    Ok(group)
}

// ----------------------------------------------------------------- users

pub async fn list_filtered_users(
    state: &MyAppState,
    actor: &AppCtx,
    query: UserListQuery,
) -> Result<Vec<UserWithGroups>> {
    actor.ctx.assert_global(GlobalPermission::ViewUsers)?;
    let rows = state.users().list_users_detailed().await?;

    let search = query.search.unwrap_or_default().to_lowercase();
    Ok(rows
        .into_iter()
        .filter(|(user, _, _)| {
            search.is_empty()
                || user.username.to_lowercase().contains(&search)
                || user.email.to_lowercase().contains(&search)
        })
        .map(|(user, groups, global_permissions)| UserWithGroups {
            member: Member::from(user),
            global_permissions,
            groups,
        })
        .collect())
}

pub async fn update_user(
    state: &MyAppState,
    actor: &AppCtx,
    id: UserId,
    payload: UserUpdateInput,
) -> Result<Member> {
    actor.ctx.assert_global(GlobalPermission::ManageUsers)?;
    let current = state.users().get_user(id).await?;
    if current.id == actor.ctx.user.id && payload.is_admin == Some(false) {
        return Err(Error::Validation {
            message: "you cannot revoke your own admin privileges".into(),
        });
    }

    let updated = User {
        id: current.id,
        username: payload.username.unwrap_or(current.username),
        email: payload.email.unwrap_or(current.email),
        password_hash: current.password_hash,
        is_admin: payload.is_admin.unwrap_or(current.is_admin),
        created_at: current.created_at,
    };
    let user = state.users().update_user(updated).await?;
    audit::record(
        state,
        actor,
        action::USER_UPDATED,
        None,
        None,
        None,
        None,
        Some(json!({
            "username": user.username,
            "email": user.email,
            "is_admin": user.is_admin,
        })),
    )
        .await;
    Ok(Member::from(user))
}

/// cascade delete; refuses deleting the caller's
/// own account here.
pub async fn delete_user(state: &MyAppState, actor: &AppCtx, id: UserId) -> Result<()> {
    actor.ctx.assert_global(GlobalPermission::ManageUsers)?;
    if id == actor.ctx.user.id {
        return Err(Error::Validation {
            message: "you cannot delete your own account here".into(),
        });
    }
    state.users().delete_user_cascade(id).await?;
    audit::record(state, actor, action::USER_DELETED, None, None, None, None, Some(json!({})))
        .await;
    Ok(())
}

/// self-service (with current password) or
/// user-manager reset (without).
pub async fn change_password(
    state: &MyAppState,
    actor: &AppCtx,
    id: UserId,
    payload: PasswordChangeInput,
) -> Result<()> {
    let is_self = actor.ctx.user.id == id;
    if !is_self {
        actor.ctx.assert_global(GlobalPermission::ManageUsers)?;
    }

    if payload.new_password.len() < 8 {
        return Err(Error::Validation {
            message: "password must be at least 8 characters".into(),
        });
    }
    if is_self {
        let current_password = payload.current_password.as_deref().ok_or(Error::Validation {
            message: "current_password is required to change your own password".into(),
        })?;
        if !current_password.is_empty()
            && !bcrypt::verify(current_password, &actor.ctx.user.password_hash).unwrap_or(false)
        {
            return Err(Error::Validation {
                message: "current password is incorrect".into(),
            });
        }
    }

    state.users().update_password(id, payload.new_password).await?;
    audit::record(
        state,
        actor,
        action::PASSWORD_CHANGED,
        None,
        None,
        None,
        None,
        Some(json!({ "self_service": is_self })),
    )
        .await;
    Ok(())
}

/// `PUT /api/users/{id}/global-permissions` — replaces the user's global
/// permissions (admin access drawer). Self-lockout is prevented by refusing
/// to drop `admin:manage_users` from the caller's own account.
pub async fn set_global_permissions(
    state: &MyAppState,
    actor: &AppCtx,
    id: UserId,
    payload: GlobalPermissionsInput,
) -> Result<()> {
    actor.ctx.assert_global(GlobalPermission::ManageUsers)?;
    if id == actor.ctx.user.id && !payload.permissions.contains(&GlobalPermission::ManageUsers) {
        return Err(Error::Validation {
            message: "you cannot revoke your own admin:manage_users permission".into(),
        });
    }
    state
        .users()
        .set_user_global_permissions(id, &payload.permissions)
        .await?;
    audit::record(
        state,
        actor,
        action::GLOBAL_PERMISSIONS_CHANGED,
        None,
        None,
        None,
        None,
        Some(json!({ "permissions": payload.permissions })),
    )
        .await;
    Ok(())
}

// ---------------------------------------------------------------- groups

/// user managers see every group (with the session user's
/// own roles in it, possibly empty), regular members see their memberships.
/// Every entry carries `roles`, so the SPA can render them without falling
/// back to `/auth/me`.
pub async fn list_groups(state: &MyAppState, actor: &AppCtx) -> Result<Vec<ApiGroup>> {
    let memberships = state
        .users()
        .get_user_group_memberships(actor.ctx.user.id)
        .await?;
    if actor.ctx.can_global(GlobalPermission::ManageUsers) {
        let mut by_id: HashMap<GroupId, GroupMembership> =
            memberships.into_iter().map(|m| (m.id, m)).collect();
        let groups = state.users().get_all_groups().await?;
        Ok(groups
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
            .collect())
    } else {
        Ok(memberships
            .into_iter()
            .map(ApiGroup::from_membership)
            .collect())
    }
}

/// create a non-personal group and assign the creator
/// as `manager`. Guard `user:create_resources` (`GlobalPermission::CreateResources`).
pub async fn create_group(
    state: &MyAppState,
    actor: &AppCtx,
    input: GroupCreateInput,
) -> Result<Group> {
    actor
        .ctx
        .assert_global(GlobalPermission::CreateResources)?;
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(Error::Validation {
            message: "group name must not be empty".into(),
        });
    }
    let exists = state
        .users()
        .get_all_groups()
        .await?
        .iter()
        .any(|group| group.name == name);
    if exists {
        return Err(Error::Validation {
            message: format!("group name already taken: {name}"),
        });
    }

    // `create_group_with_manager(name, user_id)` creates the group and inserts
    // `(user_id, group_id, "manager")` into `user_group_roles` in one tx.
    let (group, _creator) = state
        .users()
        .create_group_with_manager(&name, actor.ctx.user.id)
        .await?;
    audit::record(
        state,
        actor,
        action::ENTITY_CREATED,
        Some("group"),
        Some(group.id.into()),
        Some(group.id),
        None,
        Some(json!({ "name": group.name })),
    )
        .await;
    Ok(group)
}

/// requires `group:delete` on the group; refuses
/// personal and non-empty groups.
pub async fn delete_group(state: &MyAppState, actor: &AppCtx, id: GroupId) -> Result<()> {
    actor
        .ctx
        .assert_group_perm(id, Permission::Group(GroupPerm::Delete))?;
    let group = state.users().get_group(id).await?;
    if group.is_personal {
        return Err(Error::Validation {
            message: "personal groups are deleted with their owning user".into(),
        });
    }
    let count = state.users().count_owned_entities(id).await?;
    if count > 0 {
        return Err(Error::Validation {
            message: format!("group still owns {count} entities — reassign or delete them first"),
        });
    }
    state.users().delete_group(id).await?;
    audit::record(
        state,
        actor,
        action::ENTITY_DELETED,
        Some("group"),
        Some(id.into()),
        Some(id),
        None,
        Some(json!({ "name": group.name })),
    )
        .await;
    Ok(())
}

// ---------------------------------------------------------------- members

/// members (or user admins) see member list
/// with all assigned role slugs.
pub async fn list_members(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<Vec<MemberWithRoles>> {
    if !actor.ctx.can_global(GlobalPermission::ManageUsers) && !actor.ctx.can_access_group(group_id)
    {
        return Err(Error::Forbidden("you are not a member of this group".into()));
    }
    let members = state
        .users()
        .get_group_members_with_roles(group_id)
        .await?;
    Ok(members
        .into_iter()
        .map(|(user, roles)| MemberWithRoles {
            member: Member::from(user),
            roles,
        })
        .collect())
}

/// adds a user to the group with initial
/// roles (defaults to the group's default roles).
pub async fn add_member(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
    input: AddMemberInput,
) -> Result<MemberWithRoles> {
    assert_group_member_manager(state, &actor.ctx, group_id).await?;

    let user = state
        .users()
        .get_user_by_string_reference(input.user.trim().to_string())
        .await
        .ok_or_else(|| Error::Validation {
            message: format!("unknown user: {}", input.user),
        })?;

    let roles = match input.roles {
        Some(roles) => roles,
        None => state.users().resolve_member_default_roles(group_id).await?,
    };
    validate_roles(state, &roles).await?;

    state
        .users()
        .add_user_to_group_with_roles(user.id, group_id, &roles)
        .await?;
    audit::record(
        state,
        actor,
        action::GROUP_MEMBER_ADDED,
        None,
        None,
        Some(group_id),
        None,
        Some(json!({ "roles": roles, "username": user.username })),
    )
        .await;
    Ok(MemberWithRoles {
        member: Member::from(user),
        roles,
    })
}

/// replaces the member's role set.
/// Refuses if the change would leave the group with zero members holding
/// `group:manage_members`.
pub async fn set_member_roles(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
    user_id: UserId,
    input: SetMemberRolesInput,
) -> Result<()> {
    assert_group_member_manager(state, &actor.ctx, group_id).await?;

    let roles = normalize_roles(input.roles);
    if roles.is_empty() {
        return Err(Error::Validation {
            message: "a group member must have at least one role".into(),
        });
    }
    validate_roles(state, &roles).await?;

    // Ensure at least one member in the group retains `group:manage_members`
    if state
        .users()
        .would_leave_group_without_permission(
            group_id,
            user_id,
            &roles,
            Permission::Group(GroupPerm::ManageMembers),
        )
        .await?
    {
        return Err(Error::Validation {
            message: "cannot remove the last member able to manage group members".into(),
        });
    }

    state
        .users()
        .set_member_roles(user_id, group_id, &roles)
        .await?;
    audit::record(
        state,
        actor,
        action::GROUP_MEMBER_ROLES_CHANGED,
        None,
        None,
        Some(group_id),
        None,
        Some(json!({ "roles": roles })),
    )
        .await;
    Ok(())
}

/// refuses to remove the last
/// member holding `group:manage_members`.
pub async fn remove_member(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
    user_id: UserId,
) -> Result<()> {
    assert_group_member_manager(state, &actor.ctx, group_id).await?;

    if state
        .users()
        .would_leave_group_without_permission(
            group_id,
            user_id,
            &[],
            Permission::Group(GroupPerm::ManageMembers),
        )
        .await?
    {
        return Err(Error::Validation {
            message: "cannot remove the last member able to manage group members".into(),
        });
    }

    state
        .users()
        .remove_user_from_group(user_id, group_id)
        .await?;
    audit::record(
        state,
        actor,
        action::GROUP_MEMBER_REMOVED,
        None,
        None,
        Some(group_id),
        None,
        Some(json!({})),
    )
        .await;
    Ok(())
}

// ---------------------------------------------------------- group defaults

/// `GET /api/groups/{id}/default-roles` — guard `group:manage_settings`.
pub async fn get_default_roles(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<GroupDefaultRoles> {
    actor
        .ctx
        .assert_group_perm(group_id, Permission::Group(GroupPerm::ManageSettings))?;
    let roles = state.users().get_group_default_roles(group_id).await?;
    let resolved = state.users().resolve_member_default_roles(group_id).await?;
    Ok(GroupDefaultRoles {
        inherited: roles.is_none(),
        roles,
        resolved,
    })
}

/// `PUT /api/groups/{id}/default-roles` — guard `group:manage_settings`.
pub async fn put_default_roles(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
    input: GroupDefaultRolesInput,
) -> Result<()> {
    actor
        .ctx
        .assert_group_perm(group_id, Permission::Group(GroupPerm::ManageSettings))?;
    let roles = normalize_roles(input.roles);
    if roles.is_empty() {
        return Err(Error::Validation {
            message: "use DELETE to inherit the global default".into(),
        });
    }
    validate_roles(state, &roles).await?;
    state
        .users()
        .set_group_default_roles(group_id, &roles)
        .await?;
    audit::record(
        state,
        actor,
        action::GROUP_DEFAULTS_CHANGED,
        Some("group"),
        Some(group_id.into()),
        Some(group_id),
        None,
        Some(json!({ "roles": roles })),
    )
        .await;
    Ok(())
}

/// `DELETE /api/groups/{id}/default-roles` — reset to "inherit the global
/// default". Guard `group:manage_settings`.
pub async fn clear_default_roles(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<()> {
    actor
        .ctx
        .assert_group_perm(group_id, Permission::Group(GroupPerm::ManageSettings))?;
    state.users().clear_group_default_roles(group_id).await?;
    audit::record(
        state,
        actor,
        action::GROUP_DEFAULTS_CHANGED,
        Some("group"),
        Some(group_id.into()),
        Some(group_id),
        None,
        Some(json!({ "inherited": true })),
    )
        .await;
    Ok(())
}

// --------------------------------------------------------- system settings

/// `GET /api/settings/member-defaults` — global default roles for new members
/// of groups without their own default. Guard `admin:manage_settings`.
pub async fn get_member_defaults(state: &MyAppState, actor: &AppCtx) -> Result<MemberDefaults> {
    actor.ctx.assert_global(GlobalPermission::ManageSettings)?;
    let users = state.users();
    let (roles, groups_overriding) = tokio::try_join!(
        users.get_global_member_default_roles(),
        users.count_groups_with_own_defaults()
    )?;
    Ok(MemberDefaults {
        roles,
        groups_overriding,
    })
}

/// `PUT /api/settings/member-defaults` — guard `admin:manage_settings`.
pub async fn set_member_defaults(
    state: &MyAppState,
    actor: &AppCtx,
    roles: Vec<String>,
) -> Result<()> {
    actor.ctx.assert_global(GlobalPermission::ManageSettings)?;
    let roles = normalize_roles(roles);
    if roles.is_empty() {
        return Err(Error::Validation {
            message: "at least one default role is required".into(),
        });
    }
    validate_roles(state, &roles).await?;
    state.users().set_global_member_default_roles(&roles).await?;
    audit::record(
        state,
        actor,
        action::SETTINGS_MEMBER_DEFAULTS_CHANGED,
        None,
        None,
        None,
        None,
        Some(json!({ "roles": roles })),
    )
        .await;
    Ok(())
}

// -------------------------------------------------------- admin self-grant

/// a global user-manager may grant
/// **themselves** roles in any group; afterwards standard group policies
/// apply. Guard `admin:manage_users`.
pub async fn admin_self_grant(
    state: &MyAppState,
    actor: &AppCtx,
    input: AdminSelfGrantInput,
) -> Result<()> {
    let ctx = &actor.ctx;
    ctx.assert_global(GlobalPermission::ManageUsers)?;

    let mut roles = normalize_roles(input.roles.unwrap_or_default());
    if let Some(role) = input.role {
        roles.push(role.trim().to_string());
        roles = normalize_roles(roles);
    }
    if roles.is_empty() {
        return Err(Error::Validation {
            message: "at least one role is required".into(),
        });
    }
    validate_roles(state, &roles).await?;

    for role in &roles {
        state
            .users()
            .grant_membership(ctx.user.id, input.group_id, role)
            .await?;
    }
    audit::record(
        state,
        actor,
        action::ADMIN_SELF_GRANT,
        Some("group"),
        Some(input.group_id.into()),
        Some(input.group_id),
        None,
        Some(json!({ "roles": roles })),
    )
        .await;
    Ok(())
}