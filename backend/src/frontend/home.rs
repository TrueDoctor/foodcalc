use axum::extract::State;
use maud::{html, Markup};

use crate::{
    frontend::{ingredients_tab::ingredients_view, CSS_HASH},
    MyAppState,
};

pub(crate) fn home_router() -> axum::Router<crate::MyAppState> {
    axum::Router::new().route("/", axum::routing::get(home_view))
}

pub async fn home_view(State(state): State<MyAppState>) -> Markup {
    html! {
        head {
            title { "Foodbase" }
            link rel="stylesheet" href=(format!("/static/{}-style.css", CSS_HASH.with(|x| *x))) {};
            script src="https://unpkg.com/htmx.org@1.9.6" {}
        }
        body {
        h1 { "Home" }
        p { "Welcome to Foodbase!" }
            (navbar())
            (content(State(state)).await)
        }
    }
}

pub async fn content(State(state): State<MyAppState>) -> Markup {
    html! {
        div id="content" {
            (ingredients_view(State(state)).await)
        }
    }
}

pub fn navbar() -> Markup {
    html! {
        div id="navbar" {
            ul {
                li { a href="/" { "Home" } }
                li { a href="/ingredients" { "Ingredients" } }
                li { a href="/recipes" { "Recipes" } }
                li { a href="/events" { "Events" } }
                li { a href="/stores" { "Stores" } }
            }
        }
    }
}
