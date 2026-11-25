use crate::ctx::AppCtx;
use crate::entities::user::{CreateUser, Group, User};
use crate::types::ids::UserId;
use crate::{AppState, Error, Result};

/// Everything the user detail view renders, fetched in one use case.
pub struct UserDetailData {
    pub user: User,
    pub user_groups: Vec<Group>,
    pub all_groups: Vec<Group>,
}

/// Requires admin privileges.
pub async fn gather_user_detail(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
) -> Result<UserDetailData> {
    actor.require_admin()?;
    let user = state.db().users().get_user(user_id).await?;
    let user_groups = state.db().users().get_user_groups(user_id).await?;
    let all_groups = state.db().users().get_all_groups().await?;
    Ok(UserDetailData {
        user,
        user_groups,
        all_groups,
    })
}

/// Creates a user together with their personal group and sets the initial
/// password. Username/email are trimmed; username, email and password are
/// required.
pub async fn create_user(
    state: &AppState,
    actor: &AppCtx,
    username: String,
    email: String,
    password: String,
    is_admin: bool,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    let username = username.trim().to_string();
    let email = email.trim().to_string();
    if username.is_empty() || email.is_empty() || password.is_empty() {
        return Err(Error::Validation {
            message: "Username, email, and password are required".into(),
        });
    }
    let (created, _) = state
        .db()
        .users()
        .create_user_with_personal_group(CreateUser {
            username,
            email,
            password_hash: String::new(),
            is_admin,
        })
        .await?;
    state
        .db()
        .users()
        .update_password(created.id, password)
        .await?;
    Ok(())
}

/// Updates a user's profile. Username/email are trimmed.
/// Requires admin privileges.
pub async fn update_user(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
    username: String,
    email: String,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    let username = username.trim().to_string();
    let email = email.trim().to_string();
    if username.is_empty() || email.is_empty() {
        return Err(Error::Validation {
            message: "Username and email are required".into(),
        });
    }
    let mut user = state.db().users().get_user(user_id).await?;
    user.username = username;
    user.email = email;
    state.db().users().update_user(user).await?;
    Ok(())
}

/// Flips a user's admin flag. Admins cannot change their own admin status.
pub async fn toggle_admin(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    if user_id == actor.user.id {
        return Err(Error::Forbidden(
            "You cannot change your own admin status".into(),
        ));
    }
    let mut user = state.db().users().get_user(user_id).await?;
    user.is_admin = !user.is_admin;
    state.db().users().update_user(user).await?;
    Ok(())
}

/// Sets a user's password. An empty password is rejected.
/// Requires admin privileges.
pub async fn set_password(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
    password: String,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    if password.is_empty() {
        return Err(Error::Validation {
            message: "Password cannot be empty".into(),
        });
    }
    state
        .db()
        .users()
        .update_password(user_id, password)
        .await?;
    Ok(())
}

/// Deletes a user and their personal group. Admins cannot delete themselves.
pub async fn delete_user(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    if user_id == actor.user.id {
        return Err(Error::Forbidden("You cannot delete yourself".into()));
    }
    state.db().users().delete_user_cascade(user_id).await?;
    Ok(())
}


/// Lists every user in the system.
pub async fn list_all_users(state: &AppState) -> Result<Vec<User>> {
    state.db().users().get_all_users().await
}
