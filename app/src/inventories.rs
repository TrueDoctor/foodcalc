//! Inventory use-cases.

use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{GroupId, IngredientId, InventoryId};
use foodlib_new::inventory::{CreateInventory, Inventory, InventoryItem, InventoryItemWithName};

use crate::app_ctx::{assert_can_move_to, AppCtx};
use crate::{MyAppState, Result};

/// Creates a new inventory owned by the actor's personal group.
pub async fn create_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
) -> Result<()> {
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    state
        .inventories()
        .create(CreateInventory {
            name,
            group_id: group.id,
        })
        .await?;
    Ok(())
}

/// Renames an inventory and optionally transfers it to another group.
/// Requires `EditInventory` permission on the inventory; moving to another
/// group additionally requires membership in (or admin over) the target
/// group. Returns the final inventory state.
pub async fn rename_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
    name: String,
    target_group: GroupId,
) -> Result<Inventory> {
    actor.ctx.assert_can_edit_inventory(inventory_id).await?;
    let mut inv = state.inventories().get(inventory_id).await?;
    if inv.group_id != target_group {
        assert_can_move_to(actor, target_group)?;
    }
    inv.name = name;
    state.inventories().update(inv.clone()).await?;
    if inv.group_id != target_group {
        state.inventories().set_group(inventory_id, target_group).await?;
        inv.group_id = target_group;
    }
    Ok(inv)
}

/// Deletes an inventory.
/// Requires `EditInventory` permission.
pub async fn delete_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
) -> Result<()> {
    actor.ctx.assert_can_edit_inventory(inventory_id).await?;
    state.inventories().delete(inventory_id).await?;
    Ok(())
}

/// Adds one item to an inventory, or overwrites its amount when the
/// ingredient is already in the inventory. An invalid `ingredient_id` is the
/// client-side datalist sentinel ("typed a name with no match") and leaves
/// the inventory untouched so the user can correct the input.
/// Requires `EditInventory` permission.
pub async fn set_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
    ingredient_id: IngredientId,
    amount: Amount,
) -> Result<()> {
    actor.ctx.assert_can_edit_inventory(inventory_id).await?;
    if !ingredient_id.is_valid() {
        return Ok(());
    }

    let items = state.inventories().get_items(inventory_id).await?;
    let item = InventoryItem {
        inventory_id,
        ingredient_id,
        amount,
    };
    if items.iter().any(|x| x.ingredient_id == ingredient_id) {
        state.inventories().update_item(item).await?;
    } else {
        state.inventories().add_item(item).await?;
    }
    Ok(())
}

/// Updates one item's amount. Returns the item together with its ingredient
/// name for the caller's presentation layer.
/// Requires `EditInventory` permission.
pub async fn update_item_amount(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
    ingredient_id: IngredientId,
    amount: Amount,
) -> Result<InventoryItemWithName> {
    actor.ctx.assert_can_edit_inventory(inventory_id).await?;
    let updated = state
        .inventories()
        .update_item(InventoryItem {
            inventory_id,
            ingredient_id,
            amount,
        })
        .await?;
    let ingredient = state.ingredients().get(ingredient_id).await?;
    Ok(InventoryItemWithName {
        name: ingredient.name,
        ..updated.into()
    })
}

/// Removes one item from an inventory.
/// Requires `EditInventory` permission.
pub async fn delete_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
    ingredient_id: IngredientId,
) -> Result<()> {
    actor.ctx.assert_can_edit_inventory(inventory_id).await?;
    state
        .inventories()
        .delete_item(inventory_id, ingredient_id)
        .await?;
    Ok(())
}

/// Move (drag-and-drop) an item from one inventory to another. The whole
/// amount transfers; if the destination already has that ingredient the
/// amounts are summed. Moving onto itself, or an item that does not exist in
/// the source inventory, is a no-op reported as `Ok(false)`.
/// Requires `EditInventory` permission on both inventories.
///
/// Returns whether the item was moved.
pub async fn move_item(
    state: &MyAppState,
    actor: &AppCtx,
    from_id: InventoryId,
    to_id: InventoryId,
    ingredient_id: IngredientId,
) -> Result<bool> {
    if from_id == to_id {
        return Ok(false);
    }
    actor.ctx.assert_can_edit_inventory(from_id).await?;
    actor.ctx.assert_can_edit_inventory(to_id).await?;

    let source_items = state.inventories().get_items(from_id).await?;
    let Some(source_item) = source_items
        .iter()
        .find(|x| x.ingredient_id == ingredient_id)
    else {
        return Ok(false);
    };
    let amount_to_move = source_item.amount.clone();

    let dest_items = state.inventories().get_items(to_id).await?;
    let merged_amount = dest_items
        .iter()
        .find(|x| x.ingredient_id == ingredient_id)
        .map(|x| x.amount.clone() + amount_to_move.clone())
        .unwrap_or(amount_to_move);

    let dest_has = dest_items.iter().any(|x| x.ingredient_id == ingredient_id);
    let dest_item = InventoryItem {
        inventory_id: to_id,
        ingredient_id,
        amount: merged_amount,
    };
    if dest_has {
        state.inventories().update_item(dest_item).await?;
    } else {
        state.inventories().add_item(dest_item).await?;
    }
    state.inventories().delete_item(from_id, ingredient_id).await?;
    Ok(true)
}
