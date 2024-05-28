use super::LOGIN_URL;
use axum::extract::{Host, State};
use foodlib::{AuthContext, User};
use maud::{html, Markup};

use crate::{
    frontend::{ingredients_tab::ingredients_view, CSS_HASH},
    MyAppState,
};

pub(crate) fn home_router() -> axum::Router<crate::MyAppState> {
    axum::Router::new().route("/", axum::routing::get(home_view))
}

pub async fn home_view(mut auth: AuthContext, host: Host, state: State<MyAppState>) -> Markup {
    let (host, _) = host.0.split_once(':').unwrap_or_default();
    #[cfg(debug_assertions)]
    if host == "127.0.0.1" || host == "localhost" {
        let user = User {
            username: "test".into(),
            id: 0,
            is_admin: true,
            // TODO: replace this with a better way to handle this
            password_hash: String::from("password"),
            ..Default::default()
        };
        auth.login(&user).await.unwrap();
        log::info!("logged in test user");
    }
    html! {
        head {
            title { "Foodbase" }
            link rel="stylesheet" href=(format!("/static/{}-style.css", CSS_HASH.with(|x| *x))) {}
            script src="https://unpkg.com/htmx.org@1.9.6" {}
            script src="https://unpkg.com/htmx.org@1.9.12/dist/ext/debug.js" {}
        }
        body class="
            bg-light-bg-light text-gray-800
            dark:bg-dark-bg-dark dark:text-gray-100" {
            div {
                (navbar())
                (content(state).await)
            }
        }
    }
}

pub async fn content(State(state): State<MyAppState>) -> Markup {
    html! {
        div class="flex flex-col items-center justify-center mb-16" {
            div id="content" class="w-3/4 flex flex-col items-center justify-center" {
                (ingredients_view(State(state)).await)
            }
        }
    }
}

pub fn navbar() -> Markup {
    {
        extern crate alloc;
        extern crate maud;
        let mut __maud_output = alloc::string::String::with_capacity(1288usize);
        __maud_output.push_str("<div class=\"\n            flex items-center justify-between flex-wrap \n            bg-navbar text-white \n            mx-16 my-4 \n            rounded-xl shadow-xl overflow-hidden\n            \">");
        maud::macro_private::render_to!(
            &navbutton("Ingredients", "/ingredients"),
            &mut __maud_output
        );
        maud::macro_private::render_to!(&navbutton("Recipes", "/recipes"), &mut __maud_output);
        maud::macro_private::render_to!(&navbutton("Events", "/events"), &mut __maud_output);
        maud::macro_private::render_to!(
            &navbutton("Inventories", "/inventories"),
            &mut __maud_output
        );
        maud::macro_private::render_to!(&navbutton("Stores", "/stores"), &mut __maud_output);
        __maud_output.push_str("<a hx-get=\"");
        maud::macro_private::render_to!(&LOGIN_URL, &mut __maud_output);
        __maud_output.push_str("\" hx-target=\"#content\" class=\"flex flex-col items-center\n                    transition ease-in-out transition duration-200 \n                    rounded-xl p-6\n                    hover:shadow-inner hover:bg-blue-800\"><svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 512 512\" class=\"h-6 w-6\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M217.9 105.9L340.7 228.7c7.2 7.2 11.3 17.1 11.3 27.3s-4.1 20.1-11.3 27.3L217.9 406.1c-6.4 6.4-15 9.9-24 9.9c-18.7 0-33.9-15.2-33.9-33.9l0-62.1L32 320c-17.7 0-32-14.3-32-32l0-64c0-17.7 14.3-32 32-32l128 0 0-62.1c0-18.7 15.2-33.9 33.9-33.9c9 0 17.6 3.6 24 9.9zM352 416l64 0c17.7 0 32-14.3 32-32l0-256c0-17.7-14.3-32-32-32l-64 0c-17.7 0-32-14.3-32-32s14.3-32 32-32l64 0c53 0 96 43 96 96l0 256c0 53-43 96-96 96l-64 0c-17.7 0-32-14.3-32-32s14.3-32 32-32z\"></path></svg></a></div>");
        maud::PreEscaped(__maud_output)
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
