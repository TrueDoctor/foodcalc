use crate::FoodLib;
use axum::extract::{Form, Path};
use bigdecimal::FromPrimitive;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::user::User;
use maud::{html, Markup};
use serde::Deserialize;

mod event_detail_tab;

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
    budget: Option<f64>,
    comment: Option<String>,
}

pub async fn add(
    foodlib: FoodLib,
    user: User,
    Form(form): Form<AddEventForm>,
) -> MResponse {
    let group = foodlib.users().get_personal_group(user.id).await?;
    let event = foodlib_new::event::Event {
        id: -1,
        name: form.name,
        comment: form.comment.filter(|s| !s.is_empty()),
        budget: form.budget.and_then(bigdecimal::BigDecimal::from_f64),
        group_id: group.id,
    };
    foodlib.events().create(event).await?;
    Ok(event_list(foodlib, user).await)
}

/// Inline add-row for events. Visible columns mirror the table (name +
/// comment); budget sits in the column normally used for Edit, with the Add
/// button in the next cell. Submitting refreshes the whole `#content`, so a
/// fresh empty add-row reappears for rapid entry.
fn event_add_row() -> Markup {
    html! {
        tr id="event--1" {
            td data-label="Name" { input class="text w-full" type="text" name="name" placeholder="Event name" required="required"; }
            td data-label="Comment" { input class="text w-full" type="text" name="comment" placeholder="Comment"; }
            td data-label="Budget" { input class="text w-full" type="number" step="0.01" min="0" name="budget" placeholder="Budget"; }
            td class="no-label" colspan="2" {
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

pub async fn duplicate(foodlib: FoodLib, ctx: AuthCtx, Path(event_id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let group = foodlib.users().get_personal_group(ctx.user.id).await?;
    let id = foodlib.events().duplicate(event_id, group.id).await?;
    event_detail_tab::event_form(foodlib, ctx, Path(id)).await
}

pub async fn delete_dialog(foodlib: FoodLib, ctx: AuthCtx, Path(event_id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let event = foodlib.events().get(event_id).await?;
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

pub async fn delete(foodlib: FoodLib, ctx: AuthCtx, Path(event_id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    foodlib.events().delete(event_id).await?;
    Ok(event_list(foodlib, ctx.user).await)
}

pub async fn event_list(foodlib: FoodLib, user: User) -> Markup {
    let events = foodlib.events().list().await.unwrap_or_default();
    let user_group_ids: Vec<i32> = foodlib
        .users()
        .get_user_groups(user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|g| g.id)
        .collect();
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
            thead { tr { th { "Name" } th { "Comment" } th {} th {} th {}} }
            tbody id="search-results" {
                (event_add_row())
                @for event in visible_events.iter() {
                    (format_event(event))
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

pub async fn search(foodlib: FoodLib, user: User, query: Form<SearchParameters>) -> Markup {
    let query_str = query.search.to_lowercase();
    let mine_only = query.mine_only.is_some();
    let events = foodlib.events().list().await.unwrap_or_default();
    let user_group_ids: Vec<i32> = foodlib
        .users()
        .get_user_groups(user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|g| g.id)
        .collect();

    let filtered_events = events.iter().filter(|x| {
        x.name.to_lowercase().contains(&query_str)
            && (user.is_admin || user_group_ids.contains(&x.group_id))
            && (!mine_only || user_group_ids.contains(&x.group_id))
    });

    html! {
        (event_add_row())
        @for event in filtered_events {
            (format_event(event))
        }
    }
}

fn format_event(event: &foodlib_new::event::Event) -> Markup {
    html! {
        tr id=(format!("event-{}", event.id)) {
            @let indicator_id = format!("indicator-{}", event.id);
            td data-label="Name" { (event.name) }
            td data-label="Comment" { (event.comment.clone().unwrap_or_default()) }
            td class="no-label" { button class="btn btn-primary" hx-target="#content" hx-push-url="true" hx-get=(format!("/events/edit/{}", event.id)) {"Edit"} }
            td class="no-label" { button class="btn btn-primary" hx-target="#content" hx-indicator=("#".to_owned() + &indicator_id) hx-post=(format!("/events/duplicate/{}", event.id)) hx-swap="innerHTML show:window:top" {
            span id=(indicator_id) class="inverse-htmx-indicator" { "Duplicate" }
            span id=(indicator_id) class="my-htmx-indicator" { "Duplicating..." }
            } }
            td class="no-label" { button class="btn btn-cancel" hx-target="#content" hx-get=(format!("/events/delete/{}", event.id) ) {"Delete"} }
        }
    }
}
