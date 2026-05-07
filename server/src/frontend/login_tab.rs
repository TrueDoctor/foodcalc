use axum::{
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
pub struct RedirectUrl {
    protected: Option<String>,
}

pub async fn login_view(user: Option<User>, Form(redirect): Form<RedirectUrl>) -> IResponse {
    if user.is_some() {
        return Ok((
            [("HX-Reswap", "none")],
            axum::response::Redirect::to("/auth/logout"),
        )
            .into_response());
    }
    let html = html! {
        div id="login-dialog"  {
            dialog class="dialog" open="open" id="login-dialog" {
                div class="flex items-center justify-center" {
                    form hx-post=(format!("/auth/login?protected={}", redirect.protected.unwrap_or("/".to_string()))) class="flex flex-col gap-1 justify-items-center justify-center h-full w-full" {
                        (wrong_credentials(true))
                        input class="text" type="text" name="username" placeholder="Username" id="username" {}
                        input class="text" type="password" placeholder="Password" name="password" id="password" {}
                        input class="btn btn-success" hx-swap="delete" hx-target="#login-dialog" type="submit" value="Login" {}
                        button class="btn btn-cancel" hx-on:click="document.getElementById('login-dialog').remove()" type="button" {
                            "Cancel"
                        }
                        @if signup_enabled() {
                            a hx-get="/auth/signup/form" hx-target="#login-dialog" hx-swap="outerHTML" class="text-center text-sm underline cursor-pointer mt-2" {
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
}

fn wrong_credentials(hidden: bool) -> Markup {
    match hidden {
        true => html! { span id="login-error" hidden class="text-red-700" {} },
        false => html! { span id="login-error" class="text-red-700" { "Wrong credentials" } },
    }
}

async fn login_handler(auth: AuthSession, Form(data): Form<LoginData>) -> impl IntoResponse {
    let (username, password) = (data.username, data.password);
    let Ok(_) = foodlib_new::auth::login(auth, Credentials { username, password }).await else {
        return (
            [("HX-Reswap", "outerHTML"), ("HX-Retarget", "#login-error")],
            wrong_credentials(false),
        )
            .into_response();
    };
    (
        [
            // ("HX-Reswap", "delete"),
            // ("HX-Retarget", "#login-dialog"),
            ("HX-Redirect", "/"),
        ],
        (),
    )
        .into_response()
}

fn signup_error(message: &str) -> Markup {
    html! { span id="signup-error" class="text-red-700" { (message) } }
}

async fn signup_view(user: Option<User>) -> IResponse {
    if !signup_enabled() {
        return Err(Error::NotFound {
            entity: "signup",
            id: "disabled".into(),
        });
    }
    if user.is_some() {
        return Ok((
            [("HX-Reswap", "none")],
            axum::response::Redirect::to("/"),
        )
            .into_response());
    }
    let html = html! {
        div id="login-dialog" {
            dialog class="dialog" open="open" {
                div class="flex items-center justify-center" {
                    form hx-post="/auth/signup" class="flex flex-col gap-1 justify-items-center justify-center h-full w-full" {
                        p class="text-xl text-center mb-2" { "Create account" }
                        span id="signup-error" hidden class="text-red-700" {}
                        input class="text" type="text" name="username" placeholder="Username" required="required";
                        input class="text" type="email" name="email" placeholder="Email" required="required";
                        input class="text" type="password" name="password" placeholder="Password" required="required";
                        input class="btn btn-success" type="submit" value="Sign up";
                        a hx-get="/auth/login/form" hx-target="#login-dialog" hx-swap="outerHTML" class="text-center text-sm underline cursor-pointer mt-2" {
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
        return Ok(([("HX-Redirect", "/auth/login/form")], ()).into_response());
    }

    Ok(([("HX-Redirect", "/")], ()).into_response())
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
