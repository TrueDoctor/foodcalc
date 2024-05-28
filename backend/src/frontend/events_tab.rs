use axum::extract::{Form, State};
use maud::{html, Markup};
use serde::Deserialize;

mod event_detail_tab;

use crate::MyAppState;

pub(crate) fn events_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", axum::routing::get(event_list))
        .nest("/edit", event_detail_tab::event_detail_router())
        .route("/search", axum::routing::post(search))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn event_list(State(state): State<MyAppState>) -> Markup {
    let events = state.db_connection.get_events().await.unwrap_or_default();

    html! {
        div id="recipes" class="flex flex-col items-center justify-center mb-16" {
            div  class="w-3/4 flex flex-col items-center justify-center" {
                div class="
                    flex flex-row items-center justify-stretch
                    mb-2 gap-5 h-10
                    w-full
                    " {
                    input class="grow text h-full" type="search" placeholder="Search for event" id="search" name="search" autocomplete="off"
                        autofocus="autofocus" hx-post="/events/search" hx-trigger="keyup changed delay:20ms, search"
                        hx-target="#search-results" hx-indicator=".htmx-indicator";

                }
                div class = "grow-0 h-full m-2"
                    hx-target="this"  hx-swap="outerHTML" {
                    button class="btn btn-primary" hx-get="/recipes/add" { "Add recipe (+)" }
                }
                table class="w-full text-inherit table-auto object-center" {
                    // We add extra table headers to account for the buttons
                    thead { tr { th { "Name" } th { "Comment" } } }
                    tbody id="search-results" {
                        @for recipe in events.iter() {
                            (format_event(recipe))
                        }
                    }
                }
            }
        }
    }
}

pub async fn search(State(state): State<MyAppState>, query: Form<SearchParameters>) -> Markup {
    let query = query.search.to_lowercase();
    let events = state.db_connection.get_events().await.unwrap_or_default();

    let filtered_events = events
        .iter()
        .filter(|x| x.event_name.to_lowercase().contains(&query));

    html! {
        @for event in filtered_events {
            (format_event(event))
        }
    }
}

fn format_event(event: &foodlib::Event) -> Markup {
    html! {
        tr id=(format!("event-{}", event.event_id)) {
            td { (event.event_name) }
            td class="text-center" { (event.comment.clone().unwrap_or_default()) }
            td { button class="btn btn-primary" hx-target="#content" hx-get=(format!("/events/edit/{}", event.event_id)) {"Edit"} }
        }
    }
}
