use crate::{entities::inventory::*, error::Result};
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

    pub async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            INSERT INTO inventories (name)
            VALUES ($1)
            RETURNING inventory_id as "id", name
            "#,
            inventory.name
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            UPDATE inventories
            SET name = $1
            WHERE inventory_id = $2
            RETURNING inventory_id as "id", name
            "#,
            inventory.name,
            inventory.id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_inventory(&self, id: i32) -> Result<()> {
        sqlx::query!(r#"DELETE FROM inventories WHERE inventory_id = $1"#, id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_inventory(&self, id: i32) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            SELECT inventory_id as "id", name
            FROM inventories
            WHERE inventory_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_all_inventories(&self) -> Result<Vec<Inventory>> {
        let rows = sqlx::query_as!(
            Inventory,
            r#"
            SELECT inventory_id as "id", name
            FROM inventories
            ORDER BY name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn add_inventory_item(&self, item: InventoryItem) -> Result<InventoryItem> {
        let row = sqlx::query_as!(
            InventoryItem,
            r#"
            INSERT INTO inventory_ingredients (inventory_id, ingredient_id, amount)
            VALUES ($1, $2, $3)
            RETURNING inventory_id, ingredient_id, amount
            "#,
            item.inventory_id,
            item.ingredient_id,
            item.amount
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_inventory_item(&self, item: InventoryItem) -> Result<InventoryItem> {
        let row = sqlx::query_as!(
            InventoryItem,
            r#"
            UPDATE inventory_ingredients
            SET amount = $1
            WHERE inventory_id = $2 AND ingredient_id = $3
            RETURNING inventory_id, ingredient_id, amount
            "#,
            item.amount,
            item.inventory_id,
            item.ingredient_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_inventory_item(&self, inventory_id: i32, ingredient_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM inventory_ingredients
            WHERE inventory_id = $1 AND ingredient_id = $2
            "#,
            inventory_id,
            ingredient_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_inventory_items(&self, inventory_id: i32) -> Result<Vec<InventoryItem>> {
        let rows = sqlx::query_as!(
            InventoryItem,
            r#"
            SELECT inventory_id, ingredient_id, amount
            FROM inventory_ingredients
            WHERE inventory_id = $1
            "#,
            inventory_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
