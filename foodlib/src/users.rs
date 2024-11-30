use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use time::macros::datetime;

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: PgPool,
}

impl Backend {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(creds.username)
            .fetch_optional(&self.db)
            .await?;

        // Verify password in a blocking task since bcrypt is CPU intensive
        Ok(
            user.filter(|user| {
                bcrypt::verify(creds.password, &user.password_hash).unwrap_or(false)
            }),
        )
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;
        Ok(user)
    }
}

// Type alias for convenience
pub type AuthSession = axum_login::AuthSession<Backend>;

use crate::PrimitiveDateTime;

use crate::FoodBase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credenitals {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: PrimitiveDateTime,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 0,
            username: "test".to_string(),
            email: "".to_string(),
            password_hash: "password".to_string(),
            is_admin: true,
            created_at: datetime!(1970-1-1 00:00),
        }
    }
}

pub enum UserError {
    UsernameNotFound,
    PasswordIncorrect,
}

impl FoodBase {
    pub async fn authenticate_user(
        &self,
        Credenitals { username, password }: Credenitals,
    ) -> Result<User, UserError> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT * FROM users WHERE username = $1
            "#,
            username
        )
        .fetch_one(&*self.pg_pool)
        .await
        .map_err(|_| UserError::UsernameNotFound)?;

        if bcrypt::verify(password, &user.password_hash).unwrap() {
            Ok(user)
        } else {
            Err(UserError::PasswordIncorrect)
        }
    }

    pub async fn create_user(
        &self,
        email: String,
        credentials: Credenitals,
        is_admin: bool,
    ) -> Result<User, sqlx::Error> {
        let password_hash = bcrypt::hash(credentials.password, 12).unwrap();

        let user = sqlx::query_as!(
            User,
            r#"
                INSERT INTO users (username, email, password_hash, is_admin)
                VALUES ($1, $2, $3, $4)
                RETURNING *
            "#,
            credentials.username,
            email,
            password_hash,
            is_admin
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(user)
    }

    pub async fn update_password(&self, id: i64, password: String) -> Result<User, sqlx::Error> {
        let password_hash = bcrypt::hash(password, 12).unwrap();

        let user = sqlx::query_as!(
            User,
            r#"
                UPDATE users SET password_hash = $1
                WHERE id = $2
                RETURNING *
            "#,
            password_hash,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user(&self, id: i64) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT * FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_string_reference(&self, reference: String) -> Option<User> {
        let id = reference.parse::<i64>().unwrap_or(-1);
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT * FROM users 
                WHERE id = $1 OR username = $2 OR email = $2
            "#,
            id,
            reference
        )
        .fetch_one(&*self.pg_pool)
        .await;

        match user {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    pub async fn get_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as!(User, r#"SELECT * FROM users"#)
            .fetch_all(&*self.pg_pool)
            .await?;

        Ok(users)
    }

    pub async fn delete_user(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                DELETE FROM users WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pg_pool)
        .await?;

        Ok(())
    }

    pub async fn change_username(&self, id: i64, username: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                UPDATE users SET username = $1
                WHERE id = $2
                RETURNING *
            "#,
            username,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(user)
    }

    pub async fn change_email(&self, id: i64, email: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                UPDATE users SET email = $1
                WHERE id = $2
                RETURNING *
            "#,
            email,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(user)
    }

    pub async fn change_is_admin(&self, id: i64, is_admin: bool) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
                UPDATE users SET is_admin = $1
                WHERE id = $2
                RETURNING *
            "#,
            is_admin,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(user)
    }
}
