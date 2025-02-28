use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inventory {
    pub id: i32,
    pub name: String,
    pub owner_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryItem {
    pub inventory_id: i32,
    pub ingredient_id: i32,
    pub amount: BigDecimal,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct InventoryItemWithName {
    pub inventory_id: i32,
    pub ingredient_id: i32,
    pub amount: bigdecimal::BigDecimal,
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
