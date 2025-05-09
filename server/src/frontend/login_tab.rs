use axum::{
    response::IntoResponse,
    routing::{any, get, post},
    Form,
};
use foodlib_new::{
    auth::{AuthSession, Credentials},
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
