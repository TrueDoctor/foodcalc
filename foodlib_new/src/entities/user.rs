use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: OffsetDateTime,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: -1,
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
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub user_id: i64,
    pub group_id: i32,
}
