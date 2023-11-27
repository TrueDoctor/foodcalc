use std::hash::Hasher;

use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::api::Router;

mod home;
mod ingredients_tab;
mod inventories_tab;

pub fn frontend_router() -> Router {
    Router::new()
        .nest("/", home::home_router())
        .nest("/ingredients", ingredients_tab::ingredients_router())
        .nest("/inventories", inventories_tab::inventories_router())
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
