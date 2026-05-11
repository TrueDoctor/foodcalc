use axum::{
    extract::Query,
    response::IntoResponse,
    routing::{any, get, post},
    Form,
};
use foodlib_new::{
    auth::{AuthSession, Credentials},
    error::Error,
    user::User,
};
use maud::{html, Markup};
use serde::Deserialize;

use crate::MyAppState;

use super::IResponse;

pub(crate) fn login_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/login/form", any(login_view))
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
        .route("/signup/form", get(signup_view))
        .route("/signup", post(signup_handler))
}

fn signup_enabled() -> bool {
    std::env::var("SIGNUP_ENABLED")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(false)
}

#[derive(Deserialize)]
pub struct NextParam {
    next: Option<String>,
}

/// Sanitise a `?next=` redirect to a same-origin absolute path. Anything that
/// could redirect off-site (full URLs, protocol-relative `//host`, missing
/// leading `/`) collapses to `/`.
fn safe_next(raw: Option<&str>) -> String {
    match raw {
        Some(s) if s.starts_with('/') && !s.starts_with("//") => s.to_string(),
        _ => "/".to_string(),
    }
}

pub async fn login_view(user: Option<User>, Query(q): Query<NextParam>) -> IResponse {
    let next = safe_next(q.next.as_deref());
    if user.is_some() {
        return Ok((
            [("HX-Reswap", "none")],
            axum::response::Redirect::to("/auth/logout"),
        )
            .into_response());
    }
    let signup_link = format!("/auth/signup/form?next={}", urlencoding::encode(&next));
    let html = html! {
        div id="login-dialog"  {
            dialog class="dialog" open="open" id="login-dialog" {
                div class="flex items-center justify-center" {
                    form hx-post="/auth/login" class="flex flex-col gap-1 justify-items-center justify-center h-full w-full" {
                        input type="hidden" name="next" value=(next);
                        (wrong_credentials(true))
                        input class="text" type="text" name="username" placeholder="Username" id="username" {}
                        input class="text" type="password" placeholder="Password" name="password" id="password" {}
                        input class="btn btn-success" hx-swap="delete" hx-target="#login-dialog" type="submit" value="Login" {}
                        button class="btn btn-cancel" hx-on:click="document.getElementById('login-dialog').remove()" type="button" {
                            "Cancel"
                        }
                        @if signup_enabled() {
                            a hx-get=(signup_link) hx-target="#login-dialog" hx-swap="outerHTML" class="text-center text-sm underline cursor-pointer mt-2" {
                                "Need an account? Sign up"
                            }
                        }
                    }
                }
            }
        }
    };

    Ok((
        [
            ("HX-Retarget", "#content"),
            ("HX-Reswap", "afterbegin show:top"),
        ],
        html,
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct LoginData {
    username: String,
    password: String,
    #[serde(default)]
    next: Option<String>,
}

fn wrong_credentials(hidden: bool) -> Markup {
    match hidden {
        true => html! { span id="login-error" hidden class="text-red-700" {} },
        false => html! { span id="login-error" class="text-red-700" { "Wrong credentials" } },
    }
}

async fn login_handler(auth: AuthSession, Form(data): Form<LoginData>) -> impl IntoResponse {
    let next = safe_next(data.next.as_deref());
    let (username, password) = (data.username, data.password);
    let Ok(_) = foodlib_new::auth::login(auth, Credentials { username, password }).await else {
        return (
            [("HX-Reswap", "outerHTML"), ("HX-Retarget", "#login-error")],
            wrong_credentials(false),
        )
            .into_response();
    };
    ([("HX-Redirect", next.as_str())], ()).into_response()
}

fn signup_error(message: &str) -> Markup {
    html! { span id="signup-error" class="text-red-700" { (message) } }
}

async fn signup_view(user: Option<User>, Query(q): Query<NextParam>) -> IResponse {
    if !signup_enabled() {
        return Err(Error::NotFound {
            entity: "signup",
            id: "disabled".into(),
        });
    }
    let next = safe_next(q.next.as_deref());
    if user.is_some() {
        return Ok((
            [("HX-Reswap", "none")],
            axum::response::Redirect::to(&next),
        )
            .into_response());
    }
    let login_link = format!("/auth/login/form?next={}", urlencoding::encode(&next));
    let html = html! {
        div id="login-dialog" {
            dialog class="dialog" open="open" {
                div class="flex items-center justify-center" {
                    form hx-post="/auth/signup" class="flex flex-col gap-1 justify-items-center justify-center h-full w-full" {
                        input type="hidden" name="next" value=(next);
                        p class="text-xl text-center mb-2" { "Create account" }
                        span id="signup-error" hidden class="text-red-700" {}
                        input class="text" type="text" name="username" placeholder="Username" required="required";
                        input class="text" type="email" name="email" placeholder="Email" required="required";
                        input class="text" type="password" name="password" placeholder="Password" required="required";
                        input class="btn btn-success" type="submit" value="Sign up";
                        a hx-get=(login_link) hx-target="#login-dialog" hx-swap="outerHTML" class="text-center text-sm underline cursor-pointer mt-2" {
                            "Already have an account? Log in"
                        }
                        button class="btn btn-cancel" hx-on:click="document.getElementById('login-dialog').remove()" type="button" {
                            "Cancel"
                        }
                    }
                }
            }
        }
    };
    Ok(html.into_response())
}

#[derive(Deserialize)]
pub struct SignupData {
    username: String,
    email: String,
    password: String,
    #[serde(default)]
    next: Option<String>,
}

async fn signup_handler(
    auth: AuthSession,
    foodlib: crate::FoodLib,
    Form(data): Form<SignupData>,
) -> impl IntoResponse {
    if !signup_enabled() {
        return Err(Error::NotFound {
            entity: "signup",
            id: "disabled".into(),
        });
    }

    let username = data.username.trim().to_string();
    let email = data.email.trim().to_string();
    let password = data.password;
    let next = safe_next(data.next.as_deref());
    let next_login_form = format!("/auth/login/form?next={}", urlencoding::encode(&next));

    let bad = |msg: &str| {
        (
            [("HX-Reswap", "outerHTML"), ("HX-Retarget", "#signup-error")],
            signup_error(msg),
        )
            .into_response()
    };

    if username.is_empty() {
        return Ok(bad("Username is required"));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Ok(bad(
            "Username may only contain letters, digits, '-' and '_'",
        ));
    }
    if email.is_empty() || !email.contains('@') {
        return Ok(bad("A valid email is required"));
    }
    if password.len() < 8 {
        return Ok(bad("Password must be at least 8 characters"));
    }

    if foodlib
        .users()
        .get_user_by_string_reference(username.clone())
        .await
        .is_some()
    {
        return Ok(bad("That username is already taken"));
    }
    if foodlib
        .users()
        .get_user_by_string_reference(email.clone())
        .await
        .is_some()
    {
        return Ok(bad("That email is already registered"));
    }

    let credentials = Credentials {
        username: username.clone(),
        password: password.clone(),
    };
    if let Err(e) = foodlib_new::auth::register(&auth.backend, email, credentials, false).await {
        log::error!("signup failed for {username}: {e}");
        return Ok(bad("Could not create account, please try again"));
    }

    let credentials = Credentials { username, password };
    if let Err(e) = foodlib_new::auth::login(auth, credentials).await {
        log::error!("auto-login after signup failed: {e}");
        return Ok(([("HX-Redirect", next_login_form.as_str())], ()).into_response());
    }

    Ok(([("HX-Redirect", next.as_str())], ()).into_response())
}

async fn logout_handler(mut auth: AuthSession) -> impl IntoResponse {
    dbg!("Logging out user: {}", &auth.user);
    if let Err(e) = auth.logout().await {
        log::error!("failed to log out {:?}: {e}", auth.user);
    }

    (
        [
            // ("HX-Reswap", "delete"),
            // ("HX-Retarget", "#login-dialog"),
            ("HX-Redirect", "/"),
        ],
        (),
    )
}
