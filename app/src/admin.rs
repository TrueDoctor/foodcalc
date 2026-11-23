//! Admin use-cases: group management, user management, Metro sync.

use foodlib_new::entities::types::ids::{GroupId, UserId};
use foodlib_new::ops::metro_sync::{SyncOptions, SyncReport};
use foodlib_new::user::{CreateUser, Group};

use crate::app_ctx::AppCtx;
use crate::{Error, MyAppState, Result};

/// Admins may manage any group. Members may manage groups they belong to.
/// Personal groups are never manageable here — they live and die with their
/// user. Returns the group on success.
pub async fn assert_can_manage_group(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<Group> {
    let group = state.users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Personal groups can only be managed via their owning user".into(),
        ));
    }
    if !actor.ctx.can_access_group(group_id) {
        return Err(Error::Forbidden(
            "You don't have permission to manage this group".into(),
        ));
    }
    Ok(group)
}

/// Creates a shared group and adds the creating user as its first member.
/// The name is trimmed; an empty name is rejected.
pub async fn create_group(
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
) -> Result<Group> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation {
            message: "Group name cannot be empty".into(),
        });
    }
    let group = state.users().create_group(trimmed).await?;
    state
        .users()
        .add_user_to_group(actor.ctx.user.id, group.id)
        .await?;
    Ok(group)
}

/// Deletes a shared group.
/// Requires group-management rights (see `assert_can_manage_group`).
pub async fn delete_group(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<()> {
    assert_can_manage_group(state, actor, group_id).await?;
    state.users().delete_group(group_id).await?;
    Ok(())
}

/// Adds a user to a shared group.
/// Requires group-management rights.
pub async fn add_member(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
    user_id: UserId,
) -> Result<Group> {
    let group = assert_can_manage_group(state, actor, group_id).await?;
    state.users().add_user_to_group(user_id, group_id).await?;
    Ok(group)
}

/// Removes a user from a shared group.
/// Requires group-management rights.
pub async fn remove_member(
    state: &MyAppState,
    actor: &AppCtx,
    group_id: GroupId,
    user_id: UserId,
) -> Result<()> {
    assert_can_manage_group(state, actor, group_id).await?;
    state.users().remove_user_from_group(user_id, group_id).await?;
    Ok(())
}

/// Runs a Metro market-data sync. Only administrators may trigger it.
pub async fn run_metro_sync(
    state: &MyAppState,
    actor: &AppCtx,
    wipe_existing: bool,
) -> Result<SyncReport> {
    if !actor.ctx.user.is_admin {
        return Err(Error::Forbidden(
            "Only admins can trigger Metro sync".into(),
        ));
    }
    let opts = SyncOptions { wipe_existing };
    state.metro_sync().sync_with(None, opts).await
}

/// Admin privileges are required for every user-management use-case.
pub fn assert_admin(actor: &AppCtx) -> Result<()> {
    if actor.ctx.user.is_admin {
        Ok(())
    } else {
        Err(Error::Forbidden(
            "Admin privileges required to manage users".into(),
        ))
    }
}

/// Creates a user together with their personal group and sets the initial
/// password. Username/email are trimmed; username, email and password are
/// required.
pub async fn create_user(
    state: &MyAppState,
    actor: &AppCtx,
    username: String,
    email: String,
    password: String,
    is_admin: bool,
) -> Result<()> {
    assert_admin(actor)?;
    let username = username.trim().to_string();
    let email = email.trim().to_string();
    if username.is_empty() || email.is_empty() || password.is_empty() {
        return Err(Error::Validation {
            message: "Username, email, and password are required".into(),
        });
    }
    let (created, _) = state
        .users()
        .create_user_with_personal_group(CreateUser {
            username,
            email,
            password_hash: String::new(),
            is_admin,
        })
        .await?;
    state.users().update_password(created.id, password).await?;
    Ok(())
}

/// Updates a user's profile. Username/email are trimmed.
/// Requires admin privileges.
pub async fn update_user(
    state: &MyAppState,
    actor: &AppCtx,
    user_id: UserId,
    username: String,
    email: String,
) -> Result<()> {
    assert_admin(actor)?;
    let mut user = state.users().get_user(user_id).await?;
    user.username = username.trim().to_string();
    user.email = email.trim().to_string();
    state.users().update_user(user).await?;
    Ok(())
}

/// Flips a user's admin flag. Admins cannot change their own admin status.
pub async fn toggle_admin(
    state: &MyAppState,
    actor: &AppCtx,
    user_id: UserId,
) -> Result<()> {
    assert_admin(actor)?;
    if user_id == actor.ctx.user.id {
        return Err(Error::Forbidden(
            "You cannot change your own admin status".into(),
        ));
    }
    let mut user = state.users().get_user(user_id).await?;
    user.is_admin = !user.is_admin;
    state.users().update_user(user).await?;
    Ok(())
}

/// Sets a user's password. An empty password is rejected.
/// Requires admin privileges.
pub async fn set_password(
    state: &MyAppState,
    actor: &AppCtx,
    user_id: UserId,
    password: String,
) -> Result<()> {
    assert_admin(actor)?;
    if password.is_empty() {
        return Err(Error::Validation {
            message: "Password cannot be empty".into(),
        });
    }
    state.users().update_password(user_id, password).await?;
    Ok(())
}

/// Deletes a user and their personal group. Admins cannot delete themselves.
pub async fn delete_user(
    state: &MyAppState,
    actor: &AppCtx,
    user_id: UserId,
) -> Result<()> {
    assert_admin(actor)?;
    if user_id == actor.ctx.user.id {
        return Err(Error::Forbidden("You cannot delete yourself".into()));
    }
    state.users().delete_user_cascade(user_id).await?;
    Ok(())
}

/// Adds a user to a group. Requires admin privileges.
pub async fn add_user_to_group(
    state: &MyAppState,
    actor: &AppCtx,
    user_id: UserId,
    group_id: GroupId,
) -> Result<()> {
    assert_admin(actor)?;
    state.users().add_user_to_group(user_id, group_id).await?;
    Ok(())
}

/// Removes a user from a group. Users cannot be removed from their personal
/// group. Requires admin privileges.
pub async fn remove_user_from_group(
    state: &MyAppState,
    actor: &AppCtx,
    user_id: UserId,
    group_id: GroupId,
) -> Result<()> {
    assert_admin(actor)?;
    let group = state.users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Cannot remove a user from their personal group".into(),
        ));
    }
    state
        .users()
        .remove_user_from_group(user_id, group_id)
        .await?;
    Ok(())
}
