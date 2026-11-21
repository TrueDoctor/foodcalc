//! Source-override use-cases for events: per-event reroutes of ingredients
//! to a different source/store. (The tour/item use-cases live in
//! `app::events` itself.)

use foodlib_new::entities::event::{SourceOverride, SourceOverrideView};
use foodlib_new::policy::EventPerm;

use crate::app::{AppCtx, MyAppState, Result};

/// `GET /api/events/{id}/source-overrides` — per-event reroutes of
/// ingredients to a different source/store. Requires `events:view`.
pub async fn list_overrides(
    state: &MyAppState,
    ctx: &AppCtx,
    event_id: i32,
) -> Result<Vec<SourceOverrideView>> {
    ctx.ctx.assert_event_action(event_id, EventPerm::View).await?;
    state.events().get_source_overrides(event_id).await
}

/// `POST /api/events/{id}/source-overrides` — reroute an ingredient source
/// for this event (idempotent upsert). Requires `events:manage_shopping`.
pub async fn add_override(
    state: &MyAppState,
    ctx: &AppCtx,
    event_id: i32,
    source_id: i32,
) -> Result<SourceOverride> {
    ctx.ctx
        .assert_event_action(event_id, EventPerm::ManageShopping)
        .await?;
    state.events().set_source_override(event_id, source_id).await
}

/// `DELETE /api/events/{id}/source-overrides/{source_id}` — remove a reroute.
/// Requires `events:manage_shopping`.
pub async fn delete_override(
    state: &MyAppState,
    ctx: &AppCtx,
    event_id: i32,
    source_id: i32,
) -> Result<()> {
    ctx.ctx
        .assert_event_action(event_id, EventPerm::ManageShopping)
        .await?;
    state.events().delete_source_override(event_id, source_id).await
}
