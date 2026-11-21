use super::{assert_ownership_transfer, audit, AppCtx, MyAppState, Result};
use bigdecimal::BigDecimal;
use foodlib_new::inventory::{Inventory, InventoryItem, InventoryItemWithName};
use foodlib_new::policy::{InventoryPerm, Permission};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

pub async fn list_inventories(
    state: &MyAppState,
    actor: &AppCtx,
    // query: IngredientListQuery,
) -> Result<Vec<Inventory>> {
    let inventories = state.inventories().list().await?;
    let visible: Vec<Inventory> = inventories
        .into_iter()
        .filter(|inventory| actor.ctx.can_access_group(inventory.group_id))
        .collect();
    Ok(visible)
}


#[derive(Debug, Deserialize)]
pub struct InventoryCreateInput {
    pub name: String,
    pub group_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct InventoryUpdateInput {
    pub name: Option<String>,
}

pub async fn create_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    input: InventoryCreateInput,
) -> Result<Inventory> {
    // Check user is blocked
    actor.ctx.assert_global(foodlib_new::policy::GlobalPermission::AllowedToCreateNewDomainObjects)?;
    actor.ctx.assert_group_perm(input.group_id, foodlib_new::policy::Permission::Inventory(foodlib_new::policy::InventoryPerm::Create))?;

    // Act
    let inventory = Inventory {
        id: 0,
        name: input.name,
        group_id: input.group_id,
    };
    let created = state.inventories().create(inventory).await?;
    // Return
    Ok(created)
}

pub async fn get_inventory(
    state: &MyAppState,
    ctx: &AppCtx,
    id: i32,
) -> Result<Inventory> {
    let inventory = state.inventories().get(id).await?;
    if !ctx.ctx.can_access_group(inventory.group_id) {
        return Err(foodlib_new::Error::NotFound {
            entity: "Inventory",
            id: id.to_string(),
        });
    }
    Ok(inventory)
}

pub async fn update_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
    input: InventoryUpdateInput,
) -> Result<Inventory> {
    // Get current inventory
    let current = state.inventories().get(id).await?;
    // Check permissions
    actor.ctx.assert_inventory_perm(id, foodlib_new::policy::InventoryPerm::Edit).await?;
    actor.ctx.assert_group_perm(current.group_id, foodlib_new::policy::Permission::Inventory(foodlib_new::policy::InventoryPerm::Edit))?;

    // Update fields
    let name = input.name.unwrap_or(current.name);
    let updated = Inventory {
        id: current.id,
        name,
        group_id: current.group_id,
    };

    // Save
    let saved = state.inventories().update(updated).await?;

    // Log the update action
    audit::record(
        state,
        actor,
        foodlib_new::ops::audit::action::ENTITY_UPDATED,
        Some("inventory"),
        Some(id),
        Some(current.group_id),
        None,
        Some(json!(saved)),
    )
        .await;

    Ok(saved)
}

pub async fn delete_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    id: i32,
) -> Result<()> {
    // Get current inventory
    let current = state.inventories().get(id).await?;
    // Check permissions
    actor.ctx.assert_inventory_perm(id, foodlib_new::policy::InventoryPerm::Delete).await?;
    actor.ctx.assert_group_perm(current.group_id, foodlib_new::policy::Permission::Inventory(foodlib_new::policy::InventoryPerm::Delete))?;

    // Delete
    state.inventories().delete(id).await?;

    // Log the delete action
    audit::record(
        state,
        actor,
        foodlib_new::ops::audit::action::ENTITY_DELETED,
        Some("inventory"),
        Some(id),
        Some(current.group_id),
        None,
        None,
    )
        .await;

    Ok(())
}


