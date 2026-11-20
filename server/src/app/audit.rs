//! Audit use-cases for the application layer.
//!
//! Two concerns live here:
//!
//! 1. `record` — the audit-write helper every use-case calls, stamping the
//!    request [`Channel`] into the entry's `details` JSON (ISO 27001:2022
//!    A.8.15).
//! 2. The audit-log *read* use-cases (list, CSV export, retention purge).
//!    Reading the log requires `admin:view_audit_log` and every read is
//!    itself audited (`audit.read`); purging requires `admin:manage_settings`
//!    and is meant to be driven by an explicit ISMS decision.

use serde::Deserialize;
use time::OffsetDateTime;

pub(crate) use foodlib_new::ops::audit::{action, AuditFilter, AuditRecord};
use foodlib_new::policy::GlobalPermission;

use super::{AppCtx, Error, MyAppState, Result};

/// Records one audit entry with the actor's channel stamped into `details`.
/// Audit writes are post-commit and never block the change (documented
/// trade-off of `AuditOps::record`); failures are logged.
#[allow(clippy::too_many_arguments)]
pub async fn record(
    state: &MyAppState,
    actor: &AppCtx,
    action: &'static str,
    entity_type: Option<&str>,
    entity_id: Option<i32>,
    group_id: Option<i32>,
    event_id: Option<i32>,
    mut details: Option<serde_json::Value>,
) {
    let channel = serde_json::json!(actor.channel.as_str());
    match &mut details {
        Some(value @ serde_json::Value::Object(_)) => {
            value
                .as_object_mut()
                .expect("checked above")
                .insert("channel".into(), channel);
        }
        _ => details = Some(serde_json::json!({ "channel": channel })),
    }

    if let Err(err) = state
        .audit()
        .record(AuditRecord {
            actor_user_id: Some(actor.ctx.user.id),
            actor_username: &actor.ctx.user.username,
            action,
            target_user_id: None,
            entity_type,
            entity_id,
            group_id,
            event_id,
            details,
        })
        .await
    {
        log::error!("audit write for `{action}` failed: {err}");
    }
}

// ------------------------------------------------------------ read use-cases

#[derive(Debug, Deserialize)]
pub struct AuditListQuery {
    pub from: Option<OffsetDateTime>,
    pub until: Option<OffsetDateTime>,
    pub actor_user_id: Option<i64>,
    /// Action namespace prefix (e.g. `access.` or `auth.`).
    pub action: Option<String>,
    pub group_id: Option<i32>,
    pub event_id: Option<i32>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl AuditListQuery {
    fn filter(&self) -> AuditFilter<'_> {
        AuditFilter {
            from: self.from,
            until: self.until,
            actor_user_id: self.actor_user_id,
            action_prefix: self.action.as_deref(),
            group_id: self.group_id,
            event_id: self.event_id,
            limit: self.limit.unwrap_or(100).min(1000),
            offset: self.offset.unwrap_or(0).max(0),
        }
    }
}

/// Filtered audit entries plus the total count. Requires
/// `admin:view_audit_log`; the read itself is audited (`audit.read`).
pub async fn list(
    state: &MyAppState,
    ctx: &AppCtx,
    query: AuditListQuery,
) -> Result<serde_json::Value> {
    ctx.ctx.assert_global(GlobalPermission::ViewAuditLog)?;
    let audit = state.audit();
    let filter = query.filter();
    let (entries, total) = tokio::try_join!(audit.list(filter), audit.count(filter))?;
    if let Err(err) = audit
        .record(AuditRecord {
            actor_user_id: Some(ctx.ctx.user.id),
            actor_username: &ctx.ctx.user.username,
            action: action::AUDIT_READ,
            details: Some(serde_json::json!({ "total": total })),
            ..Default::default()
        })
        .await
    {
        log::error!("audit read not logged: {err}");
    }
    Ok(serde_json::json!({ "entries": entries, "total": total }))
}

/// CSV of the filtered entries (ISMS evidence, max 10 000 rows). Requires
/// `admin:view_audit_log`.
pub async fn export_csv(state: &MyAppState, ctx: &AppCtx, query: AuditListQuery) -> Result<String> {
    ctx.ctx.assert_global(GlobalPermission::ViewAuditLog)?;
    let mut filter = query.filter();
    filter.limit = 10_000;
    filter.offset = 0;
    let entries = state.audit().list(filter).await?;

    let mut csv = String::from(
        "id,created_at,actor_user_id,actor_username,action,target_user_id,entity_type,entity_id,group_id,event_id,details\n",
    );
    for entry in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            entry.id,
            entry.created_at,
            opt(entry.actor_user_id),
            csv_escape(&entry.actor_username),
            csv_escape(&entry.action),
            opt(entry.target_user_id),
            opt(entry.entity_type.as_deref()),
            opt(entry.entity_id),
            opt(entry.group_id),
            opt(entry.event_id),
            csv_escape(&entry.details),
        ));
    }
    Ok(csv)
}

/// Retention cleanup of entries older than `until`. Requires
/// `admin:manage_settings`; the purge is audited (`audit.purged`).
pub async fn purge(
    state: &MyAppState,
    ctx: &AppCtx,
    until: Option<OffsetDateTime>,
) -> Result<u64> {
    ctx.ctx.assert_global(GlobalPermission::ManageSettings)?;
    let Some(until) = until else {
        return Err(Error::Validation {
            message: "`until` is required to purge audit entries".into(),
        });
    };
    let deleted = state.audit().purge_older_than(until).await?;
    let details = serde_json::json!({ "purged": deleted, "until": until });
    if let Err(err) = state
        .audit()
        .record(AuditRecord {
            actor_user_id: Some(ctx.ctx.user.id),
            actor_username: &ctx.ctx.user.username,
            action: "audit.purged",
            details: Some(details),
            ..Default::default()
        })
        .await
    {
        log::error!("audit purge not logged: {err}");
    }
    Ok(deleted)
}

fn opt<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
