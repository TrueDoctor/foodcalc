use axum::extract::State;
use maud::{html, Markup};

use crate::{
    frontend::{ingredients_tab::ingredients_view, inventories_tab::inventories_view, CSS_HASH},
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
            (inventories_view(State(state)).await)
        }
    }
}

pub fn navbar() -> Markup {
    html! {
        div class="
            flex items-center justify-between flex-wrap 
            bg-blue-700 text-white 
            mx-16 my-4 
            rounded-xl shadow-xl overflow-hidden
            " {
             a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" href="/" { "Home" }
             a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" href="/ingredients" { "Ingredients" }
             a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" href="/inventories" { "Inventories" }
             a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" href="/recipes" { "Recipes" }
             a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" href="/events" { "Events" }
             a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" href="/stores" { "Stores" }
        }
    }
}