// ---------------------------------------------------------------- items
#[derive(Debug, Deserialize)]
pub struct InventoryItemAddInput {
    pub ingredient_id: i32,
    /// Decimal string, amount in kg.
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct InventoryItemUpdateInput {
    /// Decimal string, amount in kg.
    pub amount: String,
}

pub async fn list_inventory_items(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: i32,
) -> Result<Vec<InventoryItemWithName>> {
    // Check permissions
    actor.ctx.assert_inventory_perm(inventory_id, foodlib_new::policy::InventoryPerm::View).await?;
    let items = state.inventories().get_items(inventory_id).await?;
    Ok(items)
}

pub async fn add_inventory_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: i32,
    input: InventoryItemAddInput,
) -> Result<InventoryItem> {
    // Check permissions
    actor.ctx.assert_inventory_perm(inventory_id, foodlib_new::policy::InventoryPerm::Edit).await?;

    let amount: BigDecimal = BigDecimal::from_str(&input.amount)
        .map_err(|_|             foodlib_new::Error::InvalidInput {
            message: format!("invalid amount value: {}", input.amount)
        })?;
    if amount <= BigDecimal::from(0) {
        return Err(foodlib_new::Error::InvalidInput {
            message: format!("amount must be positive: {}", input.amount),
        });
    }

    // Add item
    let item = InventoryItem {
        inventory_id,
        ingredient_id: input.ingredient_id,
        amount,
    };
    let result = state.inventories().add_item(item).await?;

    // Log the add action
    audit::record(
        state,
        actor,
        foodlib_new::ops::audit::action::ENTITY_CREATED,
        Some("inventory_item"),
        Some(result.inventory_id),
        None,
        None,
        Some(json!(result)),
    )
        .await;

    Ok(result)
}

pub async fn update_inventory_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: i32,
    ingredient_id: i32,
    input: InventoryItemUpdateInput,
) -> Result<InventoryItem> {
    // Check permissions
    actor.ctx.assert_inventory_perm(inventory_id, foodlib_new::policy::InventoryPerm::Edit).await?;

    let amount: BigDecimal = BigDecimal::from_str(&input.amount)
        .map_err(|_|             foodlib_new::Error::InvalidInput {
            message: format!("invalid amount value: {}", input.amount)
        })?;
    if amount <= BigDecimal::from(0) {
        return Err(foodlib_new::Error::InvalidInput {
            message: format!("amount must be positive: {}", input.amount),
        });
    }

    // Update item
    let item = InventoryItem {
        inventory_id,
        ingredient_id,
        amount,
    };
    let item = state.inventories().update_item(item).await?;

    // Log the update action
    audit::record(
        state,
        actor,
        foodlib_new::ops::audit::action::ENTITY_UPDATED,
        Some("inventory_item"),
        Some(ingredient_id),
        None,
        None,
        Some(json!(item)),
    )
        .await;

    Ok(item)
}

pub async fn remove_inventory_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: i32,
    ingredient_id: i32,
) -> Result<()> {
    // Check permissions
    actor.ctx.assert_inventory_perm(inventory_id, foodlib_new::policy::InventoryPerm::Edit).await?;

    // Remove item
    state.inventories().delete_item(inventory_id, ingredient_id).await?;

    // Log the delete action
    audit::record(
        state,
        actor,
        foodlib_new::ops::audit::action::ENTITY_DELETED,
        Some("inventory_item"),
        Some(ingredient_id),
        None,
        None,
        None,
    )
        .await;

    Ok(())
}

// --------------------------------------------------------------- ownership
pub async fn transfer_inventory_ownership(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: i32,
    target_group_id: i32,
) -> Result<()> {
    let ctx = &actor.ctx;
    let current = state.inventories().get(inventory_id).await?;

    assert_ownership_transfer(
        ctx,
        "inventory",
        current.group_id,
        target_group_id,
        Permission::Inventory(InventoryPerm::TransferOwner),
    )?;

    state.inventories().set_group(inventory_id, target_group_id).await?;

    audit::record(
        state,
        actor,
        foodlib_new::ops::audit::action::OWNER_TRANSFERRED,
        Some("inventory"),
        Some(inventory_id),
        Some(target_group_id),
        None,
        Some(json!({ "from_group_id": current.group_id })),
    )
        .await;

    Ok(())
}