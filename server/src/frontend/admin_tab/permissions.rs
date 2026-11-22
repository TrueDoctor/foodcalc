//! Permission management for the legacy UI (deprecated view — functional,
//! not beautiful).
//!
//! Two parts:
//! 1. the **role×permission matrix**: permissions are the rows, roles the
//!    columns. Cells are checkboxes backed by the DB `role_permissions`
//!    table (the source of truth since migration 20261122000000). Each role
//!    column is one HTML5 form (`form=` attribute on the checkboxes), saved
//!    via `app::users::set_role_permissions` (guard `admin:manage_settings`,
//!    audited). Custom roles can be created and deleted here.
//! 2. the static role catalog below: name, description and grouped
//!    permissions per role, rendered purely from the app layer.

use axum::extract::{Form, Path, State};
use axum::response::Response;
use foodlib_new::ops::users::DbRole;
use foodlib_new::policy::{Permission, RoleDefinition};
use maud::{html, Markup};
use serde::Deserialize;

use crate::app;
use crate::frontend::form::HtmlForm;
use crate::app::users::role_catalog;
use crate::frontend::ctx::WebCtx;
use crate::MyAppState;

/// Group label for a permission's domain prefix (`events:edit` → `Events`).
fn domain_group(permission: &Permission) -> &'static str {
    match permission {
        Permission::Event(_) => "Events",
        Permission::Recipe(_) => "Rezepte",
        Permission::Ingredient(_) => "Zutaten",
        Permission::Inventory(_) => "Inventare",
        Permission::Group(_) => "Gruppe",
    }
}

/// Short human label for a permission slug (`events:edit` → `edit`).
fn short_label(permission: &Permission) -> String {
    let slug = permission.to_string();
    let domain = domain_group(permission).to_lowercase();
    slug.strip_prefix(&format!("{domain}:"))
        .unwrap_or(&slug)
        .replace('_', " ")
        .to_string()
}

