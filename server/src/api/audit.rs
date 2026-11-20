//! Audit log endpoints (ISO 27001:2022 A.8.15 / A.5.28).
//!
//! Reading the log requires `admin:view_audit_log`; every read is itself
//! audited (`audit.read`). Purging old entries (retention policy) requires
//! `admin:manage_settings` and is meant to be driven by an explicit ISMS
//! decision, not by habit.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use serde::Deserialize;
use time::OffsetDateTime;

use foodlib_new::{
    auth_context::AuthCtx,
    error::Error,
    ops::audit::{action, AuditFilter, AuditRecord},
    policy::GlobalPermission,
};

use crate::api::map_api_error;
use crate::MyAppState;

pub fn audit_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_audit))
        .route("/export", get(export_audit))
        .route("/purge", delete(purge_audit))
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
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

impl AuditQuery {
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

/// `GET /api/audit` — filtered entries plus the total count.
async fn list_audit(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<AuditQuery>,
) -> Response {
    if let Err(err) = ctx.assert_global(GlobalPermission::ViewAuditLog) {
        return map_api_error(err).into_response();
    }
    let audit = state.audit();
    let filter = query.filter();
    match tokio::try_join!(audit.list(filter), audit.count(filter)) {
        Ok((entries, total)) => {
            if let Err(err) = audit
                .record(AuditRecord {
                    actor_user_id: Some(ctx.user.id),
                    actor_username: &ctx.user.username,
                    action: action::AUDIT_READ,
                    details: Some(serde_json::json!({ "total": total })),
                    ..Default::default()
                })
                .await
            {
                log::error!("audit read not logged: {err}");
            }
            Json(serde_json::json!({ "entries": entries, "total": total })).into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/audit/export` — CSV of the filtered entries (ISMS evidence).
async fn export_audit(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<AuditQuery>,
) -> Response {
    if let Err(err) = ctx.assert_global(GlobalPermission::ViewAuditLog) {
        return map_api_error(err).into_response();
    }
    let mut filter = query.filter();
    filter.limit = 10_000;
    filter.offset = 0;
    match state.audit().list(filter).await {
        Ok(entries) => {
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
            (
                StatusCode::OK,
                [
                    ("Content-Type", "text/csv; charset=utf-8"),
                    (
                        "Content-Disposition",
                        "attachment; filename=\"foodcalc-audit-log.csv\"",
                    ),
                ],
                csv,
            )
                .into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/audit/purge?until=<rfc3339>` — retention cleanup; guard
/// `admin:manage_settings`.
async fn purge_audit(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Query(query): Query<AuditQuery>,
) -> Response {
    if let Err(response) = ctx.assert_global(GlobalPermission::ManageSettings) {
        return map_api_error(response).into_response();
    }
    let Some(until) = query.until else {
        return map_api_error(Error::Validation {
            message: "`until` is required to purge audit entries".into(),
        })
        .into_response();
    };
    match state.audit().purge_older_than(until).await {
        Ok(deleted) => {
            let details = serde_json::json!({ "purged": deleted, "until": until });
            if let Err(err) = state
                .audit()
                .record(AuditRecord {
                    actor_user_id: Some(ctx.user.id),
                    actor_username: &ctx.user.username,
                    action: "audit.purged",
                    details: Some(details),
                    ..Default::default()
                })
                .await
            {
                log::error!("audit purge not logged: {err}");
            }
            Json(serde_json::json!({ "deleted": deleted })).into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
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
