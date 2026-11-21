//! Source-override use-cases for events: per-event reroutes of ingredients
//! to a different source/store. (The tour/item use-cases live in
//! `app::events` itself.)

use foodlib_new::entities::event::{SourceOverride, SourceOverrideView};
use foodlib_new::entities::types::ids::{EventId, IngredientSourceId};
use foodlib_new::policy::EventPerm;

use crate::app::{AppCtx, MyAppState, Result};

/// list all source overrides for an event.
/// Requires `ViewEvent`.
pub async fn list_overrides(
    state: &MyAppState,
    ctx: &AppCtx,
    event_id: EventId,
) -> Result<Vec<SourceOverrideView>> {
    ctx.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    state.events().get_source_overrides(event_id).await
}

/// Overrides the default source for an item for this event.
/// Requires `ShoppingTourEdit`.
pub async fn add_default_override(
    state: &MyAppState,
    ctx: &AppCtx,
    event_id: EventId,
    source_id: IngredientSourceId,
) -> Result<SourceOverride> {
    ctx.ctx
        .assert_event_action(event_id, EventPerm::ShoppingTourEdit)
        .await?;
    state.events().set_source_default_override(EventId(event_id.get()), source_id).await
}

/// removes the default source override for an item for this event.
/// Requires `ShoppingTourEdit`.
pub async fn delete_override(
    state: &MyAppState,
    ctx: &AppCtx,
    event_id: EventId,
    source_id: IngredientSourceId,
) -> Result<()> {
    ctx.ctx
        .assert_event_action(event_id, EventPerm::ShoppingTourEdit)
        .await?;
    state.events().delete_source_override(EventId(event_id.get()), source_id).await
}
