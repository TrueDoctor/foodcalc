use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use bcrypt;
use serde::Deserialize;
use sqlx::postgres::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;

use crate::entities::user::User;
use crate::error::{Error, Result};
use crate::ops::users::UserOps;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct AuthBackend {
    user_ops: UserOps,
}

impl AuthBackend {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self {
            user_ops: UserOps::new(db),
        }
    }
}

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>> {
        let user = self
            .user_ops
            .get_user_by_string_reference(creds.username)
            .await;

        Ok(
            user.filter(|user| {
                bcrypt::verify(creds.password, &user.password_hash).unwrap_or(false)
            }),
        )
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>> {
        match self.user_ops.get_user(*user_id).await {
            Err(Error::UserNotFound { .. }) => Ok(None),
            a => a.map(Some),
        }
    }
}

// Type alias for convenience
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

pub async fn login(mut auth: AuthSession, credentials: Credentials) -> Result<AuthSession> {
    let username = credentials.username.clone();
    let user = auth
        .backend
        .authenticate(credentials)
        .await?
        .ok_or(Error::UserNotFound { name: username })?;

    auth.login(&user).await?;
    Ok(auth)
}

pub async fn logout(mut auth: AuthSession) -> Result<()> {
    auth.logout().await?;
    Ok(())
}

pub async fn register(
    backend: &AuthBackend,
    email: String,
    credentials: Credentials,
    is_admin: bool,
) -> Result<User> {
    backend
        .user_ops
        .create_user(User {
            id: -1,
            username: credentials.username,
            email,
            password_hash: bcrypt::hash(&credentials.password, 12).unwrap(),
            is_admin,
            created_at: OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc()),
        })
        .await
        .map_err(Into::into)
}

pub async fn change_password(
    backend: &AuthBackend,
    user_id: i64,
    new_password: String,
) -> Result<User> {
    backend
        .user_ops
        .update_password(user_id, new_password)
        .await
        .map_err(Into::into)
}
