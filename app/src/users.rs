//! User & group read use-cases shared across the app layer.

use foodlib_new::entities::types::ids::{GroupId, UserId};
use foodlib_new::user::{Group, User};

use crate::{MyAppState, Result};

/// Ids of every group `user_id` belongs to.
pub async fn user_group_ids(state: &MyAppState, user_id: UserId) -> Result<Vec<GroupId>> {
    Ok(state
        .users()
        .get_user_groups(user_id)
        .await?
        .into_iter()
        .map(|g| g.id)
        .collect())
}

/// Group ids of `user`, degrading DB failures to an empty list — matches the
/// read-tolerant list views this feeds (they render "nothing visible"
/// instead of erroring).
pub async fn lenient_user_group_ids(state: &MyAppState, user: &User) -> Vec<GroupId> {
    user_group_ids(state, user.id)
        .await
        .unwrap_or_default()
}

pub async fn list_all_groups(state: &MyAppState) -> Result<Vec<Group>> {
    state.users().get_all_groups().await
}

pub async fn list_all_users(state: &MyAppState) -> Result<Vec<User>> {
    state.users().get_all_users().await
}

pub async fn get_user(state: &MyAppState, user_id: UserId) -> Result<User> {
    state.users().get_user(user_id).await
}

pub async fn get_user_groups(state: &MyAppState, user_id: UserId) -> Result<Vec<Group>> {
    state.users().get_user_groups(user_id).await
}

pub async fn get_group(state: &MyAppState, group_id: GroupId) -> Result<Group> {
    state.users().get_group(group_id).await
}

/// Members of a group.
pub async fn group_members(state: &MyAppState, group_id: GroupId) -> Result<Vec<User>> {
    foodlib_new::ops::users::members_of_group(&state.db, group_id).await
}
