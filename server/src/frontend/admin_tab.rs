use crate::frontend::LOGIN_URL;
use crate::MyAppState;
use axum::{
    extract::{Form, Path},
    extract::State,
    routing::{delete, get, post},
};
use axum_login::login_required;
use foodlib_new::types::ids::UserId;
use foodlib_new::{auth::AuthBackend, auth_context::AuthCtx, ops, types::ids::GroupId, user::{Group, User}};
use foodlib_new::error::Error;
use app::{AppCtx, Channel};
use crate::frontend::WebCtx;
use maud::{html, Markup};
use serde::Deserialize;

use super::MResponse;

pub(crate) mod users;

pub(crate) fn admin_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/groups", get(groups_view))
        .route("/groups", post(create_group))
        .route("/groups/{group_id}", delete(delete_group))
        .route("/groups/{group_id}/members", get(group_members_view))
        .route("/groups/{group_id}/members", post(add_member))
        .route(
            "/groups/{group_id}/members/{user_id}",
            delete(remove_member),
        )
        .route("/metro-sync", get(metro_sync_view))
        .route("/metro-sync", post(metro_sync_run))
        .nest("/users", users::users_router())
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
}

async fn groups_view(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
) -> MResponse {
    let groups = state.users().get_all_groups().await?;
    let visible: Vec<Group> = groups
        .into_iter()
        .filter(|g| !g.is_personal && (ctx.user.is_admin || ctx.group_ids.contains(&g.id)))
        .collect();

    Ok(html! {
        div class="flex flex-col items-center w-full" {
            div class="flex justify-center w-full mb-4" {
                p class="text-3xl" { "Group Management" }
            }
            div id="admin-groups" class="w-full max-w-3xl" {
                (group_list(&visible))
            }
        }
    })
}

fn group_list(groups: &[Group]) -> Markup {
    html! {
        table class="w-full text-inherit table-auto object-center responsive-card mb-4" {
            thead { tr { th class="w-1/2" { "Name" } th { "Members" } th { "Delete" } } }
            tbody {
                (create_group_row())
                @if groups.is_empty() {
                    tr { td colspan="3" class="text-center opacity-70" { "No shared groups yet" } }
                }
                @for group in groups {
                    tr id=(format!("group-{}", group.id)) {
                        td data-label="Name" { (group.name) }
                        td class="no-label" { button class="btn btn-primary"
                            hx-get=(format!("/admin/groups/{}/members", group.id))
                            hx-target="#group-detail" { "Manage" } }
                        td class="no-label" { button class="btn btn-cancel"
                            hx-delete=(format!("/admin/groups/{}", group.id))
                            hx-target=(format!("#group-{}", group.id))
                            hx-swap="outerHTML"
                            hx-confirm="Delete this group? Entities owned by it will be orphaned." { "Delete" } }
                    }
                }
            }
        }
        div id="group-detail" class="w-full" {}
    }
}

/// First-row inline add for the groups table. The handler returns a fresh
/// `#admin-groups` block (which re-renders the table including a new empty
/// add-row), so no after-request focus handler is needed — the input is
/// already empty and present in the new DOM.
fn create_group_row() -> Markup {
    html! {
        tr id="group--1" {
            td { input class="text w-full" type="text" name="name" placeholder="New group name" required="required"; }
            td colspan="2" {
                button class="btn btn-primary"
                    hx-post="/admin/groups"
                    hx-include="closest tr"
                    hx-target="#admin-groups"
                    hx-swap="outerHTML"
                    hx-on::after-request="if(event.detail.successful){const i=document.querySelector('#group--1 input[name=name]');if(i)i.focus();}"
                    { "Create group" }
            }
        }
    }
}

#[derive(Deserialize)]
struct CreateGroupForm {
    name: String,
}

async fn create_group(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<CreateGroupForm>,
) -> MResponse {
    let group = app::admin::create_group(&state, &actor, form.name).await?;
    let mut group_ids = actor.ctx.group_ids.clone();
    group_ids.push(group.id);
    let groups = state.users().get_all_groups().await?;
    let visible: Vec<Group> = groups
        .into_iter()
        .filter(|g| !g.is_personal && (actor.ctx.user.is_admin || group_ids.contains(&g.id)))
        .collect();
    Ok(html! {
        div id="admin-groups" class="w-full max-w-3xl" {
            (group_list(&visible))
        }
    })
}

async fn delete_group(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(group_id): Path<i32>,
) -> MResponse {
    app::admin::delete_group(&state, &actor, GroupId(group_id)).await?;
    Ok(html! {})
}

async fn group_members_view(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(group_id): Path<i32>,
) -> MResponse {
    let actor = AppCtx::new(ctx.clone(), Channel::WebUi);
    let group = app::admin::assert_can_manage_group(&state, &actor, GroupId(group_id)).await?;
    render_group_detail(&state, group).await
}

