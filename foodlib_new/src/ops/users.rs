use crate::entities::types::ids::{EventId, GroupId, UserId};
use crate::policy::{permissions_for_role_slug, BuiltInRole, GlobalPermission, Permission};
use crate::{
    entities::user::*,
    error::{Error, Result},
    policy::UserPolicy,
};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct UserOps {
    pool: Arc<PgPool>,
}

/// One role as stored in the DB, with its resolved permission set.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DbRole {
    pub slug: String,
    pub name: String,
    pub is_system: bool,
    pub permissions: Vec<Permission>,
}

impl UserOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: User) -> Result<User> {
        let mut tx = self.pool.begin().await?;


        let row = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash, is_admin, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id as "id: UserId", username, email, password_hash, is_admin, created_at
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.is_admin,
            user.created_at
        )
            .fetch_one(&mut *tx)
            .await?;

        for perm in GlobalPermission::DEFAULT_REGISTERED {
                sqlx::query!(
            r#"
            INSERT INTO user_global_permissions (user_id, permission)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            user.id.get(),
            perm.as_str(),
        )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(row)
    }

    pub async fn update_user(&self, user: User) -> Result<User> {
        let row = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET username = $1, email = $2, password_hash = $3, is_admin = $4
            WHERE id = $5
            RETURNING id as "id: UserId", username, email, password_hash, is_admin, created_at
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.is_admin,
            user.id.get()
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(row)
    }

    pub async fn update_password(&self, id: UserId, password: String) -> Result<User> {
        let password_hash = bcrypt::hash(password, 12)?;

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET password_hash = $1
            WHERE id = $2
            RETURNING id as "id: UserId", username, email, password_hash, is_admin, created_at
            "#,
            password_hash,
            id.get()
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(user)
    }

    pub async fn delete_user(&self, id: UserId) -> Result<()> {
        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id.get())
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Deletes a user and their personal group, plus all their group memberships.
    /// Refuses if the user's personal group still owns any recipes, ingredients,
    /// inventories, or events.
    pub async fn delete_user_cascade(&self, id: UserId) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let personal_group_id = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT g.id
            FROM user_group_roles ugr
            JOIN groups g ON g.id = ugr.group_id
            WHERE ugr.user_id = $1 AND g.is_personal = TRUE
            "#,
            id.get()
        )
            .fetch_optional(&mut *tx)
            .await?;

        if let Some(group_id) = personal_group_id {
            let owned_count = sqlx::query_scalar!(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM recipes WHERE group_id = $1)
                  + (SELECT COUNT(*) FROM ingredients WHERE group_id = $1)
                  + (SELECT COUNT(*) FROM inventories WHERE group_id = $1)
                  + (SELECT COUNT(*) FROM events WHERE group_id = $1)
                "#,
                group_id
            )
                .fetch_one(&mut *tx)
                .await?
                .unwrap_or(0);

            if owned_count > 0 {
                return Err(Error::Conflict {
                    message: format!(
                        "Cannot delete user: their personal group still owns {owned_count} entities. \
                         Reassign or delete them first."
                    ),
                });
            }
        }
        sqlx::query!(r#"DELETE FROM user_group_roles WHERE user_id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        if let Some(group_id) = personal_group_id {
            sqlx::query!(r#"DELETE FROM groups WHERE id = $1"#, group_id)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_user(&self, id: UserId) -> Result<User> {
        let row = sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: UserId", username, email, password_hash, is_admin, created_at
            FROM users
            WHERE id = $1
            "#,
            id.get()
        )
            .fetch_optional(&*self.pool)
            .await?;

        row.ok_or(Error::UserNotFound {
            name: id.to_string(),
        })
    }

    pub async fn get_user_by_string_reference(&self, reference: String) -> Option<User> {
        let id = reference.parse::<i64>().unwrap_or(-1);

        sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: UserId", username, email, password_hash, is_admin, created_at
            FROM users
            WHERE id = $1 OR username = $2 OR email = $2
            "#,
            id,
            reference
        )
            .fetch_one(&*self.pool)
            .await
            .ok()
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: UserId", username, email, password_hash, is_admin, created_at
            FROM users
            ORDER BY username
            "#
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    /// All users together with their distinct group memberships.
    pub async fn list_users_with_groups(&self) -> Result<Vec<(User, Vec<Group>)>> {
        let users = self.get_all_users().await?;
        let memberships = sqlx::query_as!(
            MembershipRow,
            r#"
            SELECT DISTINCT ugr.user_id as "user_id: UserId", g.id as "id: GroupId", g.name, g.is_personal
            FROM user_group_roles ugr
            JOIN groups g ON g.id = ugr.group_id
            ORDER BY g.is_personal DESC, g.name
            "#
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut groups_by_user: HashMap<UserId, Vec<Group>> = HashMap::new();
        for row in memberships {
            groups_by_user
                .entry(row.user_id)
                .or_default()
                .push(Group {
                    id: row.id,
                    name: row.name,
                    is_personal: row.is_personal,
                });
        }

        Ok(users
            .into_iter()
            .map(|user| {
                let groups = groups_by_user.remove(&user.id).unwrap_or_default();
                (user, groups)
            })
            .collect())
    }

    /// All distinct members of a group.
    pub async fn get_group_members(&self, group_id: GroupId) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            User,
            r#"
            SELECT DISTINCT u.id as "id: UserId", u.username, u.email, u.password_hash, u.is_admin, u.created_at
            FROM user_group_roles ugr
            JOIN users u ON u.id = ugr.user_id
            WHERE ugr.group_id = $1
            ORDER BY u.username
            "#,
            group_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    /// Creates a user and their personal group atomically.
    pub async fn create_user_with_personal_group(&self, user: User) -> Result<(User, Group)> {
        let mut tx = self.pool.begin().await?;

        let created = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash, is_admin, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id as "id: UserId", username, email, password_hash, is_admin, created_at
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.is_admin,
            user.created_at
        )
            .fetch_one(&mut *tx)
            .await?;

        let group = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (name, is_personal)
            VALUES ($1, TRUE)
            RETURNING id as "id: GroupId", name, is_personal
            "#,
            created.username
        )
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_group_roles (user_id, group_id, role_slug)
            VALUES ($1, $2, 'manager')
            "#,
            created.id.get(),
            group.id.get()
        )
            .execute(&mut *tx)
            .await?;

        for perm in GlobalPermission::DEFAULT_REGISTERED {
            sqlx::query!(
        r#"
        INSERT INTO user_global_permissions (user_id, permission)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        user.id.get(),
        perm.as_str(),
    )
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok((created, group))
    }

    /// Creates a group and assigns the user as `manager` atomically.
    pub async fn create_group_with_manager(
        &self,
        name: &str,
        user_id: UserId,
    ) -> Result<(Group, User)> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id as "id: UserId", username, email, password_hash, is_admin, created_at
            FROM users
            WHERE id = $1
            "#,
            user_id.get()
        )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(Error::UserNotFound {
                name: user_id.to_string(),
            })?;

        let group = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (name, is_personal)
            VALUES ($1, FALSE)
            RETURNING id as "id: GroupId", name, is_personal
            "#,
            name
        )
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query!(
            r#"
            INSERT INTO user_group_roles (user_id, group_id, role_slug)
            VALUES ($1, $2, 'manager')
            "#,
            user_id.get(),
            group.id.get()
        )
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok((group, user))
    }

    pub async fn create_group(&self, name: &str) -> Result<Group> {
        let row = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (name, is_personal)
            VALUES ($1, FALSE)
            RETURNING id as "id: GroupId", name, is_personal
            "#,
            name
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(row)
    }

    /// Adds a user to a group with the given initial roles.
    pub async fn add_user_to_group_with_roles(
        &self,
        user_id: UserId,
        group_id: GroupId,
        roles: &[String],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for role in roles {
            sqlx::query!(
                r#"
                INSERT INTO user_group_roles (user_id, group_id, role_slug)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                "#,
                user_id.get(),
                group_id.get(),
                role
            )
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Grants a single role to a user in a group (used by admin self-grant).
    pub async fn grant_membership(&self, user_id: UserId, group_id: GroupId, role: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_group_roles (user_id, group_id, role_slug)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            user_id.get(),
            group_id.get(),
            role
        )
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Replaces the complete set of roles for a user in a group.
    pub async fn set_member_roles(
        &self,
        user_id: UserId,
        group_id: GroupId,
        roles: &[String],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"DELETE FROM user_group_roles WHERE user_id = $1 AND group_id = $2"#,
            user_id.get(),
            group_id.get()
        )
            .execute(&mut *tx)
            .await?;

        for role in roles {
            sqlx::query!(
                r#"
                INSERT INTO user_group_roles (user_id, group_id, role_slug)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                "#,
                user_id.get(),
                group_id.get(),
                role
            )
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn remove_user_from_group(&self, user_id: UserId, group_id: GroupId) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM user_group_roles WHERE user_id = $1 AND group_id = $2"#,
            user_id.get(),
            group_id.get()
        )
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_user_groups(&self, user_id: UserId) -> Result<Vec<Group>> {
        let rows = sqlx::query_as!(
            Group,
            r#"
            SELECT DISTINCT g.id as "id: GroupId", g.name, g.is_personal
            FROM user_group_roles ugr
            JOIN groups g ON ugr.group_id = g.id
            WHERE ugr.user_id = $1
            ORDER BY g.is_personal DESC, g.name
            "#,
            user_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn list_groups(&self) -> Result<Vec<Group>> {
        let groups = sqlx::query_as!(
            Group,
            r#"SELECT id as "id: GroupId", name, is_personal FROM groups ORDER BY name"#
        )
            .fetch_all(&*self.pool)
            .await?;
        Ok(groups)
    }

    pub async fn get_personal_group(&self, user_id: UserId) -> Result<Group> {
        sqlx::query_as!(
            Group,
            r#"
            SELECT DISTINCT g.id as "id: GroupId", g.name, g.is_personal
            FROM user_group_roles ugr
            JOIN groups g ON ugr.group_id = g.id
            WHERE ugr.user_id = $1 AND g.is_personal = TRUE
            "#,
            user_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "personal group",
                id: user_id.to_string(),
            })
    }

    pub async fn get_group(&self, id: GroupId) -> Result<Group> {
        let row = sqlx::query_as!(
            Group,
            r#"SELECT id as "id: GroupId", name, is_personal FROM groups WHERE id = $1"#,
            id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Group",
                id: id.to_string(),
            })?;

        Ok(row)
    }

    pub async fn count_owned_entities(&self, group_id: GroupId) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM recipes WHERE group_id = $1)
              + (SELECT COUNT(*) FROM ingredients WHERE group_id = $1)
              + (SELECT COUNT(*) FROM inventories WHERE group_id = $1)
              + (SELECT COUNT(*) FROM events WHERE group_id = $1)
            "#,
            group_id as _
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(count.unwrap_or(0))
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let rows = sqlx::query_as!(
            Group,
            r#"SELECT id as "id: GroupId", name, is_personal FROM groups ORDER BY is_personal DESC, name"#
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn delete_group(&self, id: GroupId) -> Result<()> {
        sqlx::query!(r#"DELETE FROM user_group_roles WHERE group_id = $1"#, id.get())
            .execute(&*self.pool)
            .await?;
        sqlx::query!(r#"DELETE FROM groups WHERE id = $1"#, id.get())
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------------------
    // Multi-role & RBAC Policy Queries
    // ------------------------------------------------------------------------

    /// Members of a group together with all their assigned role slugs.
    pub async fn get_group_members_with_roles(
        &self,
        group_id: GroupId,
    ) -> Result<Vec<(User, Vec<String>)>> {
        let members = self.get_group_members(group_id).await?;
        let role_rows = sqlx::query!(
            r#"
            SELECT user_id as "user_id: UserId", role_slug
            FROM user_group_roles
            WHERE group_id = $1
            ORDER BY role_slug
            "#,
            group_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut roles_by_user: HashMap<UserId, Vec<String>> = HashMap::new();
        for row in role_rows {
            roles_by_user
                .entry(row.user_id)
                .or_default()
                .push(row.role_slug);
        }

        Ok(members
            .into_iter()
            .map(|member| {
                let roles = roles_by_user.remove(&member.id).unwrap_or_default();
                (member, roles)
            })
            .collect())
    }

    /// Verifies that every provided role slug is a known built-in role.
    pub fn validate_role_slugs(&self, slugs: &[String]) -> bool {
        slugs.iter().all(|slug| BuiltInRole::from_slug(slug).is_some())
    }

    // ------------------------------------------------------ db-backed roles
    //
    // Since the `roles`/`role_permissions` migration the DATABASE is the
    // source of truth for role→permission assignments; `BuiltInRole` remains
    // only a fallback for legacy slugs that have no row (e.g. 'editor').

    /// All roles with their permissions, for the permission-management matrix
    /// and role pickers. System roles first, then custom roles by slug.
    pub async fn list_roles(&self) -> Result<Vec<DbRole>> {
        let role_rows = sqlx::query!(
            r#"SELECT slug, name, is_system FROM roles ORDER BY is_system DESC, slug"#
        )
        .fetch_all(&*self.pool)
        .await?;

        let perm_rows =
            sqlx::query!(r#"SELECT role_slug, permission FROM role_permissions"#)
                .fetch_all(&*self.pool)
                .await?;
        let mut perms_by_role: HashMap<String, Vec<Permission>> = HashMap::new();
        for row in perm_rows {
            if let Ok(p) = row.permission.parse::<Permission>() {
                perms_by_role.entry(row.role_slug).or_default().push(p);
            }
        }

        Ok(role_rows
            .into_iter()
            .map(|r| DbRole {
                permissions: perms_by_role.remove(&r.slug).unwrap_or_default(),
                slug: r.slug,
                name: r.name,
                is_system: r.is_system,
            })
            .collect())
    }

    /// Resolves the permissions of multiple role slugs in two queries.
    /// Slugs that have no `roles` row fall back to the hardcoded
    /// `BuiltInRole` catalog (legacy aliases like `editor`).
    pub async fn permissions_for_roles(
        &self,
        slugs: &[String],
    ) -> Result<HashMap<String, Vec<Permission>>> {
        if slugs.is_empty() {
            return Ok(HashMap::new());
        }

        let existing = sqlx::query_scalar!(
            r#"SELECT slug FROM roles WHERE slug = ANY($1)"#,
            slugs as &[String]
        )
        .fetch_all(&*self.pool)
        .await?;

        let perm_rows = sqlx::query!(
            r#"SELECT role_slug, permission FROM role_permissions WHERE role_slug = ANY($1)"#,
            slugs as &[String]
        )
        .fetch_all(&*self.pool)
        .await?;
        let mut map: HashMap<String, Vec<Permission>> = HashMap::new();
        for row in perm_rows {
            if let Ok(p) = row.permission.parse::<Permission>() {
                map.entry(row.role_slug).or_default().push(p);
            }
        }

        // Fallback for legacy slugs without a roles row.
        for slug in slugs {
            if !existing.iter().any(|s| s == slug) {
                map.insert(slug.clone(), permissions_for_role_slug(slug).to_vec());
            }
        }
        Ok(map)
    }

    /// Replaces the permission set of one role (matrix save).
    pub async fn set_role_permissions(
        &self,
        slug: &str,
        permissions: &[Permission],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(r#"DELETE FROM role_permissions WHERE role_slug = $1"#, slug)
            .execute(&mut *tx)
            .await?;
        for permission in permissions {
            sqlx::query!(
                r#"INSERT INTO role_permissions (role_slug, permission) VALUES ($1, $2)
                   ON CONFLICT DO NOTHING"#,
                slug,
                permission.as_str()
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Creates a custom (non-system) role. The slug is restricted to
    /// lowercase ASCII letters, digits, `_` and `-` so it stays URL- and
    /// HTMX-safe.
    pub async fn create_role(&self, slug: &str, name: &str) -> Result<()> {
        let valid = !slug.is_empty()
            && slug.len() <= 32
            && slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        if !valid {
            return Err(Error::Validation {
                message: "role slug must be 1-32 chars of a-z, 0-9, '_' or '-'".into(),
            });
        }
        sqlx::query!(r#"INSERT INTO roles (slug, name, is_system) VALUES ($1, $2, FALSE)"#,
            slug, name)
            .execute(&*self.pool)
            .await
            .map(|_| ())
            .map_err(|e| {
                if e.as_database_error().map_or(false, |d| d.is_unique_violation()) {
                    Error::Validation {
                        message: format!("role slug already exists: {slug}"),
                    }
                } else {
                    e.into()
                }
            })
    }

    /// How many memberships reference the role (deletion guard).
    pub async fn role_usage_count(&self, slug: &str) -> Result<i64> {
        let users = sqlx::query_scalar!(
            r#"SELECT count(*) FROM user_group_roles WHERE role_slug = $1"#,
            slug
        )
        .fetch_one(&*self.pool)
        .await?
        .unwrap_or(0);
        let events = sqlx::query_scalar!(
            r#"SELECT count(*) FROM event_user_roles WHERE role_slug = $1"#,
            slug
        )
        .fetch_one(&*self.pool)
        .await?
        .unwrap_or(0);
        Ok(users + events)
    }

    /// Deletes a custom role. System roles and roles still in use are
    /// refused.
    pub async fn delete_role(&self, slug: &str) -> Result<()> {
        let is_system = sqlx::query_scalar!(r#"SELECT is_system FROM roles WHERE slug = $1"#, slug)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Role",
                id: slug.to_string(),
            })?;
        if is_system {
            return Err(Error::Validation {
                message: "system roles cannot be deleted".into(),
            });
        }
        let usage = self.role_usage_count(slug).await?;
        if usage > 0 {
            return Err(Error::Validation {
                message: format!("role is still assigned to {usage} membership(s)"),
            });
        }
        sqlx::query!(r#"DELETE FROM roles WHERE slug = $1"#, slug)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }


    /// Replaces a user's global permissions (admin access drawer).
    /// The self-lockout guard lives in the API layer (needs the caller).
    pub async fn set_user_global_permissions(
        &self,
        user_id: UserId,
        permissions: &Vec<GlobalPermission>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            r#"DELETE FROM user_global_permissions WHERE user_id = $1"#,
            user_id.get()
        )
            .execute(&mut *tx)
            .await?;
        for permission in permissions {
            sqlx::query!(
                r#"
                INSERT INTO user_global_permissions (user_id, permission)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
                user_id.get(),
                permission.as_str()
            )
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// All users with their per-group roles and global permissions in three
    /// flat queries (admin user list, no N+1).
    pub async fn list_users_detailed(
        &self,
    ) -> Result<Vec<(User, Vec<GroupMembership>, Vec<GlobalPermission>)>> {
        let users = self.get_all_users().await?;

        let global_rows = sqlx::query!(
            r#"SELECT user_id as "user_id: UserId", permission FROM user_global_permissions"#
        )
            .fetch_all(&*self.pool)
            .await?;
        let mut globals_by_user: HashMap<UserId, Vec<GlobalPermission>> = HashMap::new();
        for row in global_rows {
            if let Ok(permission) = row.permission.parse::<GlobalPermission>() {
                globals_by_user
                    .entry(row.user_id)
                    .or_default()
                    .push(permission);
            }
        }

        let membership_rows = sqlx::query!(
            r#"
            SELECT ugr.user_id as "user_id: UserId", g.id as "id: GroupId", g.name, g.is_personal, ugr.role_slug
            FROM user_group_roles ugr
            JOIN groups g ON g.id = ugr.group_id
            ORDER BY ugr.user_id, g.is_personal DESC, g.name, ugr.role_slug
            "#
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut memberships_by_user: HashMap<UserId, Vec<GroupMembership>> = HashMap::new();
        let role_slugs: Vec<String> =
            membership_rows.iter().map(|r| r.role_slug.clone()).collect();
        let role_perms = self.permissions_for_roles(&role_slugs).await?;
        for row in membership_rows {
            let memberships = memberships_by_user.entry(row.user_id).or_default();
            match memberships.iter_mut().find(|m| m.id == row.id) {
                Some(existing) => {
                    existing.roles.push(row.role_slug.clone());
                    for permission in role_perms.get(&row.role_slug).into_iter().flatten() {
                        if !existing.permissions.contains(permission) {
                            existing.permissions.push(*permission);
                        }
                    }
                }
                None => memberships.push(GroupMembership {
                    id: row.id,
                    name: row.name,
                    is_personal: row.is_personal,
                    roles: vec![row.role_slug.clone()],
                    permissions: role_perms.get(&row.role_slug).cloned().unwrap_or_default(),
                }),
            }
        }

        Ok(users
            .into_iter()
            .map(|user| {
                let memberships = memberships_by_user.remove(&user.id).unwrap_or_default();
                let global_permissions = globals_by_user.remove(&user.id).unwrap_or_default();
                (user, memberships, global_permissions)
            })
            .collect())
    }

    // ---------------------------------------------------------------------
    // Default roles for new members
    // ---------------------------------------------------------------------

    /// Group-level default roles; `None` means "inherit the global default".
    pub async fn get_group_default_roles(&self, group_id: GroupId) -> Result<Option<Vec<String>>> {
        let row = sqlx::query!(r#"SELECT default_roles FROM groups WHERE id = $1"#, group_id.get())
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Group",
                id: group_id.to_string(),
            })?;
        Ok(row.default_roles)
    }

    /// Sets the group-level default roles for new members.
    pub async fn set_group_default_roles(&self, group_id: GroupId, roles: &[String]) -> Result<()> {
        sqlx::query!(
            r#"UPDATE groups SET default_roles = $1 WHERE id = $2"#,
            roles,
            group_id.get()
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    /// Resets the group default to "inherit the global default".
    pub async fn clear_group_default_roles(&self, group_id: GroupId) -> Result<()> {
        sqlx::query!(
            r#"UPDATE groups SET default_roles = NULL WHERE id = $1"#,
            group_id.get()
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    /// Global default roles for new members
    /// (`app_settings['groups.default_member_roles']`, fallback `[viewer]`).
    pub async fn get_global_member_default_roles(&self) -> Result<Vec<String>> {
        let row = sqlx::query!(
            r#"SELECT value FROM app_settings WHERE key = 'groups.default_member_roles'"#
        )
            .fetch_optional(&*self.pool)
            .await?;
        Ok(row
            .and_then(|row| serde_json::from_str::<Vec<String>>(&row.value).ok())
            .unwrap_or_else(|| vec!["viewer".to_string()]))
    }

    pub async fn set_global_member_default_roles(&self, roles: &[String]) -> Result<()> {
        let value =
            serde_json::to_string(roles).map_err(|err| Error::Misc(err.to_string()))?;
        sqlx::query!(
            r#"
            INSERT INTO app_settings (key, value)
            VALUES ('groups.default_member_roles', $1)
            ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value
            "#,
            value
        )
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    /// Number of organizational groups that override the global default.
    pub async fn count_groups_with_own_defaults(&self) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM groups
            WHERE default_roles IS NOT NULL AND is_personal = FALSE
            "#
        )
            .fetch_one(&*self.pool)
            .await?;
        Ok(count)
    }

    /// Resolution chain for a new member's roles:
    /// explicit choice → group default → global default → `[viewer]`.
    /// Personal groups and group creation always assign `manager` explicitly
    /// and never pass through here.
    pub async fn resolve_member_default_roles(&self, group_id: GroupId) -> Result<Vec<String>> {
        if let Some(roles) = self.get_group_default_roles(group_id).await? {
            if !roles.is_empty() {
                return Ok(roles);
            }
        }
        self.get_global_member_default_roles().await
    }

    /// Guards against removing or demoting the last member whose roles grant `required_perm`
    /// (such as `Permission::Group(GroupPerm::ManageMembers)`).
    pub async fn would_leave_group_without_permission(
        &self,
        group_id: GroupId,
        target_user_id: UserId,
        new_roles_for_target: &[String],
        required_perm: Permission,
    ) -> Result<bool> {
        // 1. Check if the target user's new roles still grant `required_perm`
        let role_perms = self.permissions_for_roles(new_roles_for_target).await?;
        let target_retains = new_roles_for_target
            .iter()
            .any(|slug| role_perms.get(slug).is_some_and(|p| p.contains(&required_perm)));

        if target_retains {
            return Ok(false);
        }

        // 2. Otherwise, check if any other member in the group still holds `required_perm`
        let other_role_slugs = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT role_slug
            FROM user_group_roles
            WHERE group_id = $1 AND user_id != $2
            "#,
            group_id as GroupId,
            target_user_id as UserId
        )
            .fetch_all(&*self.pool)
            .await?;

        let other_slugs: Vec<String> = other_role_slugs;
        let other_perms = self.permissions_for_roles(&other_slugs).await?;
        let another_member_has_perm = other_perms
            .values()
            .any(|perms| perms.contains(&required_perm));

        Ok(!another_member_has_perm)
    }

    /// Loads the complete resolved RBAC policy for `AuthCtx` and `/api/auth/me`.
    // pub async fn get_user_policy(&self, user_id: i64) -> Result<UserPolicy> {
    //     let global_rows = sqlx::query_scalar!(
    //     r#"SELECT permission FROM user_global_permissions WHERE user_id = $1"#,
    //     user_id
    // )
    //         .fetch_all(&*self.pool)
    //         .await?;
    //
    //     let member_group_ids = sqlx::query_scalar!(
    //     r#"SELECT DISTINCT group_id FROM user_group_roles WHERE user_id = $1"#,
    //     user_id
    // )
    //         .fetch_all(&*self.pool)
    //         .await?;
    //
    //     let group_perm_rows = sqlx::query!(
    //     r#"
    //     SELECT ugr.group_id, rp.permission
    //     FROM user_group_roles ugr
    //     JOIN role_permissions rp ON rp.role_slug = ugr.role_slug
    //     WHERE ugr.user_id = $1
    //     "#,
    //     user_id
    // )
    //         .fetch_all(&*self.pool)
    //         .await?;
    //
    //     let event_perm_rows = sqlx::query!(
    //     r#"
    //     SELECT eur.event_id, rp.permission
    //     FROM event_user_roles eur
    //     JOIN role_permissions rp ON rp.role_slug = eur.role_slug
    //     WHERE eur.user_id = $1
    //     "#,
    //     user_id
    // )
    //         .fetch_all(&*self.pool)
    //         .await?;
    //
    //     Ok(UserPolicy {
    //         user_id,
    //         global_permissions,
    //         group_permissions,
    //         event_overrides,
    //     })
    // }

    pub async fn get_user_policy(&self, user_id: UserId) -> Result<UserPolicy> {
        let global_rows = sqlx::query_scalar!(
            r#"SELECT permission FROM user_global_permissions WHERE user_id = $1"#,
            user_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        let group_role_rows = sqlx::query!(
            r#"SELECT group_id as "group_id: GroupId", role_slug FROM user_group_roles WHERE user_id = $1"#,
            user_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        let event_role_rows = sqlx::query!(
            r#"SELECT event_id as "event_id: EventId", role_slug FROM event_user_roles WHERE user_id = $1"#,
            user_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        let global_permissions: HashSet<GlobalPermission> = global_rows
            .into_iter()
            .filter_map(|p| p.parse().ok())
            .collect();

        let mut group_permissions: HashMap<GroupId, HashSet<Permission>> = HashMap::new();
        let mut role_slugs: Vec<String> = Vec::new();
        for row in &group_role_rows {
            role_slugs.push(row.role_slug.clone());
        }
        for row in &event_role_rows {
            role_slugs.push(row.role_slug.clone());
        }
        let role_perms = self.permissions_for_roles(&role_slugs).await?;
        for row in group_role_rows {
            // `.or_default()` ensures the group_id is always present in the map
            // even if a role has 0 permissions, so `AuthCtx.group_ids` stays accurate.
            let entry = group_permissions.entry(row.group_id).or_default();
            entry.extend(role_perms.get(&row.role_slug).into_iter().flatten().copied());
        }

        let mut event_overrides: HashMap<EventId, HashSet<Permission>> = HashMap::new();
        for row in event_role_rows {
            let entry = event_overrides.entry(row.event_id).or_default();
            entry.extend(role_perms.get(&row.role_slug).into_iter().flatten().copied());
        }

        Ok(UserPolicy {
            user_id: user_id,
            global_permissions,
            group_permissions,
            event_overrides,
        })
    }

    /// Memberships of one user including their roles and resolved permissions
    /// (used by `/api/auth/me` and login responses).
    pub async fn get_user_group_memberships(
        &self,
        user_id: UserId,
    ) -> Result<Vec<GroupMembership>> {
        let rows = sqlx::query!(
            r#"
            SELECT g.id as "id: GroupId", g.name, g.is_personal, ugr.role_slug
            FROM user_group_roles ugr
            JOIN groups g ON ugr.group_id = g.id
            WHERE ugr.user_id = $1
            ORDER BY g.is_personal DESC, g.name, ugr.role_slug
            "#,
            user_id.get()
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut memberships: Vec<GroupMembership> = Vec::new();
        let role_slugs: Vec<String> = rows.iter().map(|r| r.role_slug.clone()).collect();
        let role_perms = self.permissions_for_roles(&role_slugs).await?;
        for row in rows {
            let resolved = role_perms.get(&row.role_slug).cloned().unwrap_or_default();
            if let Some(existing) = memberships.iter_mut().find(|m| m.id == row.id) {
                existing.roles.push(row.role_slug.clone());
                for perm in &resolved {
                    if !existing.permissions.contains(perm) {
                        existing.permissions.push(*perm);
                    }
                }
            } else {
                memberships.push(GroupMembership {
                    id: row.id,
                    name: row.name,
                    is_personal: row.is_personal,
                    roles: vec![row.role_slug],
                    permissions: resolved,
                });
            }
        }

        Ok(memberships)
    }
}