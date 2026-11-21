//! Audit log endpoints (ISO 27001:2022 A.8.15 / A.5.28) — thin handlers over
//! `crate::app::audit`. Permission checks, the read-audit and the retention
//! purge logic live in the app layer.

use axum::{
    extract::{Query, State},
    http::{header::CONTENT_DISPOSITION, header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};

use crate::api::respond;
use crate::app::{audit, AppCtx};
use crate::MyAppState;

// Query DTO shared with the app layer.
pub use crate::app::audit::AuditListQuery;

pub fn audit_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_audit))
        .route("/export", get(export_audit))
        .route("/purge", delete(purge_audit))
}

/// `GET /api/audit` — filtered entries plus the total count.
async fn list_audit(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Query(query): Query<AuditListQuery>,
) -> Response {
    respond(audit::list(&state, &ctx, query).await)
}

/// `GET /api/audit/export` — CSV of the filtered entries (ISMS evidence).
async fn export_audit(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Query(query): Query<AuditListQuery>,
) -> Response {
    match audit::export_csv(&state, &ctx, query).await {
        Ok(csv) => (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    CONTENT_DISPOSITION,
                    "attachment; filename=\"foodcalc-audit-log.csv\"",
                ),
            ],
            csv,
        )
            .into_response(),
        Err(err) => crate::api::map_api_error(err).into_response(),
    }
}

/// `DELETE /api/audit/purge?until=<rfc3339>` — retention cleanup; guard
/// `admin:manage_settings`.
async fn purge_audit(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Query(query): Query<AuditListQuery>,
) -> Response {
    match audit::purge(&state, &ctx, query.until).await {
        Ok(deleted) => Json(serde_json::json!({ "deleted": deleted })).into_response(),
        Err(err) => crate::api::map_api_error(err).into_response(),
    }
}
