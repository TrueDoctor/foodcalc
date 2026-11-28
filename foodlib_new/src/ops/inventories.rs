use crate::types::ids::{GroupId, IngredientId, InventoryId};
use crate::{
    entities::inventory::*,
    error::Result,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct InventoryOps {
    pool: Arc<PgPool>,
}

impl InventoryOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, inventory: Inventory) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            INSERT INTO inventories (name, group_id)
            VALUES ($1, $2)
            RETURNING inventory_id as "id: InventoryId", name, group_id as "group_id: GroupId"
            "#,
            inventory.name,
            inventory.group_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update(&self, inventory: Inventory) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            UPDATE inventories
            SET name = $1
            WHERE inventory_id = $2
            RETURNING inventory_id as "id: InventoryId", name, group_id as "group_id: GroupId"
            "#,
            inventory.name,
            inventory.id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn set_group(&self, inventory_id: InventoryId, group_id: GroupId) -> Result<()> {
        sqlx::query!(
            r#"UPDATE inventories SET group_id = $1 WHERE inventory_id = $2"#,
            group_id.get(),
            inventory_id.get()
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: InventoryId) -> Result<()> {
        sqlx::query!(r#"DELETE FROM inventories WHERE inventory_id = $1"#, id.get())
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get(&self, id: InventoryId) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            SELECT inventory_id as "id: InventoryId", name, group_id as "group_id: GroupId"
            FROM inventories
            WHERE inventory_id = $1
            "#,
            id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list(&self) -> Result<Vec<Inventory>> {
        let rows = sqlx::query_as!(
            Inventory,
            r#"
            SELECT inventory_id as "id: InventoryId", name, group_id as "group_id: GroupId"
            FROM inventories
            ORDER BY name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn add_item(&self, item: InventoryItem) -> Result<InventoryItem> {
        let row = sqlx::query_as!(
            InventoryItem,
            r#"
            INSERT INTO inventory_ingredients (inventory_id, ingredient_id, amount)
            VALUES ($1, $2, $3)
            RETURNING inventory_id as "inventory_id: InventoryId", ingredient_id as "ingredient_id: IngredientId", amount
            "#,
            item.inventory_id.get(),
            item.ingredient_id.get(),
            item.amount
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_item(&self, item: InventoryItem) -> Result<InventoryItem> {
        let row = sqlx::query_as!(
            InventoryItem,
            r#"
            UPDATE inventory_ingredients
            SET amount = $1
            WHERE inventory_id = $2 AND ingredient_id = $3
            RETURNING inventory_id as "inventory_id: InventoryId", ingredient_id as "ingredient_id: IngredientId", amount
            "#,
            item.amount,
            item.inventory_id.get(),
            item.ingredient_id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_item(
        &self,
        inventory_id: InventoryId,
        ingredient_id: IngredientId,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM inventory_ingredients
            WHERE inventory_id = $1 AND ingredient_id = $2
            "#,
            inventory_id.get(),
            ingredient_id.get()
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_items(&self, inventory_id: InventoryId) -> Result<Vec<InventoryItemWithName>> {
        let rows = sqlx::query_as!(
            InventoryItemWithName,
            r#"
            SELECT inventory_id as "inventory_id: InventoryId", ingredient_id as "ingredient_id: IngredientId", amount, name
            FROM inventory_ingredients JOIN ingredients USING(ingredient_id)
            WHERE inventory_id = $1
            "#,
            inventory_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
