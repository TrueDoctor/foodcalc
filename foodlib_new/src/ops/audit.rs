//! Audit log for security-relevant events (ISO 27001:2022 A.8.15).
//!
//! `AuditOps::record` is called by the API layer *after* a successful
//! mutation (documented trade-off: the entry is written post-commit, so a
//! failing audit write surfaces as an error log instead of blocking the
//! change). Auth events are recorded directly in the auth handlers.

use crate::error::Result;
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;

/// Closed set of audit action names, namespaced like permissions.
pub mod action {
    pub const LOGIN_SUCCESS: &str = "auth.login.success";
    pub const LOGIN_FAILED: &str = "auth.login.failed";
    pub const LOGOUT: &str = "auth.logout";
    pub const REGISTER: &str = "auth.register";

    pub const USER_CREATED: &str = "account.user.created";
    pub const USER_UPDATED: &str = "account.user.updated";
    pub const USER_DELETED: &str = "account.user.deleted";
    pub const PASSWORD_CHANGED: &str = "account.password.changed";
    pub const GROUP_MEMBER_ADDED: &str = "access.group.member.added";
    pub const GROUP_MEMBER_REMOVED: &str = "access.group.member.removed";
    pub const GROUP_MEMBER_ROLES_CHANGED: &str = "access.group.member.roles_changed";
    pub const EVENT_MEMBER_ADDED: &str = "access.event.member.added";
    pub const EVENT_MEMBER_REMOVED: &str = "access.event.member.removed";
    pub const EVENT_MEMBER_ROLES_CHANGED: &str = "access.event.member.roles_changed";
    pub const GLOBAL_PERMISSIONS_CHANGED: &str = "access.global.permissions_changed";
    pub const GROUP_DEFAULTS_CHANGED: &str = "access.group.defaults_changed";
    pub const SETTINGS_MEMBER_DEFAULTS_CHANGED: &str = "access.settings.member_defaults_changed";
    pub const ADMIN_SELF_GRANT: &str = "access.admin.self_grant";

    pub const OWNER_TRANSFERRED: &str = "ownership.owner.transferred";
    pub const ENTITY_CREATED: &str = "ownership.entity.created";
    pub const ENTITY_DELETED: &str = "ownership.entity.deleted";

    pub const AUDIT_READ: &str = "audit.read";
}

/// One audit log row as returned by the query endpoints.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct AuditEntry {
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub actor_user_id: Option<i64>,
    pub actor_username: String,
    pub action: String,
    pub target_user_id: Option<i64>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i32>,
    pub group_id: Option<i32>,
    pub event_id: Option<i32>,
    /// JSON object serialized as text (kept as `TEXT` to avoid requiring the
    /// sqlx `json` feature).
    pub details: String,
}

#[derive(Clone, Debug, Default)]
pub struct AuditRecord<'a> {
    pub actor_user_id: Option<i64>,
    pub actor_username: &'a str,
    pub action: &'a str,
    pub target_user_id: Option<i64>,
    pub entity_type: Option<&'a str>,
    pub entity_id: Option<i32>,
    pub group_id: Option<i32>,
    pub event_id: Option<i32>,
    pub details: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct AuditOps {
    pool: Arc<PgPool>,
}

impl AuditOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Appends one audit entry. Errors are surfaced to the caller (which
    /// logs them) — audit writes must never be silently swallowed.
    pub async fn record(&self, record: AuditRecord<'_>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_log
                (actor_user_id, actor_username, action, target_user_id,
                 entity_type, entity_id, group_id, event_id, details)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            record.actor_user_id,
            record.actor_username,
            record.action,
            record.target_user_id,
            record.entity_type,
            record.entity_id,
            record.group_id,
            record.event_id,
            record
                .details
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "{}".to_string())
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    /// Filtered, newest-first audit entries.
    pub async fn list(&self, filter: AuditFilter<'_>) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query_as!(
            AuditEntry,
            r#"
            SELECT id, created_at, actor_user_id, actor_username, action,
                   target_user_id, entity_type, entity_id, group_id, event_id, details
            FROM audit_log
            WHERE ($1::timestamptz IS NULL OR created_at >= $1)
              AND ($2::timestamptz IS NULL OR created_at <= $2)
              AND ($3::bigint IS NULL OR actor_user_id = $3)
              AND ($4::text IS NULL OR action LIKE $4)
              AND ($5::int IS NULL OR group_id = $5)
              AND ($6::int IS NULL OR event_id = $6)
            ORDER BY created_at DESC, id DESC
            LIMIT $7 OFFSET $8
            "#,
            filter.from,
            filter.until,
            filter.actor_user_id,
            filter.action_prefix.map(|p| format!("{p}%")),
            filter.group_id,
            filter.event_id,
            filter.limit as i64,
            filter.offset as i64
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    /// Total number of entries matching the filter (for pagination UIs).
    pub async fn count(&self, filter: AuditFilter<'_>) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM audit_log
            WHERE ($1::timestamptz IS NULL OR created_at >= $1)
              AND ($2::timestamptz IS NULL OR created_at <= $2)
              AND ($3::bigint IS NULL OR actor_user_id = $3)
              AND ($4::text IS NULL OR action LIKE $4)
              AND ($5::int IS NULL OR group_id = $5)
              AND ($6::int IS NULL OR event_id = $6)
            "#,
            filter.from,
            filter.until,
            filter.actor_user_id,
            filter.action_prefix.map(|p| format!("{p}%")),
            filter.group_id,
            filter.event_id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(count)
    }

    /// Deletes entries older than the given date (retention policy).
    pub async fn purge_older_than(&self, until: OffsetDateTime) -> Result<u64> {
        let result = sqlx::query!(
            r#"DELETE FROM audit_log WHERE created_at <= $1"#,
            until
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Filter for `list` and `count`; `None` disables the condition.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditFilter<'a> {
    pub from: Option<OffsetDateTime>,
    pub until: Option<OffsetDateTime>,
    pub actor_user_id: Option<i64>,
    /// Matched as `prefix%` (e.g. `access.` for the whole access namespace).
    pub action_prefix: Option<&'a str>,
    pub group_id: Option<i32>,
    pub event_id: Option<i32>,
    pub limit: i32,
    pub offset: i32,
}
