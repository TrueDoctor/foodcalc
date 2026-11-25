//! Admin use-cases: group management, user management, Metro sync.

pub mod groups;
pub mod users;



use crate::ctx::AppCtx;
use crate::ops::metro_sync::{SyncOptions, SyncReport};
use crate::{AppState, Error, Result};

/// Runs a Metro market-data sync. Only administrators may trigger it.
pub async fn run_metro_sync(
    state: &AppState,
    actor: &AppCtx,
    wipe_existing: bool,
) -> Result<SyncReport> {
    if !actor.user.is_admin {
        return Err(Error::Forbidden(
            "Only admins can trigger Metro sync".into(),
        ));
    }
    let opts = SyncOptions { wipe_existing };
    state.db().metro_sync().sync_with(None, opts).await
}

/// Admin privileges are required for every user-management use-case.
pub fn assert_admin(actor: &AppCtx) -> Result<()> {
    if actor.user.is_admin {
        Ok(())
    } else {
        Err(Error::Forbidden(
            "Admin privileges required to manage users".into(),
        ))
    }
}
