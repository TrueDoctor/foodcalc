use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use crate::types::ids::{GroupId, IngredientId, InventoryId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inventory {
    pub id: InventoryId,
    pub name: String,
    pub group_id: GroupId,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateInventory {
    pub name: String,
    pub group_id: GroupId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryItem {
    pub inventory_id: InventoryId,
    pub ingredient_id: IngredientId,
    pub amount: BigDecimal,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct InventoryItemWithName {
    pub inventory_id: InventoryId,
    pub ingredient_id: IngredientId,
    pub amount: BigDecimal,
    pub name: String,
}

impl From<InventoryItem> for InventoryItemWithName {
    fn from(
        InventoryItem {
            inventory_id,
            ingredient_id,
            amount,
        }: InventoryItem,
    ) -> Self {
        Self {
            inventory_id,
            ingredient_id,
            amount,
            name: String::new(),
        }
    }
}
impl From<InventoryItemWithName> for InventoryItem {
    fn from(
        InventoryItemWithName {
            inventory_id,
            ingredient_id,
            amount,
            ..
        }: InventoryItemWithName,
    ) -> Self {
        Self {
            inventory_id,
            ingredient_id,
            amount,
        }
    }
}
