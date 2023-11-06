use axum::extract::State;
use maud::{html, Markup};

use crate::{frontend::ingredients_tab::ingredients_view, MyAppState};

pub(crate) fn home_router() -> axum::Router<crate::MyAppState> {
    axum::Router::new().route("/", axum::routing::get(home_view))
}

pub async fn home_view(State(state): State<MyAppState>) -> Markup {
    html! {
        h1 { "Home" }
        p { "Welcome to Foodbase!" }
        div {
            (ingredients_view(State(state)).await)
        }
    }
}
