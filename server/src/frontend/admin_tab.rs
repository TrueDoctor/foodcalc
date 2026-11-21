use axum::{
    extract::{Form, Path, State},
    routing::{delete, get, post, put},
};
use axum_login::login_required;
use foodlib_new::{
    auth::AuthBackend,
    auth_context::AuthCtx,
    entities::types::ids::{GroupId, UserId},
    error::{Error, Result},
    policy::{GroupPerm, Permission},
    user::{Group, User},
};
use maud::{html, Markup};
use serde::Deserialize;

use crate::app;
use crate::frontend::ctx::WebCtx;
use crate::frontend::LOGIN_URL;
use crate::{FoodLib, MyAppState};

use super::MResponse;

pub(crate) mod permissions;
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
        .route(
            "/groups/{group_id}/members/{user_id}/roles",
            get(role_edit_form).post(set_member_roles),
        )
        .route(
            "/groups/{group_id}/default-roles",
            put(put_default_roles).delete(clear_default_roles),
        )
        .route("/metro-sync", get(metro_sync_view))
        .route("/metro-sync", post(metro_sync_run))
        .route("/permissions", get(permissions::permissions_view))
        .nest("/users", users::users_router())
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
}

/// Admins may manage any group. Members may manage groups they belong to (Option C).
/// Personal groups are never editable here — they live and die with their user.
async fn assert_can_manage_group(
    foodlib: &foodlib_new::FoodLib,
    ctx: &AuthCtx,
    group_id: GroupId,
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
        .filter(|g| !g.is_personal && (ctx.user.is_admin || super::user_group_ids(&ctx).contains(&g.id)))
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
        .add_user_to_group_with_roles(ctx.user.id, group.id, &["manager".into()])
        .await?;
    let mut group_ids = super::user_group_ids(&ctx);
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

async fn delete_group(foodlib: FoodLib, ctx: AuthCtx, Path(group_id): Path<GroupId>) -> MResponse {
    assert_can_manage_group(&foodlib, &ctx, group_id).await?;
    foodlib.users().delete_group(group_id).await?;
    Ok(html! {})
}

async fn group_members_view(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path(group_id): Path<GroupId>,
) -> MResponse {
    let group = state.users().get_group(group_id).await?;
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    render_group_detail(&state, &ctx, group, members).await
}

/// Role checkboxes shared by the member editor and (later) the group
/// default-roles section. Slugs come from the app layer's role catalog —
/// never hardcoded.
fn role_checkboxes(name: &str, selected: &[String]) -> Markup {
    html! {
        div class="flex flex-wrap gap-3" {
            @for def in app::users::role_catalog() {
                label class="flex items-center gap-1 text-sm" {
                    input type="checkbox" name=(name) value=(def.slug)
                        checked[selected.contains(&def.slug.to_string())];
                    (def.name)
                }
            }
        }
    }
}

/// Chip per assigned role, shown in the members table.
fn role_chips(roles: &[String]) -> Markup {
    html! {
        div class="flex flex-wrap gap-1" {
            @for role in roles {
                span class="px-2 py-0.5 rounded-full text-xs bg-blue-100 text-blue-800 border border-blue-300" {
                    (role)
                }
            }
            @if roles.is_empty() {
                span class="text-xs opacity-60" { "keine" }
            }
        }
    }
}

/// The edit affordance row for one member's roles: a form with one checkbox
/// per catalog role. Saving posts all checked slugs to the
/// `set_member_roles` use-case (guard, validation, last-manager protection
/// and audit live there).
fn member_roles_row(group_id: GroupId, member: &app::users::MemberWithRoles) -> Markup {
    html! {
        tr class="bg-gray-50 dark:bg-gray-800" {
            td colspan="4" {
                form class="flex flex-col gap-2"
                    hx-post=(format!("/admin/groups/{}/members/{}/roles", group_id, member.member.id))
                    hx-target="#group-detail"
                    hx-swap="outerHTML" {
                    (role_checkboxes("roles", &member.roles))
                    div class="flex gap-2" {
                        button class="btn btn-primary" type="submit" { "Rollen speichern" }
                        button class="btn btn-cancel" type="button"
                            hx-get=(format!("/admin/groups/{}/members", group_id))
                            hx-target="#group-detail"
                            hx-swap="outerHTML" { "Abbrechen" }
                    }
                }
            }
        }
    }
}

async fn render_group_detail(
    state: &MyAppState,
    ctx: &WebCtx,
    group: Group,
    members: Vec<app::users::MemberWithRoles>,
) -> MResponse {
    // Default-roles section is only interactive for holders of
    // `group:manage_settings`; everyone else just sees the resolved defaults.
    let can_manage_settings = ctx
        .ctx
        .has_group_permission(group.id, Permission::Group(GroupPerm::ManageSettings));
    let defaults = app::users::get_default_roles(state, ctx, group.id).await.ok();
    let all_users = state.db.users().get_all_users().await?;
    let member_ids: std::collections::HashSet<UserId> =
        members.iter().map(|m| m.member.id).collect();
    let candidates: Vec<&User> = all_users
        .iter()
        .filter(|u| !member_ids.contains(&u.id))
        .collect();

    Ok(html! {
        div id="group-detail" class="w-full mt-6 border-t pt-4" {
            p class="text-2xl mb-2" { "Members of \"" (group.name) "\"" }
            table class="w-full text-inherit table-auto responsive-card mb-4" {
                thead { tr { th { "Username" } th { "Email" } th { "Rollen" } th { "Aktionen" } } }
                tbody id=(format!("group-{}-members", group.id)) {
                    @if members.is_empty() {
                        tr { td colspan="4" class="text-center opacity-70" { "No members" } }
                    }
                    @for m in &members {
                        tr id=(format!("group-{}-member-{}", group.id, m.member.id)) {
                            td data-label="Username" { (m.member.username) }
                            td data-label="Email" { (m.member.email) }
                            td data-label="Rollen" { (role_chips(&m.roles)) }
                            td class="no-label" {
                                div class="flex gap-1 justify-end" {
                                    button class="btn btn-primary"
                                        hx-get=(format!("/admin/groups/{}/members/{}/roles", group.id, m.member.id))
                                        hx-target=(format!("#group-{}-member-{}", group.id, m.member.id))
                                        hx-swap="afterend" { "Rollen" }
                                    button class="btn btn-cancel"
                                        hx-delete=(format!("/admin/groups/{}/members/{}", group.id, m.member.id))
                                        hx-target="#group-detail"
                                        hx-swap="outerHTML"
                                        hx-confirm="Mitglied wirklich entfernen?" { "Remove" }
                                }
                            }
                        }
                    }
                }
            }
            form class="flex flex-row gap-2 items-start"
                hx-post=(format!("/admin/groups/{}/members", group.id))
                hx-target="#group-detail"
                hx-swap="outerHTML" {
                select class="fc-select grow" name="user" required="required" {
                    option value="" { "Select user to add..." }
                    @for u in &candidates {
                        option value=(u.id.get()) { (u.username) " (" (u.email) ")" }
                    }
                }
                div class="flex flex-col gap-1" {
                    p class="text-sm font-semibold" { "Initiale Rollen (optional)" }
                    (role_checkboxes("roles", &[]))
                }
                button class="btn btn-primary" type="submit" { "Add member" }
            }
            // Default roles for new members of this group. Only holders of
            // `group:manage_settings` get the editor; others see what a new
            // member would receive.
            @if let Some(d) = &defaults {
                details class="border rounded-lg p-3 mt-4" {
                    summary class="cursor-pointer font-semibold" {
                        "Standard-Rollen für neue Mitglieder"
                        @if d.inherited {
                            span class="text-xs opacity-60 ml-2" { "(erbt global)" }
                        }
                    }
                    p class="text-sm opacity-80 mt-2 mb-2" {
                        "Neue Mitglieder erhalten automatisch: "
                        (role_chips(&d.resolved))
                    }
                    @if can_manage_settings {
                        form class="flex flex-col gap-2 mt-2"
                            hx-put=(format!("/admin/groups/{}/default-roles", group.id))
                            hx-target="#group-detail"
                            hx-swap="outerHTML" {
                            (role_checkboxes("roles", d.roles.as_deref().unwrap_or(&[])))
                            div class="flex gap-2" {
                                button class="btn btn-primary" type="submit" { "Speichern" }
                                @if !d.inherited {
                                    button class="btn btn-cancel" type="button"
                                        hx-delete=(format!("/admin/groups/{}/default-roles", group.id))
                                        hx-target="#group-detail"
                                        hx-swap="outerHTML"
                                        hx-confirm="Zurücksetzen auf den globalen Standard?" {
                                            "Globalen Standard erben"
                                        }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct AddMemberForm {
    /// Username, email or numeric id — resolved by the app layer.
    user: String,
    /// Optional initial roles; empty → the group's default roles.
    #[serde(default)]
    roles: Vec<String>,
}

async fn add_member(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path(group_id): Path<GroupId>,
    Form(form): Form<AddMemberForm>,
) -> MResponse {
    let input = app::users::AddMemberInput {
        user: form.user,
        roles: if form.roles.is_empty() { None } else { Some(form.roles) },
    };
    app::users::add_member(&state, &ctx, group_id, input).await?;
    let group = state.users().get_group(group_id).await?;
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    render_group_detail(&state, &ctx, group, members).await
}

async fn remove_member(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path((group_id, user_id)): Path<(GroupId, UserId)>,
) -> MResponse {
    app::users::remove_member(&state, &ctx, group_id, user_id).await?;
    let group = state.users().get_group(group_id).await?;
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    render_group_detail(&state, &ctx, group, members).await
}

/// `GET …/roles` — the checkbox editor row for one member.
async fn role_edit_form(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path((group_id, user_id)): Path<(GroupId, UserId)>,
) -> MResponse {
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    let member = members
        .iter()
        .find(|m| m.member.id == user_id)
        .ok_or(Error::NotFound {
            entity: "GroupMember",
            id: user_id.to_string(),
        })?;
    Ok(member_roles_row(group_id, member))
}

#[derive(Deserialize)]
struct SetRolesForm {
    /// All checked role slugs. At least one is enforced by the app layer.
    #[serde(default)]
    roles: Vec<String>,
}

/// `POST …/roles` — replace the member's role set via the app use-case
/// (validates slugs, protects the last manager, writes the audit log).
async fn set_member_roles(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path((group_id, user_id)): Path<(GroupId, UserId)>,
    Form(form): Form<SetRolesForm>,
) -> MResponse {
    let input = app::users::SetMemberRolesInput { roles: form.roles };
    app::users::set_member_roles(&state, &ctx, group_id, user_id, input).await?;
    let group = state.users().get_group(group_id).await?;
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    render_group_detail(&state, &ctx, group, members).await
}

/// `PUT …/default-roles` — set the group's default roles for new members
/// (guard `group:manage_settings`, validated in the app layer).
async fn put_default_roles(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path(group_id): Path<GroupId>,
    Form(form): Form<SetRolesForm>,
) -> MResponse {
    let input = app::users::GroupDefaultRolesInput { roles: form.roles };
    app::users::put_default_roles(&state, &ctx, group_id, input).await?;
    let group = state.users().get_group(group_id).await?;
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    render_group_detail(&state, &ctx, group, members).await
}

/// `DELETE …/default-roles` — reset to "inherit the global default".
async fn clear_default_roles(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path(group_id): Path<GroupId>,
) -> MResponse {
    app::users::clear_default_roles(&state, &ctx, group_id).await?;
    let group = state.users().get_group(group_id).await?;
    let members = app::users::list_members(&state, &ctx, group_id).await?;
    render_group_detail(&state, &ctx, group, members).await
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
