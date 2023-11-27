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
        body class="
            bg-white text-gray-800
            dark:bg-gray-900 dark:text-gray-100" {
            div {
                (navbar())
                (content(State(state)).await)
            }
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
        div class="
            rounded-xl 
            flex items-center justify-center flex-wrap p-6 mx-16 my-4 gap-24
            bg-blue-700 text-white 
            " {
             a href="/" { "Home" }
             a href="/ingredients" { "Ingredients" }
             a href="/recipes" { "Recipes" }
             a href="/events" { "Events" }
             a href="/stores" { "Stores" }
        }
    }
}
