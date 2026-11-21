use async_trait::async_trait;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
#[cfg(feature = "legacy-frontend")]
use time::OffsetDateTime;


use crate::entities::types::ids::UserId as DbUserId;
use crate::entities::user::User;
use crate::error::{Error, Result};
use crate::ops::users::UserOps;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl AuthUser for User {
    type Id = DbUserId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Unauthorized("No auth session available".into()))?;

        auth.user
            .ok_or_else(|| Error::Unauthorized("Authentication required".into()))
    }
}

impl<S> OptionalFromRequestParts<S> for User
where
    S: Send + Sync,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Option<Self>> {
        let auth = AuthSession::from_request_parts(parts, state).await.ok();
        Ok(auth.and_then(|a| a.user))
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

        Ok(user.filter(|u| bcrypt::verify(&creds.password, &u.password_hash).unwrap_or(false)))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>> {
        match self.user_ops.get_user(*user_id).await {
            Err(Error::UserNotFound { .. } | Error::NotFound { .. }) => Ok(None),
            res => res.map(Some),
        }
    }
}


#[cfg(feature = "legacy-frontend")]
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

#[cfg(feature = "legacy-frontend")]
pub async fn logout(mut auth: AuthSession) -> Result<()> {
    auth.logout().await?;
    Ok(())
}

#[cfg(feature = "legacy-frontend")]
pub async fn register(
    backend: &AuthBackend,
    email: String,
    credentials: Credentials,
    is_admin: bool,
) -> Result<User> {
    let password_hash = bcrypt::hash(&credentials.password, 12)
        .map_err(|err| Error::Misc(err.to_string()))?;

    backend
        .user_ops
        .create_user_with_personal_group(User {
            id: DbUserId(0),
            username: credentials.username,
            email,
            password_hash,
            is_admin,
            created_at: OffsetDateTime::now_utc(),
        })
        .await
        .map(|(user, _group)| user)
}

#[cfg(feature = "legacy-frontend")]
pub async fn change_password(
    backend: &AuthBackend,
    user_id: DbUserId,
    new_password: String,
) -> Result<User> {
    backend.user_ops.update_password(user_id, new_password).await
}