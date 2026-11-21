//! Permission management for the legacy UI.
//!
//! Deprecated view — functional, not beautiful. Two parts:
//! 1. a static role catalog (this file): what the built-in roles grant, so
//!    users can *understand* the new RBAC schema without the REST API;
//! 2. the group member role editor (routes on `admin_tab`, rendering here):
//!    checkboxes per member → `app::users::set_member_roles`, which brings
//!    the `group:manage_members` guard, role validation, last-manager
//!    protection and audit logging for free.

use axum::response::Response;
use foodlib_new::policy::{Permission, RoleDefinition};
use maud::{html, Markup};

use crate::app::users::role_catalog;

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
        details class="border rounded-lg p-3" open {
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

/// The static role catalog page. Rendered purely from the app layer's
/// `role_catalog()` — no hardcoding, so the view stays truthful as the
/// permission schema evolves.
pub fn render_role_catalog(roles: &[RoleDefinition]) -> Markup {
    html! {
        div class="flex flex-col items-center w-full" {
            div class="flex justify-center w-full mb-4" {
                p class="text-3xl" { "Rollen & Berechtigungen" }
            }
            p class="opacity-80 mb-4 max-w-3xl text-center" {
                "Jedes Gruppenmitglied hat eine oder mehrere Rollen. Die Rolle "
                "bestimmt, welche Berechtigungen es in der Gruppe hat. "
                "Globale Berechtigungen (Benutzerverwaltung, Einstellungen) "
                "werden separat pro Benutzer vergeben."
            }
            div class="w-full max-w-3xl flex flex-col gap-4" {
                @for role in roles {
                    (role_card(role))
                }
            }
        }
    }
}

/// `GET /admin/permissions` — the role catalog page.
pub async fn permissions_view(
    _ctx: crate::frontend::ctx::WebCtx,
) -> Result<Markup, Response> {
    Ok(render_role_catalog(&role_catalog()))
}
