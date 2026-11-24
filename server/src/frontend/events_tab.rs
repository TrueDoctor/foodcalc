use axum::extract::{Form, Path};
use axum::extract::State;
use foodlib_new::entities::types::ids::EventId;
use foodlib_new::user::User;
use maud::{html, Markup};
use serde::Deserialize;

mod event_detail_tab;

use crate::frontend::icons;
use crate::frontend::WebCtx;
use crate::MyAppState;

use super::MResponse;

pub(crate) fn events_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", axum::routing::get(event_list))
        .route("/delete/{event_id}", axum::routing::get(delete_dialog))
        .route("/delete/{event_id}", axum::routing::delete(delete))
        .route("/add", axum::routing::post(add))
        .route("/duplicate/{event_id}", axum::routing::post(duplicate))
        .nest("/edit", event_detail_tab::event_detail_router())
        .route("/search", axum::routing::post(search))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
    #[serde(default)]
    mine_only: Option<String>,
}

#[derive(Deserialize)]
pub struct AddEventForm {
    name: String,
    #[serde(default, deserialize_with = "empty_to_none")]
    budget: Option<f64>,
    comment: Option<String>,
}

/// An empty form field (`""`) means "unset", not "unparseable number".
fn empty_to_none<'de, D>(deserializer: D) -> std::result::Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
    }
}

pub async fn add(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<AddEventForm>,
) -> MResponse {
    app::events::create_event(&state, &actor, form.name, form.comment, form.budget).await?;
    Ok(event_list(State(state), actor.ctx.user).await)
}

/// Inline add-row for events. Visible columns mirror the table (name +
/// comment); budget sits in the column normally used for Edit, with the Add
/// button in the next cell. Submitting refreshes the whole `#content`, so a
/// fresh empty add-row reappears for rapid entry.
fn event_add_row(is_admin: bool) -> Markup {
    html! {
        tr id="event--1" {
            @if is_admin {
                td class="text-center opacity-70 no-label" { "+" }
            }
            td data-label="Name" { input class="text w-full" type="text" name="name" placeholder="Event name" required="required"; }
            td data-label="Comment" { input class="text w-full" type="text" name="comment" placeholder="Comment"; }
            td class="no-label" {
                div class="flex flex-row items-center gap-2" {
                    input class="text w-full" type="number" step="0.01" min="0" name="budget" placeholder="Budget";
                    button class="btn btn-primary"
                        hx-post="/events/add"
                        hx-include="closest tr"
                        hx-target="#content"
                        hx-on::after-request="if(event.detail.successful){const i=document.querySelector('#event--1 input[name=name]');if(i)i.focus();}"
                        { "Add" }
                }
            }
        }
    }
}

pub async fn duplicate(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
) -> MResponse {
    let id = app::events::duplicate_event(&state, &actor, event_id).await?;
    event_detail_tab::event_form(State(state), WebCtx(actor), Path(id)).await
}

pub async fn delete_dialog(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
) -> MResponse {
    let event = app::events::get_event(&state, &actor, event_id).await?;
    Ok(html! {
        div class="flex flex-col gap-2" {
            p class="text-2xl" { "Are you sure you want to delete this Event permanently?" }
            div class="flex flex-col gap-2" {
                p { "Event ID: " (event.id) }
                p { "Event Name: " (event.name) }
                p { "Comment: " (event.comment.clone().unwrap_or_default()) }
            }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get="/events" hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/delete/{}", event_id)) { "Confirm Delete" }
            }
        }
    })
}

pub async fn delete(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
) -> MResponse {
    app::events::delete_event(&state, &actor, event_id).await?;
    Ok(event_list(State(state), actor.ctx.user).await)
}

pub async fn event_list(State(state): State<MyAppState>, user: User) -> Markup {
    let events = app::events::list_events(&state).await.unwrap_or_default();
    let user_group_ids = app::users::lenient_user_group_ids(&state, &user).await;
    let visible_events: Vec<_> = if user.is_admin {
        events.iter().collect()
    } else {
        events
            .iter()
            .filter(|e| user_group_ids.contains(&e.group_id))
            .collect()
    };

    html! {
        form id="events-filter" class="
            flex flex-row items-center justify-stretch
            mb-2 gap-5 h-10
            w-full
            "
            hx-post="/events/search"
            hx-trigger="keyup changed delay:20ms from:#search, change from:#mine-only, search"
            hx-target="#search-results" {
            input class="grow text h-full" type="search" placeholder="Search for event" id="search" name="search" autocomplete="off" autofocus="autofocus";
            @if user.is_admin {
                label class="flex items-center gap-2 whitespace-nowrap" {
                    input type="checkbox" id="mine-only" name="mine_only" value="1";
                    "Mine only"
                }
            }
        }
        table class="w-full text-inherit table-auto object-center responsive-card" {
            thead { tr { @if user.is_admin { th class="w-1/12" { "ID" } } th { "Name" } th { "Comment" } th {}} }
            tbody id="search-results" {
                (event_add_row(user.is_admin))
                @for event in visible_events.iter() {
                    (format_event(event, user.is_admin))
                }
            }
        }
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

pub async fn search(
    State(state): State<MyAppState>,
    user: User,
    query: Form<SearchParameters>,
) -> Markup {
    let query_str = query.search.to_lowercase();
    let mine_only = query.mine_only.is_some();
    let events = app::events::list_events(&state).await.unwrap_or_default();
    let user_group_ids = app::users::lenient_user_group_ids(&state, &user).await;

    let filtered_events = events.iter().filter(|x| {
        x.name.to_lowercase().contains(&query_str)
            && (user.is_admin || user_group_ids.contains(&x.group_id))
            && (!mine_only || user_group_ids.contains(&x.group_id))
    });

    html! {
        (event_add_row(user.is_admin))
        @for event in filtered_events {
            (format_event(event, user.is_admin))
        }
    }
}

fn format_event(event: &foodlib_new::event::Event, is_admin: bool) -> Markup {
    html! {
        tr id=(format!("event-{}", event.id)) {
            @let indicator_id = format!("indicator-{}", event.id);
            @if is_admin {
                td class="text-center opacity-70" data-label="ID" { (event.id) }
            }
            td data-label="Name" { (event.name) }
            td data-label="Comment" { (event.comment.clone().unwrap_or_default()) }
            td class="no-label whitespace-nowrap" {
                div class="flex flex-row flex-nowrap gap-2 justify-end items-center" {
                    button class="icon-btn" title="Timeline" aria-label="Timeline" hx-target="#content" hx-push-url="true" hx-get=(format!("/events/edit/timeline/{}", event.id)) { (icons::timeline()) }
                    button class="icon-btn" title="Edit" aria-label="Edit" hx-target="#content" hx-push-url="true" hx-get=(format!("/events/edit/{}", event.id)) { (icons::edit()) }
                    button class="icon-btn" title="Duplicate" aria-label="Duplicate" hx-target="#content" hx-indicator=("#".to_owned() + &indicator_id) hx-post=(format!("/events/duplicate/{}", event.id)) hx-swap="innerHTML show:window:top" {
                        span id=(indicator_id) class="inverse-htmx-indicator" { (icons::duplicate()) }
                        span id=(indicator_id) class="my-htmx-indicator" { "…" }
                    }
                    button class="icon-btn" title="Delete" aria-label="Delete" hx-target="#content" hx-get=(format!("/events/delete/{}", event.id) ) { (icons::delete()) }
                }
            }
        }
    }
}
