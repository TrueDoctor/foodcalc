use super::{assert_ownership_transfer, audit, AppCtx, MyAppState, Result};

use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{GroupId, IngredientId, InventoryId};
use foodlib_new::inventory::{Inventory, InventoryItem, InventoryItemWithName};
use foodlib_new::policy::{GlobalPermission, InventoryPerm, Permission};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

pub async fn list_inventories(
    state: &MyAppState,
    actor: &AppCtx,
) -> Result<Vec<Inventory>> {
    let inventories = state.inventories().list().await?;
    Ok(inventories
        .into_iter()
        .filter(|inventory| actor.ctx.can_access_group(inventory.group_id))
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct InventoryCreateInput {
    pub name: String,
    pub group_id: GroupId,
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
    actor
        .ctx
        .assert_global(GlobalPermission::CreateResources)?;
    actor.ctx.assert_group_perm(
        input.group_id,
        Permission::Inventory(InventoryPerm::Create),
    )?;

    let inventory = Inventory {
        id: InventoryId::default(),
        name: input.name,
        group_id: input.group_id,
    };
    state.inventories().create(inventory).await
}

pub async fn get_inventory(
    state: &MyAppState,
    ctx: &AppCtx,
    id: InventoryId,
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
    id: InventoryId,
    input: InventoryUpdateInput,
) -> Result<Inventory> {
    let current = state.inventories().get(id).await?;
    actor
        .ctx
        .assert_inventory_perm(id, InventoryPerm::EditMetadata)
        .await?;
    actor
        .ctx
        .assert_group_perm(current.group_id, Permission::Inventory(InventoryPerm::EditMetadata))?;

    let updated = Inventory {
        id: current.id,
        name: input.name.unwrap_or(current.name),
        group_id: current.group_id,
    };

    let saved = state.inventories().update(updated).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("inventory"),
        Some(id.into()),
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
    id: InventoryId,
) -> Result<()> {
    let current = state.inventories().get(id).await?;
    actor
        .ctx
        .assert_inventory_perm(id, InventoryPerm::Delete)
        .await?;
    actor
        .ctx
        .assert_group_perm(current.group_id, Permission::Inventory(InventoryPerm::Delete))?;

    state.inventories().delete(id).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("inventory"),
        Some(id.into()),
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
    pub ingredient_id: IngredientId,
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
    inventory_id: InventoryId,
) -> Result<Vec<InventoryItemWithName>> {
    actor
        .ctx
        .assert_inventory_perm(inventory_id, InventoryPerm::View)
        .await?;
    state.inventories().get_items(inventory_id).await
}

pub async fn add_inventory_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
    input: InventoryItemAddInput,
) -> Result<InventoryItem> {
    actor
        .ctx
        .assert_inventory_perm(inventory_id, InventoryPerm::EditStock)
        .await?;

    let amount = parse_positive_amount(&input.amount)?;

    let item = InventoryItem {
        inventory_id,
        ingredient_id: input.ingredient_id,
        amount,
    };
    let result = state.inventories().add_item(item).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_CREATED,
        Some("inventory_item"),
        Some(result.inventory_id.into()),
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
    inventory_id: InventoryId,
    ingredient_id: IngredientId,
    input: InventoryItemUpdateInput,
) -> Result<InventoryItem> {
    actor
        .ctx
        .assert_inventory_perm(inventory_id, InventoryPerm::EditStock)
        .await?;

    let amount = parse_positive_amount(&input.amount)?;

    let item = InventoryItem {
        inventory_id,
        ingredient_id,
        amount,
    };
    let updated = state.inventories().update_item(item).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("inventory_item"),
        Some(ingredient_id.into()),
        None,
        None,
        Some(json!(updated)),
    )
        .await;

    Ok(updated)
}

pub async fn remove_inventory_item(
    state: &MyAppState,
    actor: &AppCtx,
    inventory_id: InventoryId,
    ingredient_id: IngredientId,
) -> Result<()> {
    actor
        .ctx
        .assert_inventory_perm(inventory_id, InventoryPerm::EditStock)
        .await?;

    state
        .inventories()
        .delete_item(inventory_id, ingredient_id)
        .await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("inventory_item"),
        Some(ingredient_id.into()),
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
    inventory_id: InventoryId,
    target_group_id: GroupId,
) -> Result<()> {
    let current = state.inventories().get(inventory_id).await?;

    assert_ownership_transfer(
        &actor.ctx,
        "inventory",
        current.group_id,
        target_group_id,
        Permission::Inventory(InventoryPerm::TransferOwner),
    )?;

    state
        .inventories()
        .set_group(inventory_id, target_group_id)
        .await?;

    audit::record(
        state,
        actor,
        audit::action::OWNER_TRANSFERRED,
        Some("inventory"),
        Some(inventory_id.into()),
        Some(target_group_id),
        None,
        Some(json!({ "from_group_id": current.group_id })),
    )
        .await;

    Ok(())
}

// --------------------------------------------------------------- helpers

/// Validates a form- or payload-provided amount: strictly positive.
/// Shared by the REST and HTMX item handlers so zero/negative amounts can
/// never be written.
pub(crate) fn ensure_positive_amount(amount: Amount) -> Result<Amount> {
    if amount <= Amount::from(0) {
        return Err(foodlib_new::Error::InvalidInput {
            message: "amount must be positive".into(),
        });
    }
    Ok(amount)
}

fn parse_positive_amount(raw: &str) -> Result<Amount> {
    let amount = Amount::from_str(raw).map_err(|_| foodlib_new::Error::InvalidInput {
        message: format!("invalid amount value: {raw}"),
    })?;
    ensure_positive_amount(amount)
}