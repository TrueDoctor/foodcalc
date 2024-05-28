use std::hash::Hasher;
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_login::RequireAuthorizationLayer;
use foodlib::User;
use maud::{html, Markup};

use crate::MyAppState;

pub type Router = axum::Router<MyAppState>;

mod events_tab;
mod home;
mod ingredients_tab;
mod inventories_tab;
mod login_tab;
mod recipes_tab;

pub(crate) const LOGIN_URL: &str = "/auth/login/form";

pub fn frontend_router() -> Router {
    let login_url = Arc::new(LOGIN_URL.into());
    Router::new()
        .nest("/inventories", inventories_tab::inventories_router())
        .nest("/events", events_tab::events_router())
        .route_layer(RequireAuthorizationLayer::<i64, User>::login_or_redirect(
            login_url, None,
        ))
        .nest("/", home::home_router())
        .nest("/ingredients", ingredients_tab::ingredients_router())
        .nest("/recipes", recipes_tab::recipes_router())
        .nest("/auth", login_tab::login_router())
        .route("/static/*-style.css", get(static_style))
}

thread_local! {
    static CSS_HASH: u64 = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(include_str!("index.css").as_bytes());
        hasher.finish()
    }
}

async fn static_style() -> impl IntoResponse {
    let style = include_str!("index.css");
    Response::builder()
        .header("Content-Type", "text/css")
        .body(style.to_owned())
        .unwrap()
}

pub fn html_error(reason: &str, redirect: &str) -> Markup {
    html! {
        dialog open="true"
        id="error"
        class="flex flex-col items-center justify-center text-red-500" {
            div {
                h1 { "Error" }
                p { "Failed to delete recipe" }
                p { (reason) }
                button class="btn btn-primary" hx-get="/recipes" hx-target="#content"  { "Back" }
            }
        }

    }
}
