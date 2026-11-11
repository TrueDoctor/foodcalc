use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    #[serde(with = "time::serde::rfc3339")]
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
    pub is_personal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub user_id: i64,
    pub group_id: i32,
}

/// Raw `(user_id, group)` join row used by `UserOps::list_users_with_groups`
/// to group memberships in Rust with two flat queries.
#[derive(Debug)]
pub struct MembershipRow {
    pub user_id: i64,
    pub id: i32,
    pub name: String,
    pub is_personal: bool,
}

/// Group as seen by one member: the group plus that member's role in it
/// (`auth/me`, member lists). Role is one of `viewer`, `editor`, `manager`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMembership {
    pub id: i32,
    pub name: String,
    pub is_personal: bool,
    pub role: String,
}

impl GroupMembership {
    pub fn group(&self) -> Group {
        Group {
            id: self.id,
            name: self.name.clone(),
            is_personal: self.is_personal,
        }
    }
}
