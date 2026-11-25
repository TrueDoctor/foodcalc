use crate::ctx::AppCtx;
use crate::entities::user::{Group, User};
use crate::types::ids::{GroupId, UserId};
use crate::{AppState, Error, Result};

/// Admins may manage any group. Members may manage groups they belong to.
/// Personal groups are never manageable here — they live and die with their
/// user. Returns the group on success.
pub async fn assert_can_manage_group(
    state: &AppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<Group> {
    let group = state.db().users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Personal groups can only be managed via their owning user".into(),
        ));
    }
    if !actor.can_access_group(group_id) {
        return Err(Error::Forbidden(
            "You don't have permission to manage this group".into(),
        ));
    }
    Ok(group)
}

/// Creates a shared group and adds the creating user as its first member.
/// The name is trimmed; an empty name is rejected.
pub async fn create_group(
    state: &AppState,
    actor: &AppCtx,
    name: String,
) -> Result<Group> {
    actor.require_human()?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation {
            message: "Group name cannot be empty".into(),
        });
    }
    let group = state.db().users().create_group(trimmed).await?;
    state
        .db()
        .users()
        .add_user_to_group(actor.user.id, group.id)
        .await?;
    Ok(group)
}

/// Deletes a shared group.
/// Requires group-management rights (see `assert_can_manage_group`).
pub async fn delete_group(
    state: &AppState,
    actor: &AppCtx,
    group_id: GroupId,
) -> Result<()> {
    actor.require_human()?;
    assert_can_manage_group(state, actor, group_id).await?;
    state.db().users().delete_group(group_id).await?;
    Ok(())
}

/// Adds a user to a shared group.
/// Requires group-management rights.
pub async fn add_member(
    state: &AppState,
    actor: &AppCtx,
    group_id: GroupId,
    user_id: UserId,
) -> Result<Group> {
    actor.require_human()?;
    let group = assert_can_manage_group(state, actor, group_id).await?;
    state.db().users().add_user_to_group(user_id, group_id).await?;
    Ok(group)
}

/// Removes a user from a shared group.
/// Requires group-management rights.
pub async fn remove_member(
    state: &AppState,
    actor: &AppCtx,
    group_id: GroupId,
    user_id: UserId,
) -> Result<()> {
    actor.require_human()?;
    assert_can_manage_group(state, actor, group_id).await?;
    state
        .db()
        .users()
        .remove_user_from_group(user_id, group_id)
        .await?;
    Ok(())
}

/// Adds a user to a shared group. Personal groups cannot have additional
/// members. Requires admin privileges.
pub async fn add_user_to_group(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
    group_id: GroupId,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    let group = state.db().users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Cannot add a user to a personal group".into(),
        ));
    }
    state.db().users().add_user_to_group(user_id, group_id).await?;
    Ok(())
}

/// Removes a user from a group. Users cannot be removed from their personal
/// group. Requires admin privileges.
pub async fn remove_user_from_group(
    state: &AppState,
    actor: &AppCtx,
    user_id: UserId,
    group_id: GroupId,
) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    let group = state.db().users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Cannot remove a user from their personal group".into(),
        ));
    }
    state
        .db()
        .users()
        .remove_user_from_group(user_id, group_id)
        .await?;
    Ok(())
}

/// Fetches one group by id.
pub async fn get_group(state: &AppState, group_id: GroupId) -> Result<Group> {
    state.db().users().get_group(group_id).await
}

/// Lists every group in the system (used for admin ownership pickers).
pub async fn list_all_groups(state: &AppState) -> Result<Vec<Group>> {
    state.db().users().get_all_groups().await
}

/// Lists every group a user belongs to.
pub async fn get_user_groups(state: &AppState, user_id: UserId) -> Result<Vec<Group>> {
    state.db().users().get_user_groups(user_id).await
}

/// Lists all users belonging to `group_id`.
pub async fn group_members(state: &AppState, group_id: GroupId) -> Result<Vec<User>> {
    state.db().users().get_group_members(group_id).await
}
