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

    pub async fn create_group(&self, group: Group) -> Result<Group> {
        let row = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (name)
            VALUES ($1)
            RETURNING id, name
            "#,
            group.name
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn add_user_to_group(&self, user_group: UserGroup) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_groups (user_id, group_id)
            VALUES ($1, $2)
            "#,
            user_group.user_id,
            user_group.group_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_user_from_group(&self, user_id: i64, group_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM user_groups
            WHERE user_id = $1 AND group_id = $2
            "#,
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
            SELECT g.id, g.name
            FROM user_groups ug
            JOIN groups g ON ug.group_id = g.id
            WHERE ug.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_group(&self, id: i32) -> Result<Group> {
        let row = sqlx::query_as!(
            Group,
            r#"
            SELECT id, name
            FROM groups
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_all_groups(&self) -> Result<Vec<Group>> {
        let rows = sqlx::query_as!(
            Group,
            r#"
            SELECT id, name
            FROM groups
            ORDER BY name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