async fn render_group_detail(state: &MyAppState, group: Group) -> MResponse {
    let members = ops::users::members_of_group(&state.db, group.id).await?;
    let all_users = state.users().get_all_users().await?;
    let member_ids: std::collections::HashSet<UserId> = members.iter().map(|u| u.id).collect();
    let candidates: Vec<&User> = all_users
        .iter()
        .filter(|u| !member_ids.contains(&u.id))
        .collect();

    Ok(html! {
        div id="group-detail" class="w-full mt-6 border-t pt-4" {
            p class="text-2xl mb-2" { "Members of \"" (group.name) "\"" }
            table class="w-full text-inherit table-auto responsive-card mb-4" {
                thead { tr { th { "Username" } th { "Email" } th { "Remove" } } }
                tbody id=(format!("group-{}-members", group.id)) {
                    @if members.is_empty() {
                        tr { td colspan="3" class="text-center opacity-70" { "No members" } }
                    }
                    @for u in &members {
                        tr id=(format!("group-{}-member-{}", group.id, u.id)) {
                            td data-label="Username" { (u.username) }
                            td data-label="Email" { (u.email) }
                            td class="no-label" { button class="btn btn-cancel"
                                hx-delete=(format!("/admin/groups/{}/members/{}", group.id, u.id))
                                hx-target=(format!("#group-{}-member-{}", group.id, u.id))
                                hx-swap="outerHTML" { "Remove" } }
                        }
                    }
                }
            }
            form class="flex flex-row gap-2"
                hx-post=(format!("/admin/groups/{}/members", group.id))
                hx-target="#group-detail"
                hx-swap="outerHTML" {
                select class="fc-select grow" name="user_id" required="required" {
                    option value="" { "Select user to add..." }
                    @for u in &candidates {
                        option value=(u.id) { (u.username) " (" (u.email) ")" }
                    }
                }
                button class="btn btn-primary" type="submit" { "Add member" }
            }
        }
    })
}

#[derive(Deserialize)]
struct AddMemberForm {
    user_id: i64,
}

async fn add_member(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(group_id): Path<i32>,
    Form(form): Form<AddMemberForm>,
) -> MResponse {
    let group =
        app::admin::add_member(&state, &actor, GroupId(group_id), UserId(form.user_id)).await?;
    render_group_detail(&state, group).await
}

async fn remove_member(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path((group_id, user_id)): Path<(i32, i64)>,
) -> MResponse {
    app::admin::remove_member(&state, &actor, GroupId(group_id), UserId(user_id)).await?;
    Ok(html! {})
}

async fn metro_sync_view(ctx: AuthCtx) -> MResponse {
    if !ctx.user.is_admin {
        return Err(Error::Forbidden(
            "Only admins can trigger Metro sync".into(),
        ));
    }
    Ok(html! {
        div class="flex flex-col items-center w-full" {
            div class="flex justify-center w-full mb-4" {
                p class="text-3xl" { "Metro Sync" }
            }
            div class="w-full max-w-3xl flex flex-col gap-4" {
                p class="opacity-80" {
                    "Fetches all Metro-linked ingredient sources, updates prices, "
                    "categories, and classified allergen properties."
                }
                form id="metro-sync-result" class="flex flex-col gap-2" {
                    label class="flex items-center gap-2 text-sm opacity-80" {
                        input type="checkbox" name="wipe_existing" value="1";
                        "Wipe existing properties for Metro-sourced ingredients before applying new ones"
                        " (useful while iterating on the classifier — manually-curated tags on "
                        "non-Metro ingredients are preserved)"
                    }
                    div class="flex flex-row items-center gap-3" {
                        button class="btn btn-primary"
                            hx-post="/admin/metro-sync"
                            hx-include="closest form"
                            hx-target="#metro-sync-result"
                            hx-swap="innerHTML"
                            hx-indicator="#metro-sync-spinner"
                            hx-disabled-elt="this" {
                            "Run Metro sync"
                        }
                        span id="metro-sync-spinner" class="htmx-indicator flex items-center gap-2 opacity-70" {
                            span class="spinner" {}
                            "Syncing… this may take a minute"
                        }
                    }
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct MetroSyncForm {
    #[serde(default)]
    wipe_existing: Option<String>,
}

async fn metro_sync_run(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<MetroSyncForm>,
) -> MResponse {
    let report = app::admin::run_metro_sync(&state, &actor, form.wipe_existing.is_some()).await?;
    Ok(html! {
        div class="flex flex-col gap-2 p-4 border rounded" {
            p class="text-xl font-semibold" { "Sync complete" }
            ul class="list-disc pl-6" {
                li { "URLs scanned: " (report.urls_total) }
                li { "Articles fetched: " (report.articles_fetched) }
                li { "Prices updated: " (report.prices_updated) }
                @if report.allergens_wiped > 0 {
                    li { "Properties wiped before resync: " (report.allergens_wiped) }
                }
                li { "Allergen properties applied: " (report.allergens_applied) }
            }
            @if !report.warnings.is_empty() {
                details class="mt-2" {
                    summary class="cursor-pointer text-amber-600 dark:text-amber-400" {
                        (report.warnings.len()) " warning(s) (click to expand)"
                    }
                    ul class="list-disc pl-6 mt-2 text-sm opacity-80" {
                        @for w in &report.warnings {
                            li { (w) }
                        }
                    }
                }
            }
            @if !report.failures.is_empty() {
                details class="mt-2" {
                    summary class="cursor-pointer text-red-600 dark:text-red-400" {
                        (report.failures.len()) " failure(s) (click to expand)"
                    }
                    ul class="list-disc pl-6 mt-2 text-sm opacity-80" {
                        @for f in &report.failures {
                            li { (f) }
                        }
                    }
                }
            }
            button class="btn btn-primary mt-2"
                hx-get="/admin/metro-sync"
                hx-target="#content" {
                "Run again"
            }
        }
    })
}
