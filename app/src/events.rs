//! Event use-cases.

use bigdecimal::FromPrimitive;
use foodlib_new::entities::types::ids::EventId;
use foodlib_new::entities::types::money::Money;
use foodlib_new::event::CreateEvent;

use crate::app_ctx::AppCtx;
use crate::{MyAppState, Result};

/// Creates a new event owned by the actor's personal group. An empty comment
/// is normalized to `None`.
pub async fn create_event(
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
    comment: Option<String>,
    budget: Option<f64>,
) -> Result<()> {
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    state
        .events()
        .create(CreateEvent {
            name,
            comment: comment.filter(|s| !s.is_empty()),
            budget: budget
                .and_then(bigdecimal::BigDecimal::from_f64)
                .map(Money::from),
            group_id: group.id,
        })
        .await?;
    Ok(())
}

/// Duplicates an event into the actor's personal group and returns the new
/// event's id.
/// Requires `EditEvent` permission on the source event.
pub async fn duplicate_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<EventId> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    state.events().duplicate(event_id, group.id).await
}

/// Deletes an event.
/// Requires `EditEvent` permission.
pub async fn delete_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    state.events().delete(event_id).await?;
    Ok(())
}
