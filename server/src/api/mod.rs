use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use foodlib_new::auth::AuthSession;
use serde_json::json;

use crate::MyAppState;

pub mod auth;
pub mod events;
pub mod ingredients;
pub mod inventories;
pub mod recipes;

pub fn api_router() -> Router<MyAppState> {
    // Public routes: health probe and the login entry point. Everything
    // else is gated by `require_api_auth` — keep new resource routers in
    // the `protected` router below so they stay guarded.
    let public = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(auth::login));

    let protected = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .nest("/events", events::events_router())
        .nest("/recipes", recipes::recipes_router())
        .nest("/ingredients", ingredients::ingredients_router())
        .nest("/inventories", inventories::inventories_router());

    // `route_layer` applies the middleware to all routes registered
    // above on `protected` (and skips non-matching paths, so unknown
    // `/api/*` paths still 404). `login` must stay reachable without a
    // session, hence it is mounted on `public` instead.
    public.merge(protected.route_layer(middleware::from_fn(require_api_auth)))
}

/// Middleware guarding `/api` routes: answers `401 { "error": ... }`
/// instead of a redirect so `fetch`-based clients (the SPA) can react
/// programmatically. The maud frontend keeps using its own redirect
/// behavior via `login_required!`.
pub async fn require_api_auth(auth: AuthSession, req: Request, next: Next) -> Response {
    if auth.user.is_some() {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "authentication required" })),
        )
            .into_response()
    }
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "foodcalc-api",
        })),
    )
}

pub fn map_api_error(err: foodlib_new::Error) -> (StatusCode, Json<serde_json::Value>) {
    let status = match err {
        foodlib_new::Error::NotFound { .. } => StatusCode::NOT_FOUND,
        foodlib_new::Error::Unauthorized(_) | foodlib_new::Error::Forbidden(_) => {
            StatusCode::UNAUTHORIZED
        }
        foodlib_new::Error::Validation { .. } | foodlib_new::Error::Conflict { .. } => {
            StatusCode::BAD_REQUEST
        }
        foodlib_new::Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };

    (
        status,
        Json(json!({
            "error": err.to_string(),
        })),
    )
}

