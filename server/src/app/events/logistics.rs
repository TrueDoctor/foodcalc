//! Logistics use-cases beyond shopping tours: food-prep read access and
//! event-inventory links with per-ingredient coverage.

use serde_json::json;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::{EventInventory, EventInventoryCoverage, FoodPrepWithRecipe};
use foodlib_new::entities::types::ids::{EventId, InventoryId};
use foodlib_new::inventory::Inventory;
use foodlib_new::policy::{EventPerm, InventoryPerm, Permission};

use crate::app::{audit, AppCtx, Error, MyAppState, Result};

/// List food-prep tasks with recipe names (timeline feed) for an event.
/// Requires `EventPerm::View` (or `EventPerm::ViewEvent`).
pub async fn list_food_preps(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: EventId,
) -> Result<Vec<FoodPrepWithRecipe>> {
    // Permission checks
    ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;

    // Action
    state.events().list_food_preps(event_id).await
}

/// List all inventories currently linked to an event.
/// Requires `EventPerm::ViewEvent`.
pub async fn list_event_inventories(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: EventId,
) -> Result<Vec<Inventory>> {
    // Permission checks
    ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;

    // Action
    state.events().list_event_inventories(event_id).await
}

/// Link an inventory to an event so its stock counts toward event coverage.
/// Linking an already-linked inventory is a validation error.
/// Non-accessible inventories surface as 404 to prevent cross-group enumeration.
/// Requires `EventPerm::Edit` on the event and `InventoryPerm::View` on the inventory.
pub async fn link_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    inventory_id: InventoryId,
) -> Result<EventInventory> {
    // Permission & semantic checks
    actor.ctx.assert_event_action(event_id, EventPerm::EditEvent).await?;

    let inventory = state.inventories().get(inventory_id).await?;
    if !actor
        .ctx
        .has_group_permission(inventory.group_id, Permission::Inventory(InventoryPerm::View))
    {
        return Err(Error::NotFound {
            entity: "Inventory",
            id: inventory_id.to_string(),
        });
    }

    let already_linked = state
        .events()
        .list_event_inventories(event_id)
        .await?
        .iter()
        .any(|inv| inv.id == inventory_id);
    if already_linked {
        return Err(Error::Validation {
            message: "inventory is already linked to this event".into(),
        });
    }

    // Action
    let link = state.events().add_inventory(event_id, inventory_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.inventory.linked",
        Some("inventory"),
        Some(inventory_id.into()),
        Some(inventory.group_id),
        Some(event_id),
        Some(json!({ "inventory_name": inventory.name })),
    )
        .await;

    Ok(link)
}

/// Unlink an inventory from an event. Unlinking an inventory that is not
/// linked to the event returns `NotFound`.
/// Requires `EventPerm::Edit` on the event.
pub async fn unlink_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    inventory_id: InventoryId,
) -> Result<()> {
    // Permission & semantic checks
    actor.ctx.assert_event_action(event_id, EventPerm::EditEvent).await?;

    let linked = state
        .events()
        .list_event_inventories(event_id)
        .await?
        .into_iter()
        .find(|inv| inv.id == inventory_id)
        .ok_or_else(|| Error::NotFound {
            entity: "EventInventory",
            id: inventory_id.to_string(),
        })?;

    // Action
    state.events().remove_inventory(event_id, inventory_id).await?;

    // Logging
    audit::record(
        state,
        actor,
        "event.inventory.unlinked",
        Some("inventory"),
        Some(inventory_id.into()),
        Some(linked.group_id),
        Some(event_id),
        Some(json!({ "inventory_name": linked.name })),
    )
        .await;

    Ok(())
}

/// Compute per-ingredient demand vs. available stock across the event's linked inventories.
/// Requires `EventPerm::ViewEvent` (and `EventPerm::ViewCalculation` if financial/cost
/// breakdown fields are included in `EventInventoryCoverage`).
pub async fn inventory_coverage(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: EventId,
) -> Result<Vec<EventInventoryCoverage>> {
    // Permission checks
    ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;

    // Action
    state.events().get_event_inventory_coverage(event_id).await
}