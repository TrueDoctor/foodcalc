use axum::{
    extract::{Form, Path},
    routing::{delete, get, post},
};
use axum_login::login_required;
use foodlib_new::{
    auth::AuthBackend,
    auth_context::AuthCtx,
    error::{Error, Result},
    user::{Group, User},
};
use maud::{html, Markup};
use serde::Deserialize;

use crate::frontend::LOGIN_URL;
use crate::{FoodLib, MyAppState};

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

/// Admins may manage any group. Members may manage groups they belong to (Option C).
/// Personal groups are never editable here — they live and die with their user.
async fn assert_can_manage_group(
    foodlib: &foodlib_new::FoodLib,
    ctx: &AuthCtx,
    group_id: i32,
) -> Result<Group> {
    let group = foodlib.users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Personal groups can only be managed via their owning user".into(),
        ));
    }
    if !ctx.can_access_group(group_id) {
        return Err(Error::Forbidden(
            "You don't have permission to manage this group".into(),
        ));
    }
    Ok(group)
}

async fn groups_view(foodlib: FoodLib, ctx: AuthCtx) -> MResponse {
    let groups = foodlib.users().get_all_groups().await?;
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
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(form): Form<CreateGroupForm>,
) -> MResponse {
    let trimmed = form.name.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation {
            message: "Group name cannot be empty".into(),
        });
    }
    let group = foodlib.users().create_group(trimmed).await?;
    foodlib
        .users()
        .add_user_to_group(ctx.user.id, group.id)
        .await?;
    let mut group_ids = ctx.group_ids.clone();
    group_ids.push(group.id);
    let groups = foodlib.users().get_all_groups().await?;
    let visible: Vec<Group> = groups
        .into_iter()
        .filter(|g| !g.is_personal && (ctx.user.is_admin || group_ids.contains(&g.id)))
        .collect();
    Ok(html! {
        div id="admin-groups" class="w-full max-w-3xl" {
            (group_list(&visible))
        }
    })
}

async fn delete_group(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(group_id): Path<i32>,
) -> MResponse {
    assert_can_manage_group(&foodlib, &ctx, group_id).await?;
    foodlib.users().delete_group(group_id).await?;
    Ok(html! {})
}

async fn group_members_view(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(group_id): Path<i32>,
) -> MResponse {
    let group = assert_can_manage_group(&foodlib, &ctx, group_id).await?;
    render_group_detail(&foodlib, group).await
}

async fn render_group_detail(foodlib: &foodlib_new::FoodLib, group: Group) -> MResponse {
    let members = members_of_group(foodlib, group.id).await?;
    let all_users = foodlib.users().get_all_users().await?;
    let member_ids: std::collections::HashSet<i64> = members.iter().map(|u| u.id).collect();
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

async fn members_of_group(foodlib: &foodlib_new::FoodLib, group_id: i32) -> Result<Vec<User>> {
    let pool = foodlib.pool_arc();
    let rows = sqlx::query_as!(
        User,
        r#"
        SELECT u.id, u.username, u.email, u.password_hash, u.is_admin, u.created_at
        FROM user_groups ug
        JOIN users u ON u.id = ug.user_id
        WHERE ug.group_id = $1
        ORDER BY u.username
        "#,
        group_id
    )
    .fetch_all(&*pool)
    .await?;
    Ok(rows)
}

#[derive(Deserialize)]
struct AddMemberForm {
    user_id: i64,
}

async fn add_member(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(group_id): Path<i32>,
    Form(form): Form<AddMemberForm>,
) -> MResponse {
    let group = assert_can_manage_group(&foodlib, &ctx, group_id).await?;
    foodlib
        .users()
        .add_user_to_group(form.user_id, group_id)
        .await?;
    render_group_detail(&foodlib, group).await
}

async fn remove_member(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((group_id, user_id)): Path<(i32, i64)>,
) -> MResponse {
    assert_can_manage_group(&foodlib, &ctx, group_id).await?;
    foodlib
        .users()
        .remove_user_from_group(user_id, group_id)
        .await?;
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
                        span id="metro-sync-spinner" class="htmx-indicator opacity-70" {
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
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(form): Form<MetroSyncForm>,
) -> MResponse {
    if !ctx.user.is_admin {
        return Err(Error::Forbidden(
            "Only admins can trigger Metro sync".into(),
        ));
    }
    let opts = foodlib_new::ops::metro_sync::SyncOptions {
        wipe_existing: form.wipe_existing.is_some(),
    };
    let report = foodlib.metro_sync().sync_with(None, opts).await?;
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
            @if !report.failures.is_empty() {
                details class="mt-2" {
                    summary class="cursor-pointer text-yellow-600 dark:text-yellow-400" {
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
