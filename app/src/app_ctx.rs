use crate::Channel;
use foodlib_new::Error;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::types::ids::GroupId;

use crate::Result;

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

/// Authorizes moving a resource into `target_group_id`: admins may move
/// anywhere; non-admins may only move into groups they belong to.
pub fn assert_can_move_to(actor: &AppCtx, target_group_id: GroupId) -> Result<()> {
    if actor.ctx.user.is_admin || actor.ctx.group_ids.contains(&target_group_id) {
        Ok(())
    } else {
        Err(Error::Forbidden(
            "You can only move resources into groups you belong to".into(),
        ))
    }
}