use std::hash::Hasher;

use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_login::login_required;
use foodlib::Backend;
use maud::{html, Markup};

use crate::MyAppState;

pub type Router = axum::Router<MyAppState>;

pub(crate) mod events_tab;
pub(crate) mod home;
pub(crate) mod ingredients_tab;
pub(crate) mod inventories_tab;
pub(crate) mod login_tab;
pub(crate) mod recipes_tab;

pub(crate) const LOGIN_URL: &str = "/auth/login/form";

pub fn frontend_router() -> Router {
    Router::new()
        .nest("/inventories", inventories_tab::inventories_router())
        .nest("/events", events_tab::events_router())
        .route_layer(login_required!(Backend, login_url = LOGIN_URL))
        .nest("/", home::home_router())
        .nest("/ingredients", ingredients_tab::ingredients_router())
        .nest("/recipes", recipes_tab::recipes_router())
        .nest("/auth", login_tab::login_router())
        .route("/static/*-style.css", get(static_style))
}

thread_local! {
    pub (crate) static CSS_HASH: u64 = {
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
                p { (reason) }
                button class="btn btn-primary" hx-get=(redirect) hx-target="#content"  { "Back" }
            }
        }

    }
}
