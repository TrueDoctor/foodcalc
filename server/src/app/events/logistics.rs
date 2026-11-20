//! Logistics use-cases beyond shopping tours: food-prep read access and
//! event-inventory links with per-ingredient coverage.

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::{EventInventory, EventInventoryCoverage, FoodPrepWithRecipe};
use foodlib_new::inventory::Inventory;
use foodlib_new::policy::EventPerm;

use crate::app::{AppCtx, Error, MyAppState, Result};

/// `GET /api/events/{id}/preps` — food prep tasks with recipe names
/// (timeline feed). Requires `events:view`.
pub async fn list_food_preps(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: i32,
) -> Result<Vec<FoodPrepWithRecipe>> {
    ctx.assert_event_action(event_id, EventPerm::View).await?;
    state.events().list_food_preps(event_id).await
}

/// `GET /api/events/{id}/inventories` — the event's linked inventories.
/// Requires `events:view`.
pub async fn list_event_inventories(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: i32,
) -> Result<Vec<Inventory>> {
    ctx.assert_event_action(event_id, EventPerm::View).await?;
    state.events().list_event_inventories(event_id).await
}

/// `POST /api/events/{id}/inventories/{inventory_id}` — link an inventory to
/// the event. Requires event write access and read access to the inventory
/// (non-accessible inventories surface as a plain 404).
pub async fn link_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    inventory_id: i32,
) -> Result<EventInventory> {
    actor.ctx.assert_event_action(event_id, EventPerm::Edit).await?;
    let inventory = state.inventories().get(inventory_id).await?;
    if !actor.ctx.can_access_group(inventory.group_id) {
        return Err(Error::NotFound {
            entity: "Inventory",
            id: inventory_id.to_string(),
        });
    }
    state.events().add_inventory(event_id, inventory_id).await
}

/// `DELETE /api/events/{id}/inventories/{inventory_id}` — unlink. Requires
/// event write access.
pub async fn unlink_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    inventory_id: i32,
) -> Result<()> {
    actor.ctx.assert_event_action(event_id, EventPerm::Edit).await?;
    state.events().remove_inventory(event_id, inventory_id).await
}

/// `GET /api/events/{id}/inventory-coverage` — per-ingredient demand vs. the
/// stock of the event's linked inventories. Requires `events:view`.
pub async fn inventory_coverage(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: i32,
) -> Result<Vec<EventInventoryCoverage>> {
    ctx.assert_event_action(event_id, EventPerm::View).await?;
    state.events().get_event_inventory_coverage(event_id).await
}
