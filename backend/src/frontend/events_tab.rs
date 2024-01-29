use axum::extract::{State, Form};
use maud::{html, Markup};
use serde::Deserialize;

use crate::MyAppState;

pub(crate) fn events_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/search", axum::routing::post(search))
}

#[derive (Deserialize)]
pub struct SearchParameters {
    search: String
}

pub async fn search(State(state): State<MyAppState>, query: Form<SearchParameters>) -> Markup {
    let query = query.search.to_lowercase();
    let events = state
        .db_connection
        .get_events()
        .await
        .unwrap_or_default();

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
        }
    }
}