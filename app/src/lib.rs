//! Transport-agnostic application (use-case) layer.
//!
//! Owns business logic: permission checks (`AuthCtx`), validation,
//! `foodlib_new` orchestration, and audit logging. Transports (`frontend`,
//! `api`, `mcp`) depend on `app` and pass `(&MyAppState, &AppCtx, ...)`
//! (or `(&MyAppState, Option<&AppCtx>, ...)` for public reads).
//!
pub mod app_ctx;
pub mod state;

pub use app_ctx::AppCtx;

// Domain modules (use-case contracts, DTOs) — transport-agnostic
pub mod exports;
pub mod ingredients;
pub mod inventories;
pub mod recipes;
// pub mod events;
// pub mod ingredients;
// pub mod inventories;
// pub mod recipes;
// pub mod users;

pub use foodlib_new::error::{Error, Result};
pub use state::MyAppState;

/// Transport-agnostic error wrapper so HTTP adapters (`frontend`, `api`,
/// `mcp`) can convert use-case failures into their own response types
/// without reaching into `foodlib_new` directly.
#[derive(Debug)]
pub struct AppError(pub Error);

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// HTMX/Maud frontend.
    WebUi,
    /// JSON REST transport (SPA).
    RestApi,
    /// MCP transport — treated as automation.
    McpAgent,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::WebUi => "web-ui",
            Channel::RestApi => "rest-api",
            Channel::McpAgent => "mcp-agent",
        }
    }
}
