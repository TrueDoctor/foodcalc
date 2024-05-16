use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form,
};
use foodlib::{AuthContext, Credenitals};
use maud::html;
use serde::Deserialize;

use crate::MyAppState;

pub(crate) fn login_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/login", axum::routing::get(login_view))
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
}

pub async fn login_view(Form(redirect): Form<LoginData>) -> impl IntoResponse {
    let html = html! {
        dialog class="dialog" open="open" id="login-dialog" {
            div class="flex items-center justify-center" {
                form method="post" action=(format!("/auth/login?protected={}", redirect.protected.unwrap_or("/".to_string()))) class="flex flex-col gap-1 justify-items-center justify-center h-full w-full" {
                    input class="text" type="text" name="username" placeholder="Username" id="username" {}
                    input class="text" type="password" placeholder="Password" name="password" id="password" {}
                    input class="btn btn-success" hx-swap="delete" hx-target="#login-dialog" type="submit" value="Login" {}
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
    username: Option<String>,
    password: Option<String>,
}

async fn login_handler(
    mut auth: AuthContext,
    State(state): State<MyAppState>,
    Form(data): Form<LoginData>,
) -> Result<impl IntoResponse, StatusCode> {
    let (Some(username), Some(password)) = (data.username.clone(), data.password.clone()) else {
        return Ok(login_view(Form(data)).await.into_response());
    };
    let user = state
        .db_connection
        .authenticate_user(Credenitals { username, password })
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    auth.login(&user).await.unwrap();
    Ok(Redirect::to(&data.protected.unwrap_or("/".to_string())).into_response())
}

async fn logout_handler(mut auth: AuthContext) {
    dbg!("Logging out user: {}", &auth.current_user);
    auth.logout().await;
}