/// One card per role: permissions grouped by domain, in first-appearance
/// order (stable and simple).
fn role_card(role: &RoleDefinition) -> Markup {
    let mut order: Vec<&'static str> = Vec::new();
    let mut by_domain: Vec<(&'static str, Vec<&Permission>)> = Vec::new();
    for p in &role.permissions {
        let d = domain_group(p);
        if !order.contains(&d) {
            order.push(d);
            by_domain.push((d, Vec::new()));
        }
        by_domain
            .iter_mut()
            .find(|(name, _)| *name == d)
            .map(|(_, list)| list.push(p));
    }

    html! {
        details class="border rounded-lg p-3" {
            summary class="cursor-pointer font-semibold" {
                (role.name)
                " "
                code class="text-sm opacity-60" { (role.slug) }
            }
            p class="opacity-80 text-sm mt-2 mb-3" { (role.description) }
            div class="flex flex-wrap gap-4" {
                @for (domain, perms) in &by_domain {
                    div class="min-w-[10rem]" {
                        p class="font-semibold text-sm mb-1" { (domain) }
                        ul class="list-disc ml-4 text-sm opacity-90" {
                            @for p in perms {
                                li { (short_label(p)) }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The static role catalog page section. Rendered purely from the app
/// layer's `role_catalog()` — no hardcoding.
fn render_role_catalog(roles: &[RoleDefinition]) -> Markup {
    html! {
        h2 class="text-2xl mb-2" { "Rollenkatalog" }
        p class="opacity-80 mb-4 max-w-3xl" {
            "Jedes Gruppenmitglied hat eine oder mehrere Rollen. Die Rolle "
            "bestimmt, welche Berechtigungen es in der Gruppe hat. Globale "
            "Berechtigungen (Benutzerverwaltung, Einstellungen) werden "
            "separat pro Benutzer vergeben."
        }
        div class="w-full max-w-3xl flex flex-col gap-4 mb-8" {
            @for role in roles {
                (role_card(role))
            }
        }
    }
}

/// One matrix row: a permission plus whether a new domain header must be
/// emitted before it.
struct MatrixRow {
    domain_header: Option<&'static str>,
    permission: Permission,
}

/// Rows for [`render_matrix`]: every permission, with a domain header row
/// marker when the domain changes (stable, first-appearance order).
fn matrix_rows() -> Vec<MatrixRow> {
    let mut rows = Vec::new();
    let mut last_domain: Option<&'static str> = None;
    for permission in Permission::all() {
        let domain = domain_group(&permission);
        let domain_header = if last_domain != Some(domain) {
            last_domain = Some(domain);
            Some(domain)
        } else {
            None
        };
        rows.push(MatrixRow {
            domain_header,
            permission,
        });
    }
    rows
}

/// The editable role×permission matrix. Permissions are rows (grouped by
/// domain), roles are columns. Each column is one HTML5 form (`form=`
/// attribute on the checkboxes + save button), so a plain `<table>` works.
fn render_matrix(roles: &[DbRole], can_manage: bool) -> Markup {
    let rows = matrix_rows();
    html! {
        h2 class="text-2xl mb-2" { "Berechtigungen" }
        p class="opacity-80 mb-4 max-w-3xl" {
            "Zeile = Berechtigung, Spalte = Rolle. Änderungen gelten sofort "
            "für alle Gruppen, die die Rolle verwenden."
        }
        @for role in roles {
            form id=(format!("role-form-{}", role.slug))
                hx-post=(format!("/admin/permissions/roles/{}/permissions", role.slug))
                hx-target="#permissions-matrix"
                hx-swap="outerHTML" {}
        }
        div id="permissions-matrix" class="overflow-x-auto mb-8" {
            table class="text-sm text-inherit table-auto border-collapse" {
                thead {
                    tr {
                        th class="text-left pr-4" { "Berechtigung" }
                        @for role in roles {
                            th class="px-2 align-bottom" {
                                div class="flex flex-col items-center gap-1" {
                                    span class="font-semibold whitespace-nowrap" { (role.name) }
                                    code class="text-xs opacity-60" { (role.slug) }
                                    @if can_manage {
                                        button class="btn btn-primary text-xs" type="submit"
                                            form=(format!("role-form-{}", role.slug)) { "💾" }
                                    }
                                }
                            }
                        }
                    }
                }
                tbody {
                    @for row in &rows {
                        @if let Some(domain) = row.domain_header {
                            tr {
                                td class="pt-3 pb-1 font-semibold text-xs uppercase tracking-wide opacity-70"
                                    colspan=(roles.len() + 1) { (domain) }
                            }
                        }
                        tr class="border-t border-gray-200 dark:border-gray-700" {
                            td class="pr-4 py-1" {
                                code class="text-xs" { (row.permission.to_string()) }
                            }
                            @for role in roles {
                                @let has = role.permissions.contains(&row.permission);
                                td class="text-center px-2 py-1" {
                                    input type="checkbox" name="permissions"
                                        value=(row.permission.to_string())
                                        form=(format!("role-form-{}", role.slug))
                                        checked[has]
                                        disabled[!can_manage];
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Create-role and custom-role deletion section.
fn render_role_admin(roles: &[DbRole], can_manage: bool) -> Markup {
    html! {
        h2 class="text-2xl mb-2" { "Rollen verwalten" }
        @if can_manage {
            form class="flex flex-row gap-2 items-end mb-4"
                hx-post="/admin/permissions/roles"
                hx-target="#permissions-page"
                hx-swap="outerHTML" {
                div class="flex flex-col" {
                    label class="text-sm" { "Slug (a-z, 0-9, _ -)" }
                    input class="text" type="text" name="slug" required="required"
                        pattern="[a-z0-9_-]{1,32}" placeholder="z.B. kueche";
                }
                div class="flex flex-col" {
                    label class="text-sm" { "Name" }
                    input class="text" type="text" name="name" required="required" placeholder="z.B. Küche";
                }
                button class="btn btn-primary" type="submit" { "Rolle anlegen" }
            }
            div class="flex flex-wrap gap-2 mb-4" {
                @for role in roles.iter().filter(|r| !r.is_system) {
                    button class="btn btn-cancel text-xs" type="button"
                        hx-delete=(format!("/admin/permissions/roles/{}", role.slug))
                        hx-target="#permissions-page"
                        hx-swap="outerHTML"
                        hx-confirm=(format!("Rolle '{}' wirklich löschen?", role.slug)) {
                        "Löschen: " (role.slug)
                    }
                }
            }
        } @else {
            p class="text-sm opacity-70" {
                "Zum Bearbeiten wird die globale Berechtigung admin:manage_settings benötigt."
            }
        }
    }
}

/// `GET /admin/permissions` — matrix + catalog.
pub async fn permissions_view(
    State(state): State<MyAppState>,
    ctx: WebCtx,
) -> Result<Markup, Response> {
    Ok(render_page(&state, &ctx).await)
}

/// Shared page shell: matrix + role admin + catalog. Used by the view and
/// by every mutation handler's re-render.
async fn render_page(state: &MyAppState, ctx: &WebCtx) -> Markup {
    let roles = app::users::list_roles(state)
        .await
        .unwrap_or_default();
    let can_manage = ctx
        .ctx
        .can_global(foodlib_new::policy::GlobalPermission::ManageSettings);
    html! {
        div id="permissions-page" class="flex flex-col items-center w-full" {
            div class="flex justify-center w-full mb-4" {
                p class="text-3xl" { "Rollen & Berechtigungen" }
            }
            div class="w-full max-w-5xl px-2" {
                (render_matrix(&roles, can_manage))
                (render_role_admin(&roles, can_manage))
                (render_role_catalog(&role_catalog()))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct RolePermissionsForm {
    /// All checked permission slugs for the role column.
    #[serde(default)]
    pub permissions: Vec<Permission>,
}
/// `POST /admin/permissions/roles/{slug}/permissions` — save one column.
pub async fn set_role_permissions(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path(slug): Path<String>,
    form: HtmlForm<RolePermissionsForm>,
) -> Result<Markup, Response> {
    if let Err(e) = app::users::set_role_permissions(&state, &ctx, &slug, form.0.permissions).await
    {
        return Ok(crate::frontend::html_error(&format!("{e}"), "/admin/permissions"));
    }
    Ok(render_page(&state, &ctx).await)
}

#[derive(Deserialize)]
pub struct CreateRoleForm {
    pub slug: String,
    pub name: String,
}

/// `POST /admin/permissions/roles` — create a custom role.
pub async fn create_role(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Form(form): Form<CreateRoleForm>,
) -> Result<Markup, Response> {
    if let Err(e) = app::users::create_role(&state, &ctx, &form.slug, &form.name).await {
        return Ok(crate::frontend::html_error(&format!("{e}"), "/admin/permissions"));
    }
    Ok(render_page(&state, &ctx).await)
}

/// `DELETE /admin/permissions/roles/{slug}` — delete a custom, unused role.
pub async fn delete_role(
    State(state): State<MyAppState>,
    ctx: WebCtx,
    Path(slug): Path<String>,
) -> Result<Markup, Response> {
    if let Err(e) = app::users::delete_role(&state, &ctx, &slug).await {
        return Ok(crate::frontend::html_error(&format!("{e}"), "/admin/permissions"));
    }
    Ok(render_page(&state, &ctx).await)
}
