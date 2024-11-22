use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Form,
};
use foodlib::{AuthContext, Credenitals};
use maud::{html, Markup};
use serde::Deserialize;

use crate::MyAppState;

pub(crate) fn login_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/login/form", axum::routing::any(login_view))
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
}
#[derive(Deserialize)]
pub struct RedirectUrl {
    protected: Option<String>,
}

pub async fn login_view(Form(redirect): Form<RedirectUrl>) -> impl IntoResponse {
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

    (
        [("HX-Retarget", "#content"), ("HX-Reswap", "afterbegin")],
        html,
    )
}

#[derive(Deserialize)]
pub struct LoginData {
    protected: Option<String>,
    username: String,
    password: String,
}

fn wrong_credentials(hidden: bool) -> Markup {
    match hidden {
        true => html! { span id="login-error" hidden class="text-red-700" {} },
        false => html! { span id="login-error" class="text-red-700" { "Wrong credentials" } },
    }
}

async fn login_handler(
    mut auth: AuthContext,
    State(state): State<MyAppState>,
    Form(data): Form<LoginData>,
) -> impl IntoResponse {
    let (username, password) = (data.username, data.password);
    let Ok(user) = state
        .db_connection
        .authenticate_user(Credenitals { username, password })
        .await
    else {
        return (
            [("HX-Reswap", "outerHTML"), ("HX-Retarget", "#login-error")],
            wrong_credentials(false),
        )
            .into_response();
    };
    auth.login(&user).await.unwrap();
    (
        [("HX-Reswap", "delete"), ("HX-Retarget", "#login-dialog")],
        (),
    )
        .into_response()
}

async fn logout_handler(mut auth: AuthContext) {
    dbg!("Logging out user: {}", &auth.current_user);
    auth.logout().await;
}
