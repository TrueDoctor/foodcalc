use async_trait::async_trait;
use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum_login::{AuthUser, AuthnBackend, UserId as AuthUserId};
use serde::Deserialize;

use foodlib_new::entities::user::{CreateUser, User};
use foodlib_new::error::{Error, Result};
use foodlib_new::types::ids::UserId;
use foodlib_new::AppState;

use crate::frontend::web_error::WebError;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}


#[derive(Clone, Debug)]
pub struct SessionUser(pub User);

impl std::ops::Deref for SessionUser {
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AuthUser for SessionUser {
    type Id = UserId;

    fn id(&self) -> Self::Id {
        self.0.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.0.password_hash.as_bytes()
    }
}

/// Extractor for the authenticated `User` (newtype required by Rust orphan rules).
pub struct CurrentUser(pub User);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, WebError> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| WebError(Error::Unauthorized("no auth headers".to_string())))?;

        let SessionUser(user) = auth
            .user
            .ok_or_else(|| WebError(Error::Unauthorized("Authentication required".to_string())))?;
        Ok(CurrentUser(user))
    }
}

pub struct OptCurrentUser(pub Option<User>);

impl<S> OptionalFromRequestParts<S> for OptCurrentUser
where
    S: Send + Sync,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Option<Self>, WebError> {
        let auth = AuthSession::from_request_parts(parts, state).await.ok();
        Ok(Some(OptCurrentUser(auth.and_then(|a| a.user.map(|u| u.0)))))
    }
}
#[derive(Clone)]
pub struct AuthBackend {
    state: AppState,
}

impl AuthBackend {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = SessionUser;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>> {
        let user = self.state.get_user_by_login(&creds.username).await;
        Ok(user
            .filter(|u| bcrypt::verify(&creds.password, &u.password_hash).unwrap_or(false))
            .map(SessionUser))
    }

    async fn get_user(&self, user_id: &AuthUserId<Self>) -> Result<Option<Self::User>> {
        match self.state.get_user(*user_id).await {
            Err(Error::UserNotFound { .. } | Error::NotFound { .. }) => Ok(None),
            res => res.map(|u| Some(SessionUser(u))),
        }
    }
}

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

pub async fn login(mut auth: AuthSession, credentials: Credentials) -> Result<AuthSession> {
    let username = credentials.username.clone();
    let user = auth
        .backend
        .authenticate(credentials)
        .await?
        .ok_or(Error::UserNotFound { name: username })?;

    auth.login(&user)
        .await
        .map_err(|e| Error::Misc(e.to_string()))?;
    Ok(auth)
}

pub async fn logout(mut auth: AuthSession) -> Result<()> {
    auth.logout()
        .await
        .map_err(|e| Error::Misc(e.to_string()))?;
    Ok(())
}

pub async fn register(
    backend: &AuthBackend,
    email: String,
    credentials: Credentials,
    is_admin: bool,
) -> Result<User> {
    backend
        .state
        .register_user(CreateUser {
            username: credentials.username,
            email,
            password_hash: bcrypt::hash(&credentials.password, 12)?,
            is_admin,
        })
        .await
}

pub async fn change_password(
    backend: &AuthBackend,
    user_id: UserId,
    new_password: String,
) -> Result<User> {
    backend
        .state
        .update_user_password(user_id, new_password)
        .await
}
