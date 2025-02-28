use axum::response::IntoResponse;
use foodlib_new::user::User;
use maud::{html, Markup};

use crate::{
    frontend::{ingredients_tab::ingredients_view, LOGIN_URL},
    FoodLib,
};

pub(crate) fn home_router() -> axum::Router<crate::MyAppState> {
    axum::Router::new().route("/", axum::routing::get(home_view))
}

#[axum::debug_handler]
pub async fn home_view(user: Option<User>, foodlib: FoodLib) -> impl IntoResponse {
    (
        [("HX-Replace-Url", "ingredients")],
        content(foodlib, user).await,
    )
}

pub async fn content(foodlib: FoodLib, user: Option<User>) -> Markup {
    html! {
        div class="flex flex-col items-center justify-center mb-16" {
            div id="content" class="w-3/4 flex flex-col items-center justify-center" {
                (ingredients_view(foodlib, user).await)
            }
        }
    }
}

pub fn navbar(user: Option<User>) -> Markup {
    html! {
        //Warn if using production database
        @if std::env::var("DATABASE_URL").unwrap().ends_with("food_calc") {
            dialog open="true" class="bg-btn-cancel-normal text-white rounded-lg w-full" id="banner" {
                div class="p-4 flex flex-col items-center justify-center gap-4" {
                    p class="text-2xl" {
                        "WARNING: You are using the production database!"
                    }
                    p class="text-lg" {
                        (std::env::var("DATABASE_URL").unwrap())
                    }
                    button class="btn bg-black w-1/2" hx-on:click="document.getElementById('banner').remove()" {
                        "Close"
                    }
                }
            }
        } @else {
            div class="bg-green-800 text-white p-4 rounded-lg flex items-center justify-between" id="banner"{
                //print the database url
                "Database URL: "
                (std::env::var("DATABASE_URL").unwrap())
                button class="bg-dark-primary-normal text-white p-2 rounded-lg" hx-on:click="document.getElementById('banner').remove()" {
                    "Close"
                }
            }
        }

        div class="
            flex items-center justify-between flex-wrap 
            bg-navbar text-white 
            mx-16 my-4 
            rounded-xl shadow-xl overflow-hidden
        " {
            (navbutton("Ingredients", "/ingredients"))
            (navbutton("Recipes", "/recipes"))
            @if user.is_some() {
                (navbutton("Events", "/events"))
                (navbutton("Inventories", "/inventories"))
            }
            // (navbutton("Stores", "/stores"))
            a hx-get=(LOGIN_URL) hx-target="#content" class="flex flex-col items-center
                transition ease-in-out transition duration-200 
                rounded-xl p-6
                hover:shadow-inner hover:bg-blue-800" {
                    svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" class="h-6 w-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {
                        path d="M217.9 105.9L340.7 228.7c7.2 7.2 11.3 17.1 11.3 27.3s-4.1 20.1-11.3 27.3L217.9 406.1c-6.4 6.4-15 9.9-24 9.9c-18.7 0-33.9-15.2-33.9-33.9l0-62.1L32 320c-17.7 0-32-14.3-32-32l0-64c0-17.7 14.3-32 32-32l128 0 0-62.1c0-18.7 15.2-33.9 33.9-33.9c9 0 17.6 3.6 24 9.9zM352 416l64 0c17.7 0 32-14.3 32-32l0-256c0-17.7-14.3-32-32-32l-64 0c-17.7 0-32-14.3-32-32s14.3-32 32-32l64 0c53 0 96 43 96 96l0 256c0 53-43 96-96 96l-64 0c-17.7 0-32-14.3-32-32s14.3-32 32-32z" {}
                    }
            }
        }
    }
}

fn navbutton(text: &str, link: &str) -> Markup {
    html! {
        a hx-get=(link) hx-target="#content" class="flex flex-col items-center grow
            transition ease-in-out transition duration-200 ease-in-out
            rounded-xl p-6
            hover:shadow-inner hover:bg-blue-800" { (text) }
    }
}
