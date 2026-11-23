use axum::{
    extract::{Form, Path},
    extract::State,
    routing::{delete, get, post},
};
use foodlib_new::{
    types::ids::{GroupId, UserId},
    user::{Group, User},
};
use maud::{html, Markup};
use serde::Deserialize;

use crate::frontend::MResponse;
use crate::MyAppState;
use crate::frontend::WebCtx;

pub(crate) fn users_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", get(users_view))
        .route("/", post(create_user))
        .route("/{user_id}", get(user_detail_view))
        .route("/{user_id}", post(update_user))
        .route("/{user_id}", delete(delete_user))
        .route("/{user_id}/admin", post(toggle_admin))
        .route("/{user_id}/password", post(set_password))
        .route("/{user_id}/groups", post(add_to_group))
        .route("/{user_id}/groups/{group_id}", delete(remove_from_group))
}

async fn users_view(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
) -> MResponse {
    app::admin::assert_admin(&actor)?;
    let users = state.users().get_all_users().await?;
    Ok(html! {
        div class="flex flex-col items-center w-full" {
            div class="flex justify-center w-full mb-4" {
                p class="text-3xl" { "User Management" }
            }
            div id="admin-users" class="w-full max-w-3xl" {
                (user_list(&users))
            }
            div id="user-detail" class="w-full max-w-3xl" {}
        }
    })
}

fn user_list(users: &[User]) -> Markup {
    html! {
        table class="w-full text-inherit table-auto object-center responsive-card mb-4" {
            thead { tr {
                th class="w-1/12" { "ID" }
                th { "Username" }
                th { "Email" }
                th class="w-20" { "Admin" }
                th class="w-24" { "Manage" }
            } }
            tbody {
                (create_user_row())
                @for u in users {
                    tr id=(format!("user-{}", u.id)) {
                        td class="text-center opacity-70" data-label="ID" { (u.id) }
                        td data-label="Username" { (u.username) }
                        td data-label="Email" { (u.email) }
                        td data-label="Admin" class="text-center" { @if u.is_admin { "✓" } @else { "" } }
                        td class="no-label" { button class="btn btn-primary"
                            hx-get=(format!("/admin/users/{}", u.id))
                            hx-target="#user-detail" { "Manage" } }
                    }
                }
            }
        }
    }
}

