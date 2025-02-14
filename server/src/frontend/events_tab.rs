use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Response},
};
use maud::{html, Markup};
use serde::Deserialize;

mod event_detail_tab;

use crate::MyAppState;

use super::html_error;

pub(crate) fn events_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", axum::routing::get(event_list))
        .route("/delete/:event_id", axum::routing::get(delete_dialog))
        .route("/delete/:event_id", axum::routing::delete(delete))
        .route("/add", axum::routing::put(add_form))
        .route("/add", axum::routing::post(add))
        .route("/duplicate/:event_id", axum::routing::post(duplicate))
        .nest("/edit", event_detail_tab::event_detail_router())
        .route("/search", axum::routing::post(search))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn add_form() -> Markup {
    html! {
        form hx-post="/events/add" hx-target="#content" hx-swap="innerHTML" {
            div class="flex flex-row gap-2" {
                input type="hidden" id="event_id" name="event_id" value="0";
                input class="text" type="text" id="event_name" name="event_name" required="required" placeholder="Event Name";
                input class="text" type="text" id="budget" name="budget" required="required" placeholder="Budget";
                input class="text" type="text" id="comment" name="comment" placeholder="Comment";
                button class="btn btn-primary" type="submit" { "Add event" }
            }
        }
    }
}

pub async fn add(State(state): State<MyAppState>, form: Form<foodlib::Event>) -> Response {
    match state
        .add_event(
            form.event_name.clone(),
            form.budget.clone(),
            form.comment.clone(),
        )
        .await
    {
        Ok(_) => (event_list(State(state)).await).into_response(),
        Err(_) => html_error("Failed to add event", "/events/add").into_response(),
    }
}

pub async fn duplicate(State(state): State<MyAppState>, Path(event_id): Path<i32>) -> Response {
    match state.new_lib().events().duplicate(event_id).await {
        Ok(id) => (event_detail_tab::event_form(State(state), Path(id)).await).into_response(),
        Err(err) => html_error(
            &format!("Failed to add event, because: {}", err),
            "/events/add",
        )
        .into_response(),
    }
}

pub async fn delete_dialog(State(state): State<MyAppState>, Path(event_id): Path<i32>) -> Markup {
    let event = state.get_event(event_id).await.unwrap_or_default();
    html! {
        div class="flex flex-col gap-2" {
            p class="text-2xl" { "Are you sure you want to delete this Event permanently?" }
            div class="flex flex-col gap-2" {
                p { "Event ID: " (event.event_id) }
                p { "Event Name: " (event.event_name) }
                p { "Comment: " (event.comment.clone().unwrap_or_default()) }
            }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get="/events" hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/delete/{}", event_id)) { "Confirm Delete" }
            }
        }
    }
}

pub async fn delete(State(state): State<MyAppState>, Path(event_id): Path<i32>) -> Response {
    match state.new_lib().events().delete(event_id).await {
        Ok(_) => (event_list(State(state)).await).into_response(),
        Err(e) => html_error(
            &format!(
                "Failed to delete Event with id {} because of DB Error {} ",
                event_id, e
            ),
            "/events",
        )
        .into_response(),
    }
}

pub async fn event_list(State(state): State<MyAppState>) -> Markup {
    let events = state.get_events().await.unwrap_or_default();

    html! {
        div class="
            flex flex-row items-center justify-stretch
            mb-2 gap-5 h-10
            w-full
            " {
            input class="grow text h-full" type="search" placeholder="Search for event" id="search" name="search" autocomplete="off"
                autofocus="autofocus" hx-post="/events/search" hx-trigger="keyup changed delay:20ms, search"
                hx-target="#search-results";

        }
        div class = "grow-0 h-full m-2"
            hx-target="this"  hx-swap="outerHTML" {
            button class="btn btn-primary" hx-put="/events/add" { "Add event (+)" }
        }
        table class="w-full text-inherit table-auto object-center table-fixed" {
            // We add extra table headers to account for the buttons
            thead { tr { th class="w-1/3" { "Name" } th class="w-1/3" { "Comment" } th {} th {}} }
            tbody id="search-results" {
                @for recipe in events.iter() {
                    (format_event(recipe))
                }
            }
        }
        //overwrite the default htmx indicator behaviour to swap the text of the button
        style { ("
                .inverse-htmx-indicator { display: inline; } 
                .htmx-request .inverse-htmx-indicator {display: none;} 
                .htmx-request.inverse-htmx-indicator {display: none;} 

                .my-htmx-indicator { display: none; } 
                .my-htmx-request .htmx-indicator {display: inline;} 
                .my-htmx-request.htmx-indicator {display: inline;} 
            ") }
    }
}

pub async fn search(State(state): State<MyAppState>, query: Form<SearchParameters>) -> Markup {
    let query = query.search.to_lowercase();
    let events = state.get_events().await.unwrap_or_default();

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
            @let indicator_id = format!("indicator-{}", event.event_id);
            td { (event.event_name) }
            td class="text-center" { (event.comment.clone().unwrap_or_default()) }
            td { button class="btn btn-primary" hx-target="#content" hx-get=(format!("/events/edit/{}", event.event_id)) {"Edit"} }
            td { button class="btn btn-primary" hx-target="#content" hx-indicator=("#".to_owned() + &indicator_id) hx-post=(format!("/events/duplicate/{}", event.event_id)) hx-swap="innerHTML show:window:top" {
            span id=(indicator_id) class="inverse-htmx-indicator" { "Duplicate" }
            span id=(indicator_id) class="my-htmx-indicator" { "Duplicating..." }
            } }
            td { button class="btn btn-cancel" hx-target="#content" hx-get=(format!("/events/delete/{}", event.event_id) ) {"Delete"} }
        }
    }
}
