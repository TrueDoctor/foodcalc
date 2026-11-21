//! `WebCtx` — app-layer context for legacy HTMX handlers.
//!
//! Thin handlers take `ctx: WebCtx` directly and pass `&ctx` into the
//! `app::` use-cases. It resolves the same `AuthCtx` as the bare `AuthCtx`
//! extractor, but wraps it into an `AppCtx` tagged with [`Channel::WebUi`],
//! so audit records distinguish UI actions from REST/MCP traffic.
//!
//! (A newtype instead of a blanket `FromRequestParts` impl for `AppCtx`:
//! `api` already implements that trait for `AppCtx` with `Channel::RestApi`,
//! and only one impl can exist.)

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use std::ops::Deref;

use crate::app::AppCtx;
use crate::app::Channel;
use crate::frontend::LOGIN_URL;
use crate::MyAppState;

#[derive(Debug, Clone)]
pub struct WebCtx(pub AppCtx);

impl Deref for WebCtx {
    type Target = AppCtx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<MyAppState> for WebCtx {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &MyAppState,
    ) -> Result<Self, Self::Rejection> {
        let ctx = foodlib_new::auth_context::AuthCtx::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                crate::frontend::html_error(&format!("{err}"), LOGIN_URL).into_response()
            })?;
        Ok(WebCtx(AppCtx::new(ctx, Channel::WebUi)))
    }
}
