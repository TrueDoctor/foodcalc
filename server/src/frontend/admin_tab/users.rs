use axum::{
    extract::{Form, Path},
    routing::{delete, get, post},
};
use foodlib_new::{
    error::{Error, Result},
    user::{Group, User},
};
use maud::{html, Markup};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{FoodLib, MyAppState};
use foodlib_new::auth_context::AuthCtx;

use crate::frontend::MResponse;

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

fn assert_admin(ctx: &AuthCtx) -> Result<()> {
    if ctx.user.is_admin {
        Ok(())
    } else {
        Err(Error::Forbidden(
            "Admin privileges required to manage users".into(),
        ))
    }
}

async fn users_view(foodlib: FoodLib, ctx: AuthCtx) -> MResponse {
    assert_admin(&ctx)?;
    let users = foodlib.users().get_all_users().await?;
    Ok(html! {
        div class="flex flex-col items-center w-full" {
            div class="flex justify-center w-full mb-4" {
                p class="text-3xl" { "User Management" }
            }
            div id="admin-users" class="w-full max-w-3xl" {
                (user_list(&users))
                (create_user_form())
            }
            div id="user-detail" class="w-full max-w-3xl" {}
        }
    })
}

fn user_list(users: &[User]) -> Markup {
    html! {
        table class="w-full text-inherit table-auto object-center table-fixed mb-4" {
            thead { tr {
                th { "Username" }
                th { "Email" }
                th class="w-20" { "Admin" }
                th class="w-24" { "Manage" }
            } }
            tbody {
                @for u in users {
                    tr id=(format!("user-{}", u.id)) {
                        td { (u.username) }
                        td { (u.email) }
                        td class="text-center" { @if u.is_admin { "✓" } @else { "" } }
                        td { button class="btn btn-primary"
                            hx-get=(format!("/admin/users/{}", u.id))
                            hx-target="#user-detail" { "Manage" } }
                    }
                }
            }
        }
    }
}

fn create_user_form() -> Markup {
    html! {
        form class="flex flex-col gap-2 mb-4 p-4 border rounded"
            hx-post="/admin/users"
            hx-target="#admin-users"
            hx-swap="outerHTML" {
            p class="text-xl" { "Create user" }
            div class="flex flex-row gap-2" {
                input class="text grow" type="text" name="username" placeholder="Username" required="required";
                input class="text grow" type="email" name="email" placeholder="Email" required="required";
            }
            div class="flex flex-row gap-2" {
                input class="text grow" type="password" name="password" placeholder="Password" required="required";
                label class="flex items-center gap-2" {
                    input type="checkbox" name="is_admin" value="true";
                    "Admin"
                }
                button class="btn btn-primary" type="submit" { "Create" }
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
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(form): Form<CreateUserForm>,
) -> MResponse {
    assert_admin(&ctx)?;
    let username = form.username.trim();
    let email = form.email.trim();
    if username.is_empty() || email.is_empty() || form.password.is_empty() {
        return Err(Error::Validation {
            message: "Username, email, and password are required".into(),
        });
    }
    let user = User {
        id: -1,
        username: username.to_string(),
        email: email.to_string(),
        password_hash: String::new(),
        is_admin: form.is_admin.is_some(),
        created_at: OffsetDateTime::now_utc(),
    };
    let (created, _) = foodlib.users().create_user_with_personal_group(user).await?;
    foodlib
        .users()
        .update_password(created.id, form.password)
        .await?;
    let users = foodlib.users().get_all_users().await?;
    Ok(html! {
        div id="admin-users" class="w-full max-w-3xl" {
            (user_list(&users))
            (create_user_form())
        }
    })
}

async fn user_detail_view(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(user_id): Path<i64>,
) -> MResponse {
    assert_admin(&ctx)?;
    render_user_detail(&foodlib, user_id).await
}

async fn render_user_detail(foodlib: &foodlib_new::FoodLib, user_id: i64) -> MResponse {
    let user = foodlib.users().get_user(user_id).await?;
    let user_groups = foodlib.users().get_user_groups(user_id).await?;
    let all_groups = foodlib.users().get_all_groups().await?;
    let member_ids: std::collections::HashSet<i32> = user_groups.iter().map(|g| g.id).collect();
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
                table class="w-full text-inherit table-auto" {
                    thead { tr { th { "Name" } th class="w-24" { "Type" } th class="w-24" { "Remove" } } }
                    tbody {
                        @for g in &user_groups {
                            tr id=(format!("user-{}-group-{}", user.id, g.id)) {
                                td { (g.name) }
                                td { @if g.is_personal { "personal" } @else { "shared" } }
                                td {
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
                form class="flex flex-row gap-2 mt-2"
                    hx-post=(format!("/admin/users/{}/groups", user.id))
                    hx-target="#user-detail"
                    hx-swap="outerHTML" {
                    select class="fc-select grow" name="group_id" required="required" {
                        option value="" { "Select group to add..." }
                        @for g in &candidate_groups {
                            option value=(g.id) { (g.name) }
                        }
                    }
                    button class="btn btn-primary" type="submit" { "Add to group" }
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
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(user_id): Path<i64>,
    Form(form): Form<UpdateUserForm>,
) -> MResponse {
    assert_admin(&ctx)?;
    let mut user = foodlib.users().get_user(user_id).await?;
    user.username = form.username.trim().to_string();
    user.email = form.email.trim().to_string();
    foodlib.users().update_user(user).await?;
    render_user_detail(&foodlib, user_id).await
}

async fn toggle_admin(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(user_id): Path<i64>,
) -> MResponse {
    assert_admin(&ctx)?;
    if user_id == ctx.user.id {
        return Err(Error::Forbidden(
            "You cannot change your own admin status".into(),
        ));
    }
    let mut user = foodlib.users().get_user(user_id).await?;
    user.is_admin = !user.is_admin;
    foodlib.users().update_user(user).await?;
    render_user_detail(&foodlib, user_id).await
}

#[derive(Deserialize)]
struct PasswordForm {
    password: String,
}

async fn set_password(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(user_id): Path<i64>,
    Form(form): Form<PasswordForm>,
) -> MResponse {
    assert_admin(&ctx)?;
    if form.password.is_empty() {
        return Err(Error::Validation {
            message: "Password cannot be empty".into(),
        });
    }
    foodlib.users().update_password(user_id, form.password).await?;
    render_user_detail(&foodlib, user_id).await
}

async fn delete_user(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(user_id): Path<i64>,
) -> MResponse {
    assert_admin(&ctx)?;
    if user_id == ctx.user.id {
        return Err(Error::Forbidden("You cannot delete yourself".into()));
    }
    foodlib.users().delete_user_cascade(user_id).await?;
    Ok(html! {
        div id="user-detail" class="w-full max-w-3xl" {}
    })
}

#[derive(Deserialize)]
struct AddGroupForm {
    group_id: i32,
}

async fn add_to_group(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(user_id): Path<i64>,
    Form(form): Form<AddGroupForm>,
) -> MResponse {
    assert_admin(&ctx)?;
    foodlib
        .users()
        .add_user_to_group(user_id, form.group_id)
        .await?;
    render_user_detail(&foodlib, user_id).await
}

async fn remove_from_group(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((user_id, group_id)): Path<(i64, i32)>,
) -> MResponse {
    assert_admin(&ctx)?;
    let group = foodlib.users().get_group(group_id).await?;
    if group.is_personal {
        return Err(Error::Forbidden(
            "Cannot remove a user from their personal group".into(),
        ));
    }
    foodlib
        .users()
        .remove_user_from_group(user_id, group_id)
        .await?;
    Ok(html! {})
}
