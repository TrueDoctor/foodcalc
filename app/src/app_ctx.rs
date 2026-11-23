use crate::Channel;
use foodlib_new::Error;
use foodlib_new::auth_context::AuthCtx;

/// Lightweight authenticated actor context: the resolved `AuthCtx` and the
/// transport channel it arrived on. Infrastructure (`MyAppState`) is passed
/// separately as `&MyAppState`.
#[derive(Clone)]
pub struct AppCtx {
    pub ctx: AuthCtx,
    pub channel: Channel,
}

impl AppCtx {
    pub fn new(ctx: AuthCtx, channel: Channel) -> Self {
        Self { ctx, channel }
    }

    /// Rejects automation channels (MCP) for sensitive use-cases.
    pub fn require_human(&self) -> foodlib_new::Result<()> {
        if self.channel == Channel::McpAgent {
            return Err(Error::Forbidden(
                "this action is not available to automation (MCP); use the web UI".into(),
            ));
        }
        Ok(())
    }
}

impl std::fmt::Debug for AppCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppCtx")
            .field("user", &self.ctx.user.username)
            .field("channel", &self.channel)
            .finish_non_exhaustive()
    }
}