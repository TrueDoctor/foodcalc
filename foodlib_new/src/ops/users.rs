use crate::{
    entities::user::*,
    error::{Error, Result},
};
use sqlx::PgPool;
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
            SELECT g.id
            FROM user_groups ug
            JOIN groups g ON g.id = ug.group_id
            WHERE ug.user_id = $1 AND g.is_personal = TRUE
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

        sqlx::query!(r#"DELETE FROM user_groups WHERE user_id = $1"#, id)
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

    /// All users together with their group memberships, grouped in Rust.
    /// Single pair of queries instead of per-user group lookups (admin UI).
    pub async fn list_users_with_groups(&self) -> Result<Vec<(User, Vec<Group>)>> {
        let users = self.get_all_users().await?;
        let memberships = sqlx::query_as!(
            MembershipRow,
            r#"
            SELECT ug.user_id, g.id, g.name, g.is_personal
            FROM user_groups ug
            JOIN groups g ON g.id = ug.group_id
            ORDER BY g.is_personal DESC, g.name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut groups_by_user: std::collections::HashMap<i64, Vec<Group>> =
            std::collections::HashMap::new();
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

    /// All members of a group (admin UI / group management).
    pub async fn get_group_members(&self, group_id: i32) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            User,
            r#"
            SELECT u.id, u.username, u.email, u.password_hash, u.is_admin, u.created_at
            FROM user_groups ug
            JOIN users u ON u.id = ug.user_id
            WHERE ug.group_id = $1
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
            r#"INSERT INTO user_groups (user_id, group_id, role) VALUES ($1, $2, 'manager')"#,
            created.id,
            group.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok((created, group))
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

    pub async fn add_user_to_group(&self, user_id: i64, group_id: i32) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO user_groups (user_id, group_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
            user_id,
            group_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_user_from_group(&self, user_id: i64, group_id: i32) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM user_groups WHERE user_id = $1 AND group_id = $2"#,
            user_id,
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
            SELECT g.id, g.name, g.is_personal
            FROM user_groups ug
            JOIN groups g ON ug.group_id = g.id
            WHERE ug.user_id = $1
            ORDER BY g.is_personal DESC, g.name
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    /// Lists every group (for owner pickers in the UI).
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
            SELECT g.id, g.name, g.is_personal
            FROM user_groups ug
            JOIN groups g ON ug.group_id = g.id
            WHERE ug.user_id = $1 AND g.is_personal = TRUE
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

    /// Number of entities (recipes, ingredients, inventories, events) owned
    /// by a group — used to refuse deleting non-empty groups.
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
        sqlx::query!(r#"DELETE FROM user_groups WHERE group_id = $1"#, id)
            .execute(&*self.pool)
            .await?;
        sqlx::query!(r#"DELETE FROM groups WHERE id = $1"#, id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}


impl UserOps {
    /// Memberships of one user including their role in each group (`auth/me`,
    /// permission plumbing in `AuthCtx`).
    pub async fn get_user_group_memberships(
        &self,
        user_id: i64,
    ) -> Result<Vec<GroupMembership>> {
        let rows = sqlx::query_as!(
            GroupMembership,
            r#"
            SELECT g.id, g.name, g.is_personal, ug.role
            FROM user_groups ug
            JOIN groups g ON ug.group_id = g.id
            WHERE ug.user_id = $1
            ORDER BY g.is_personal DESC, g.name
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    /// Members of a group together with their role (group management UI).
    pub async fn get_group_members_with_role(
        &self,
        group_id: i32,
    ) -> Result<Vec<(User, String)>> {
        let members = self.get_group_members(group_id).await?;
        let role_rows = sqlx::query!(
            r#"SELECT user_id, role FROM user_groups WHERE group_id = $1"#,
            group_id
        )
        .fetch_all(&*self.pool)
        .await?;
        let roles: std::collections::HashMap<i64, String> = role_rows
            .into_iter()
            .map(|row| (row.user_id, row.role))
            .collect();
        Ok(members
            .into_iter()
            .map(|member| {
                let role = roles.get(&member.id).cloned().unwrap_or_else(|| "viewer".into());
                (member, role)
            })
            .collect())
    }

    /// Role of one user in one group (last-manager guards).
    pub async fn get_member_role(&self, user_id: i64, group_id: i32) -> Result<String> {
        let role = sqlx::query_scalar!(
            r#"SELECT role FROM user_groups WHERE user_id = $1 AND group_id = $2"#,
            user_id,
            group_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "membership",
            id: format!("{group_id}/{user_id}"),
        })?;
        Ok(role)
    }

    /// Sets a member's role (viewer/editor/manager) in one group.
    pub async fn set_member_role(&self, user_id: i64, group_id: i32, role: &str) -> Result<()> {
        sqlx::query!(
            r#"UPDATE user_groups SET role = $3 WHERE user_id = $1 AND group_id = $2"#,
            user_id,
            group_id,
            role
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    /// Number of managers in a group (protects the last manager from being
    /// demoted or removed).
    pub async fn count_group_managers(&self, group_id: i32) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM user_groups
            WHERE group_id = $1 AND role = 'manager'
            "#,
            group_id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(count.unwrap_or(0))
    }
}