/// First-row inline add. Password and admin-toggle share one column to fit
/// the 4-column layout. Handler returns the whole `#admin-users` block, so
/// a fresh empty add-row appears automatically and the username input is
/// refocused for rapid entry.
fn create_user_row() -> Markup {
    html! {
        tr id="user--1" {
            td colspan="5" {
                div class="flex flex-row flex-wrap gap-2 items-center" {
                    input class="text grow" type="text" name="username" placeholder="Username" required="required";
                    input class="text grow" type="email" name="email" placeholder="Email" required="required";
                    input class="text grow" type="password" name="password" placeholder="Password" required="required";
                    label class="flex items-center gap-1 text-sm whitespace-nowrap" {
                        input type="checkbox" name="is_admin" value="true";
                        "Admin"
                    }
                    button class="btn btn-primary"
                        hx-post="/admin/users"
                        hx-include="closest tr"
                        hx-target="#admin-users"
                        hx-swap="outerHTML"
                        hx-on::after-request="if(event.detail.successful){const i=document.querySelector('#user--1 input[name=username]');if(i)i.focus();}"
                        { "Create" }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct CreateUserForm {
    username: String,
    email: String,
    password: String,
    #[serde(default)]
    is_admin: Option<String>,
}

async fn create_user(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<CreateUserForm>,
) -> MResponse {
    app::admin::create_user(
        &state,
        &actor,
        form.username,
        form.email,
        form.password,
        form.is_admin.is_some(),
    )
    .await?;
    let users = state.users().get_all_users().await?;
    Ok(html! {
        div id="admin-users" class="w-full max-w-3xl" {
            (user_list(&users))
        }
    })
}

async fn user_detail_view(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(user_id): Path<UserId>,
) -> MResponse {
    app::admin::assert_admin(&actor)?;
    render_user_detail(&state, user_id).await
}

async fn render_user_detail(
    state: &MyAppState,
    user_id: UserId,
) -> MResponse {
    let user = state.users().get_user(user_id).await?;
    let user_groups = state.users().get_user_groups(user_id).await?;
    let all_groups = state.users().get_all_groups().await?;
    let member_ids: std::collections::HashSet<GroupId> =
        user_groups.iter().map(|g| g.id).collect();
    let candidate_groups: Vec<&Group> = all_groups
        .iter()
        .filter(|g| !g.is_personal && !member_ids.contains(&g.id))
        .collect();

    Ok(html! {
        div id="user-detail" class="w-full max-w-3xl mt-6 border-t pt-4" {
            p class="text-2xl mb-2" { "Manage \"" (user.username) "\"" }

            form class="flex flex-col gap-2 mb-4"
                hx-post=(format!("/admin/users/{}", user.id))
                hx-target="#user-detail"
                hx-swap="outerHTML" {
                p class="text-xl" { "Profile" }
                div class="flex flex-row gap-2" {
                    input class="text grow" type="text" name="username" value=(user.username) required="required";
                    input class="text grow" type="email" name="email" value=(user.email) required="required";
                    button class="btn btn-primary" type="submit" { "Save" }
                }
            }

            div class="flex flex-row gap-2 items-center mb-4" {
                p class="text-xl" { "Admin status: " }
                span { @if user.is_admin { "Yes" } @else { "No" } }
                button class="btn btn-primary"
                    hx-post=(format!("/admin/users/{}/admin", user.id))
                    hx-target="#user-detail"
                    hx-swap="outerHTML" {
                    @if user.is_admin { "Revoke admin" } @else { "Grant admin" }
                }
            }

            form class="flex flex-row gap-2 mb-4"
                hx-post=(format!("/admin/users/{}/password", user.id))
                hx-target="#user-detail"
                hx-swap="outerHTML" {
                input class="text grow" type="password" name="password" placeholder="New password" required="required";
                button class="btn btn-primary" type="submit" { "Set password" }
            }

            div class="mb-4" {
                p class="text-xl mb-2" { "Group memberships" }
                table class="w-full text-inherit table-auto responsive-card" {
                    thead { tr { th { "Name" } th class="w-24" { "Type" } th class="w-24" { "Remove" } } }
                    tbody {
                        tr id="add-group" {
                            td colspan="2" {
                                select class="fc-select w-full" name="group_id" required="required" {
                                    option value="" { "Select group to add..." }
                                    @for g in &candidate_groups {
                                        option value=(g.id) { (g.name) }
                                    }
                                }
                            }
                            td {
                                button class="btn btn-primary"
                                    hx-post=(format!("/admin/users/{}/groups", user.id))
                                    hx-include="closest tr"
                                    hx-target="#user-detail"
                                    hx-swap="outerHTML" { "Add" }
                            }
                        }
                        @for g in &user_groups {
                            tr id=(format!("user-{}-group-{}", user.id, g.id)) {
                                td data-label="Name" { (g.name) }
                                td data-label="Type" { @if g.is_personal { "personal" } @else { "shared" } }
                                td class="no-label" {
                                    @if !g.is_personal {
                                        button class="btn btn-cancel"
                                            hx-delete=(format!("/admin/users/{}/groups/{}", user.id, g.id))
                                            hx-target=(format!("#user-{}-group-{}", user.id, g.id))
                                            hx-swap="outerHTML" { "Remove" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div class="border-t pt-4" {
                button class="btn btn-cancel"
                    hx-delete=(format!("/admin/users/{}", user.id))
                    hx-target="#user-detail"
                    hx-swap="outerHTML"
                    hx-confirm="Delete this user permanently? This will also delete their personal group." {
                    "Delete user"
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct UpdateUserForm {
    username: String,
    email: String,
}

async fn update_user(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(user_id): Path<UserId>,
    Form(form): Form<UpdateUserForm>,
) -> MResponse {
    app::admin::update_user(&state, &actor, user_id, form.username, form.email).await?;
    render_user_detail(&state, user_id).await
}

async fn toggle_admin(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(user_id): Path<UserId>,
) -> MResponse {
    app::admin::toggle_admin(&state, &actor, user_id).await?;
    render_user_detail(&state, user_id).await
}

#[derive(Deserialize)]
struct PasswordForm {
    password: String,
}

async fn set_password(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(user_id): Path<UserId>,
    Form(form): Form<PasswordForm>,
) -> MResponse {
    app::admin::set_password(&state, &actor, user_id, form.password).await?;
    render_user_detail(&state, user_id).await
}

async fn delete_user(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(user_id): Path<UserId>,
) -> MResponse {
    app::admin::delete_user(&state, &actor, user_id).await?;
    Ok(html! {
        div id="user-detail" class="w-full max-w-3xl" {}
    })
}

#[derive(Deserialize)]
struct AddGroupForm {
    group_id: i32,
}

async fn add_to_group(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(user_id): Path<UserId>,
    Form(form): Form<AddGroupForm>,
) -> MResponse {
    app::admin::add_user_to_group(&state, &actor, user_id, GroupId(form.group_id)).await?;
    render_user_detail(&state, user_id).await
}

async fn remove_from_group(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path((user_id, group_id)): Path<(UserId, i32)>,
) -> MResponse {
    app::admin::remove_user_from_group(&state, &actor, user_id, GroupId(group_id)).await?;
    Ok(html! {})
}
