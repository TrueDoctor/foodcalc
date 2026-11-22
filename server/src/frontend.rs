use std::hash::Hasher;

use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_login::login_required;
use foodlib_new::auth::AuthBackend;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::types::ids::GroupId;
use maud::Markup;

use crate::MyAppState;

pub type Router = axum::Router<MyAppState>;

pub(crate) mod admin_tab;
pub(crate) mod ctx;
pub(crate) mod form;
pub(crate) mod events_tab;
pub(crate) mod home;
pub(crate) mod icons;
pub(crate) mod ingredients_tab;
pub(crate) mod inventories_tab;
pub(crate) mod login_tab;
pub(crate) mod move_group;
pub(crate) mod recipes_tab;

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
    pub (crate) static CSS_HASH: u64 = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(include_str!("index.css").as_bytes());
        hasher.finish()
    };
    /// Content hash of the timeline JS, used in its URL for cache-busting: the
    /// browser refetches whenever the file content changes (mirrors CSS_HASH).
    pub (crate) static TIMELINE_JS_HASH: u64 = {
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
pub type MResponse = foodlib_new::Result<Markup>;
pub type IResponse = foodlib_new::Result<Response>;

pub use foodlib_new::error::redirect_html_error as html_error;

/// Group ids the user holds any membership in.
///
/// Returns plain membership — it does *not* imply any permission. It backs
/// both the UI's offer-filtering and the `can_edit` ingredient gate (which
/// resolves the group to `IngredientPerm::Edit`); authoritative checks stay
/// in `AuthCtx` (`can_access_group` / `assert_group_perm`). Membership now
/// lives in the resolved `UserPolicy`, not on `AuthCtx` itself.
pub(crate) fn user_group_ids(ctx: &AuthCtx) -> Vec<GroupId> {
    ctx.policy.group_permissions.keys().copied().collect()
}
