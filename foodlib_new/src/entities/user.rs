use crate::types::ids::{GroupId, UserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: OffsetDateTime,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: UserId::invalid(),
            username: "test".into(),
            email: "test@example.com".into(),
            password_hash: "test".into(),
            is_admin: false,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub is_personal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub user_id: UserId,
    pub group_id: GroupId,
}
