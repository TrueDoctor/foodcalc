use axum_login::secrecy::SecretVec;
use axum_login::AuthUser;
use axum_login::PostgresStore;
use chrono::NaiveDateTime;

use serde::{Deserialize, Serialize};

use crate::FoodBase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credenitals {
    username: String,
    password: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: NaiveDateTime,
}

impl AuthUser<i64> for User {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }
}

pub type AuthContext = axum_login::extractors::AuthContext<i64, User, PostgresStore<User>>;

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
}
