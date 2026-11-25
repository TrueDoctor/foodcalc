use std::hash::Hasher;

use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_login::login_required;
use foodlib_new::error::Error;
use foodlib_new::{AppCtx, AppState, Channel};
use maud::Markup;

use crate::auth::{AuthBackend, AuthSession};
use crate::frontend::web_error::WebError;

pub type Router = axum::Router<AppState>;

pub(crate) mod admin_tab;
pub(crate) mod events_tab;
pub(crate) mod home;
pub(crate) mod icons;
pub(crate) mod ingredients_tab;
pub(crate) mod inventories_tab;
pub(crate) mod login_tab;
pub(crate) mod move_group;
pub(crate) mod recipes_tab;
pub mod web_error;

pub(crate) const LOGIN_URL: &str = "/auth/login/form";

pub fn frontend_router() -> Router {
    Router::new()
        .nest("/inventories", inventories_tab::inventories_router())
        .nest("/events", events_tab::events_router())
        .nest("/admin", admin_tab::admin_router())
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
        .merge(home::home_router())
        .nest("/ingredients", ingredients_tab::ingredients_router())
        .nest("/recipes", recipes_tab::recipes_router())
        .nest("/auth", login_tab::login_router())
        .route("/static/{*-style.css}", get(static_style))
        .route("/static/htmxv2.0.4.js", get(static_htmx))
        .route("/static/sortable.js", get(static_sortable))
        .route("/static/timeline.js", get(static_timeline))
}

thread_local! {
    pub(crate) static CSS_HASH: u64 = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(include_str!("index.css").as_bytes());
        hasher.finish()
    };
    /// Content hash of the timeline JS, used in its URL for cache-busting: the
    /// browser refetches whenever the file content changes (mirrors CSS_HASH).
    pub(crate) static TIMELINE_JS_HASH: u64 = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(include_str!("../assets/timeline.js").as_bytes());
        hasher.finish()
    };
}

async fn static_style() -> impl IntoResponse {
    let style = include_str!("index.css");
    Response::builder()
        .header("Content-Type", "text/css")
        .body(style.to_owned())
        .unwrap()
}

async fn static_htmx() -> impl IntoResponse {
    let htmx = include_str!("../assets/htmxv2.0.4.js");
    Response::builder()
        .header("Content-Type", "text/javascript")
        .body(htmx.to_owned())
        .unwrap()
}

async fn static_sortable() -> impl IntoResponse {
    let js = include_str!("../assets/Sortable-1.15.6.min.js");
    Response::builder()
        .header("Content-Type", "text/javascript")
        .body(js.to_owned())
        .unwrap()
}

async fn static_timeline() -> impl IntoResponse {
    let js = include_str!("../assets/timeline.js");
    Response::builder()
        .header("Content-Type", "text/javascript")
        .body(js.to_owned())
        .unwrap()
}

pub type MResponse = std::result::Result<Markup, WebError>;
pub type IResponse = std::result::Result<Response, WebError>;

pub use web_error::redirect_html_error as html_error;

/// Required authenticated actor context (`Channel::WebUi`).
pub struct WebCtx(pub AppCtx);

impl<S> FromRequestParts<S> for WebCtx
where
    S: Send + Sync,
    AppState: FromRef<S>,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, WebError> {
        let OptWebCtx(ctx) = OptWebCtx::from_request_parts(parts, state).await?;
        let ctx = ctx.ok_or_else(|| {
            WebError(Error::Unauthorized("Authentication required".to_string()))
        })?;
        Ok(WebCtx(ctx))
    }
}

/// Optional authenticated actor context (`Channel::WebUi`) for routes that
/// render both logged-in and anonymous views.
pub struct OptWebCtx(pub Option<AppCtx>);

impl<S> FromRequestParts<S> for OptWebCtx
where
    S: Send + Sync,
    AppState: FromRef<S>,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = WebError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, WebError> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| WebError(Error::Unauthorized("no auth headers".to_string())))?;

        let Some(user) = auth.user else {
            return Ok(OptWebCtx(None));
        };

        let State(app_state): State<AppState> = State::from_request_parts(parts, state)
            .await
            .map_err(|_| WebError(Error::Misc("AppState missing".into())))?;

        let ctx = app_state
            .build_ctx(user.0, Channel::WebUi)
            .await
            .map_err(WebError)?;
        Ok(OptWebCtx(Some(ctx)))
    }
}