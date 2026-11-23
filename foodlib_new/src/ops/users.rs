use crate::entities::types::ids::{GroupId, UserId};
use crate::{
    entities::user::*,
    error::{Error, Result},
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct UserOps {
    pool: Arc<PgPool>,
}

impl UserOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: CreateUser) -> Result<User> {
        // Encapsulation of the current timestamp to ensure consistent creation time.
        let created_at: OffsetDateTime = OffsetDateTime::now_utc();

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
            created_at
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
            FROM user_groups ugr
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
        sqlx::query!(
            r#"
            DELETE FROM user_groups
                   WHERE user_id = $1
           "#,
            id.get()
        )
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

    // ------------------------------------------------------------------------
    // Group Membership Queries
    // ------------------------------------------------------------------------

    /// All users together with their distinct group memberships.
    pub async fn list_users_with_groups(&self) -> Result<Vec<(User, Vec<Group>)>> {
        let users = self.get_all_users().await?;
        let memberships = sqlx::query_as!(
            MembershipRow,
            r#"
            SELECT DISTINCT ugr.user_id as "user_id: UserId", g.id as "id: GroupId", g.name, g.is_personal
            FROM user_groups ugr
            JOIN groups g ON g.id = ugr.group_id
            ORDER BY g.is_personal DESC, g.name
            "#
        )
            .fetch_all(&*self.pool)
            .await?;

        let mut groups_by_user: HashMap<UserId, Vec<Group>> = HashMap::new();
        for row in memberships {
            groups_by_user.entry(row.user_id).or_default().push(Group {
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
            FROM user_groups ugr
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
    pub async fn create_user_with_personal_group(&self, user: CreateUser) -> Result<(User, Group)> {
        let created_at: OffsetDateTime = OffsetDateTime::now_utc();
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
            created_at
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
            INSERT INTO user_groups (user_id, group_id)
            VALUES ($1, $2)
            "#,
            created.id.get(),
            group.id.get()
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
            RETURNING id as "id: GroupId", name, is_personal
            "#,
            name
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(row)
    }

    pub async fn add_user_to_group(&self, user_id: UserId, group_id: GroupId) -> Result<()> {
        sqlx::query!(r#"
            INSERT INTO user_groups (user_id, group_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            user_id.get(),
            group_id.get()
        )
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Removes a user from a group.
    pub async fn remove_user_from_group(&self, user_id: UserId, group_id: GroupId) -> Result<()> {
        sqlx::query!(r#"
            DELETE FROM user_groups
            WHERE user_id = $1 AND group_id = $2
            "#,
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
            FROM user_groups ugr
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
            FROM user_groups ugr
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
        let rows = sqlx::query_as!(Group, r#"
            SELECT id as "id: GroupId", name, is_personal
            FROM groups
            ORDER BY is_personal DESC, name
            "#
        )
            .fetch_all(&*self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn delete_group(&self, id: GroupId) -> Result<()> {
        // Transactional: remove all memberships first, then delete the group itself.
        let mut tx = self.pool.begin().await?;

        sqlx::query!(r#"
            DELETE FROM user_groups
               WHERE group_id = $1
           "#,
            id.get()
        )
            .execute(&mut *tx)
            .await?;

        sqlx::query!(r#"
            DELETE FROM groups
            WHERE id = $1
            "#, id.get())

            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

pub async fn members_of_group(foodlib: &crate::FoodLib, group_id: GroupId) -> Result<Vec<User>> {
    let pool = foodlib.pool_arc();
    let rows = sqlx::query_as!(
        User,
        r#"
        SELECT u.id, u.username, u.email, u.password_hash, u.is_admin, u.created_at
        FROM user_groups ug
        JOIN users u ON u.id = ug.user_id
        WHERE ug.group_id = $1
        ORDER BY u.username
        "#,
        group_id.get()
    )
    .fetch_all(&*pool)
    .await?;
    Ok(rows)
}