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
            bg-light-bg-light text-gray-800
            dark:bg-dark-bg-dark dark:text-gray-100" {
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
            flex items-center justify-between flex-wrap 
            bg-navbar text-white 
            mx-16 my-4 
            rounded-xl shadow-xl overflow-hidden
            " {
                (navbutton("Ingredients", "/ingredients"))
                (navbutton("Recipes", "/recipes"))
                (navbutton("Events", "/events"))
                (navbutton("Inventories", "/inventories"))
                (navbutton("Stores", "/stores"))
                (navbutton("Login", "/auth/login"))
        }
    }
}

fn navbutton(text: &str, link: &str) -> Markup {
    html! {
        a class="transition ease-in-out transition duration-200 ease-in-out rounded-xl hover:shadow-inner hover:bg-blue-800 p-6 grow text-center" hx-get=(link) hx-target="#content" { (text) }
    }
}

//pub fn navbar() -> Markup {
//    html! {
//        div class="
//            rounded-xl
//            flex items-center justify-around flex-wrap
//            mx-16 my-4
//            gap-24
//            bg-blue-700 text-white
//            " {
//             a class="hover:bg-blue-500 p-6 round-lg" href="/" { "Home" }
//             a class="hover:bg-blue-500 p-6 round-lg" href="/ingredients" { "Ingredients" }
//             a class="hover:bg-blue-500 p-6 round-lg" href="/recipes" { "Recipes" }
//             a class="hover:bg-blue-500 p-6 round-lg" href="/events" { "Events" }
//             a class="hover:bg-blue-500 p-6 round-lg" href="/stores" { "Stores" }
//        }
//    }
//}
