use crate::policy::{permissions_for_role_slug, BuiltInRole, GlobalPermission, Permission};
use crate::{
    entities::user::*,
    error::{Error, Result},
    policy::UserPolicy,
};
use bigdecimal::ToPrimitive;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct UserOps {
    pool: Arc<PgPool>,
}

impl UserOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: User) -> Result<User> {
        let row = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash, is_admin, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, password_hash, is_admin, created_at
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.is_admin,
            user.created_at
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(row)
    }

    pub async fn update_user(&self, user: User) -> Result<User> {
        let row = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET username = $1, email = $2, password_hash = $3, is_admin = $4
            WHERE id = $5
            RETURNING id, username, email, password_hash, is_admin, created_at
            "#,
            user.username,
            user.email,
            user.password_hash,
            user.is_admin,
            user.id
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(row)
    }

    pub async fn update_password(&self, id: i64, password: String) -> Result<User> {
        let password_hash = bcrypt::hash(password, 12)?;

        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users SET password_hash = $1
            WHERE id = $2
            RETURNING id, username, email, password_hash, is_admin, created_at
            "#,
            password_hash,
            id
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(user)
    }

    pub async fn delete_user(&self, id: i64) -> Result<()> {
        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Deletes a user and their personal group, plus all their group memberships.
    /// Refuses if the user's personal group still owns any recipes, ingredients,
    /// inventories, or events.
    pub async fn delete_user_cascade(&self, id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let personal_group_id = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT g.id
            FROM user_group_roles ugr
            JOIN groups g ON g.id = ugr.group_id
            WHERE ugr.user_id = $1 AND g.is_personal = TRUE
            "#,
            id
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
        sqlx::query!(r#"DELETE FROM user_group_roles WHERE user_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        if let Some(group_id) = personal_group_id {
            sqlx::query!(r#"DELETE FROM groups WHERE id = $1"#, group_id)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_user(&self, id: i64) -> Result<User> {
        let row = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, is_admin, created_at
            FROM users
            WHERE id = $1
            "#,
            id
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
            SELECT id, username, email, password_hash, is_admin, created_at
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
            SELECT id, username, email, password_hash, is_admin, created_at
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
            SELECT DISTINCT ugr.user_id, g.id, g.name, g.is_personal
            FROM user_group_roles ugr
            JOIN groups g ON g.id = ugr.group_id
            ORDER BY g.is_personal DESC, g.name
            "#
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut groups_by_user: HashMap<i64, Vec<Group>> = HashMap::new();
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
    pub async fn get_group_members(&self, group_id: i32) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            User,
            r#"
            SELECT DISTINCT u.id, u.username, u.email, u.password_hash, u.is_admin, u.created_at
            FROM user_group_roles ugr
            JOIN users u ON u.id = ugr.user_id
            WHERE ugr.group_id = $1
            ORDER BY u.username
            "#,
            group_id
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
            RETURNING id, username, email, password_hash, is_admin, created_at
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
            RETURNING id, name, is_personal
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
            created.id,
            group.id
        )
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok((created, group))
    }

    /// Creates a group and assigns the user as `manager` atomically.
    pub async fn create_group_with_manager(
        &self,
        name: &str,
        user_id: i64,
    ) -> Result<(Group, User)> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, password_hash, is_admin, created_at
            FROM users
            WHERE id = $1
            "#,
            user_id
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
            RETURNING id, name, is_personal
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
            user_id,
            group.id
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
            RETURNING id, name, is_personal
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
        user_id: i64,
        group_id: i32,
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
                user_id,
                group_id,
                role
            )
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Grants a single role to a user in a group (used by admin self-grant).
    pub async fn grant_membership(&self, user_id: i64, group_id: i32, role: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_group_roles (user_id, group_id, role_slug)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            user_id,
            group_id,
            role
        )
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Replaces the complete set of roles for a user in a group.
    pub async fn set_member_roles(
        &self,
        user_id: i64,
        group_id: i32,
        roles: &[String],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"DELETE FROM user_group_roles WHERE user_id = $1 AND group_id = $2"#,
            user_id as i32,
            group_id
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
                user_id as i32,
                group_id,
                role
            )
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn remove_user_from_group(&self, user_id: i64, group_id: i32) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM user_group_roles WHERE user_id = $1 AND group_id = $2"#,
            user_id as i32,
            group_id
        )
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_user_groups(&self, user_id: i64) -> Result<Vec<Group>> {
        let rows = sqlx::query_as!(
            Group,
            r#"
            SELECT DISTINCT g.id, g.name, g.is_personal
            FROM user_group_roles ugr
            JOIN groups g ON ugr.group_id = g.id
            WHERE ugr.user_id = $1
            ORDER BY g.is_personal DESC, g.name
            "#,
            user_id as i32
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn list_groups(&self) -> Result<Vec<Group>> {
        let groups = sqlx::query_as!(
            Group,
            r#"SELECT id, name, is_personal FROM groups ORDER BY name"#
        )
            .fetch_all(&*self.pool)
            .await?;
        Ok(groups)
    }

    pub async fn get_personal_group(&self, user_id: i64) -> Result<Group> {
        sqlx::query_as!(
            Group,
            r#"
            SELECT DISTINCT g.id, g.name, g.is_personal
            FROM user_group_roles ugr
            JOIN groups g ON ugr.group_id = g.id
            WHERE ugr.user_id = $1 AND g.is_personal = TRUE
            "#,
            user_id
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "personal group",
                id: user_id.to_string(),
            })
    }

    pub async fn get_group(&self, id: i32) -> Result<Group> {
        let row = sqlx::query_as!(
            Group,
            r#"SELECT id, name, is_personal FROM groups WHERE id = $1"#,
            id
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Group",
                id: id.to_string(),
            })?;

        Ok(row)
    }

    pub async fn count_owned_entities(&self, group_id: i32) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT
                (SELECT COUNT(*) FROM recipes WHERE group_id = $1)
              + (SELECT COUNT(*) FROM ingredients WHERE group_id = $1)
              + (SELECT COUNT(*) FROM inventories WHERE group_id = $1)
              + (SELECT COUNT(*) FROM events WHERE group_id = $1)
            "#,
            group_id
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(count.unwrap_or(0))
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let rows = sqlx::query_as!(
            Group,
            r#"SELECT id, name, is_personal FROM groups ORDER BY is_personal DESC, name"#
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn delete_group(&self, id: i32) -> Result<()> {
        sqlx::query!(r#"DELETE FROM user_group_roles WHERE group_id = $1"#, id)
            .execute(&*self.pool)
            .await?;
        sqlx::query!(r#"DELETE FROM groups WHERE id = $1"#, id)
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
        group_id: i32,
    ) -> Result<Vec<(User, Vec<String>)>> {
        let members = self.get_group_members(group_id).await?;
        let role_rows = sqlx::query!(
            r#"
            SELECT user_id, role_slug
            FROM user_group_roles
            WHERE group_id = $1
            ORDER BY role_slug
            "#,
            group_id
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut roles_by_user: HashMap<i64, Vec<String>> = HashMap::new();
        for row in role_rows {
            roles_by_user
                .entry(row.user_id as i64)
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

    /// Guards against removing or demoting the last member whose roles grant `required_perm`
    /// (such as `Permission::Group(GroupPerm::ManageMembers)`).
    pub async fn would_leave_group_without_permission(
        &self,
        group_id: i32,
        target_user_id: i64,
        new_roles_for_target: &[String],
        required_perm: Permission,
    ) -> Result<bool> {
        // 1. Check if the target user's new roles still grant `required_perm`
        let target_retains = new_roles_for_target
            .iter()
            .any(|slug| permissions_for_role_slug(slug).contains(&required_perm));

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
            group_id,
            target_user_id
        )
            .fetch_all(&*self.pool)
            .await?;

        let another_member_has_perm = other_role_slugs
            .iter()
            .any(|slug| permissions_for_role_slug(slug).contains(&required_perm));

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

    pub async fn get_user_policy(&self, user_id: i64) -> Result<UserPolicy> {
        let global_rows = sqlx::query_scalar!(
            r#"SELECT permission FROM user_global_permissions WHERE user_id = $1"#,
            user_id
        )
            .fetch_all(&*self.pool)
            .await?;

        let group_role_rows = sqlx::query!(
            r#"SELECT group_id, role_slug FROM user_group_roles WHERE user_id = $1"#,
            user_id
        )
            .fetch_all(&*self.pool)
            .await?;

        let event_role_rows = sqlx::query!(
            r#"SELECT event_id, role_slug FROM event_user_roles WHERE user_id = $1"#,
            user_id
        )
            .fetch_all(&*self.pool)
            .await?;

        let global_permissions: HashSet<GlobalPermission> = global_rows
            .into_iter()
            .filter_map(|p| p.parse().ok())
            .collect();

        let mut group_permissions: HashMap<i32, HashSet<Permission>> = HashMap::new();
        for row in group_role_rows {
            // `.or_default()` ensures the group_id is always present in the map
            // even if a role has 0 permissions, so `AuthCtx.group_ids` stays accurate.
            let entry = group_permissions.entry(row.group_id).or_default();
            entry.extend(permissions_for_role_slug(&row.role_slug).iter().copied());
        }

        let mut event_overrides: HashMap<i32, HashSet<Permission>> = HashMap::new();
        for row in event_role_rows {
            let entry = event_overrides.entry(row.event_id).or_default();
            entry.extend(permissions_for_role_slug(&row.role_slug).iter().copied());
        }

        Ok(UserPolicy {
            user_id: user_id.to_i32().unwrap_or(0),
            global_permissions,
            group_permissions,
            event_overrides,
        })
    }

    /// Memberships of one user including their roles and resolved permissions
    /// (used by `/api/auth/me` and login responses).
    pub async fn get_user_group_memberships(
        &self,
        user_id: i64,
    ) -> Result<Vec<GroupMembership>> {
        let rows = sqlx::query!(
            r#"
            SELECT g.id, g.name, g.is_personal, ugr.role_slug
            FROM user_group_roles ugr
            JOIN groups g ON ugr.group_id = g.id
            WHERE ugr.user_id = $1
            ORDER BY g.is_personal DESC, g.name, ugr.role_slug
            "#,
            user_id
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut memberships: Vec<GroupMembership> = Vec::new();
        for row in rows {
            if let Some(existing) = memberships.iter_mut().find(|m| m.id == row.id) {
                existing.roles.push(row.role_slug.clone());
                for &perm in permissions_for_role_slug(&row.role_slug) {
                    if !existing.permissions.contains(&perm) {
                        existing.permissions.push(perm);
                    }
                }
            } else {
                let mut permissions = Vec::new();
                for &perm in permissions_for_role_slug(&row.role_slug) {
                    if !permissions.contains(&perm) {
                        permissions.push(perm);
                    }
                }
                memberships.push(GroupMembership {
                    id: row.id,
                    name: row.name,
                    is_personal: row.is_personal,
                    roles: vec![row.role_slug],
                    permissions,
                });
            }
        }

        Ok(memberships)
    }
}