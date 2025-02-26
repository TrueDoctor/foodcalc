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

    pub async fn create(&self, inventory: Inventory) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            INSERT INTO inventories (name, owner_id)
            VALUES ($1, $2)
            RETURNING inventory_id as "id", name, owner_id
            "#,
            inventory.name,
            inventory.owner_id,
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
            RETURNING inventory_id as "id", name, owner_id
            "#,
            inventory.name,
            inventory.id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        sqlx::query!(r#"DELETE FROM inventories WHERE inventory_id = $1"#, id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get(&self, id: i32) -> Result<Inventory> {
        let row = sqlx::query_as!(
            Inventory,
            r#"
            SELECT inventory_id as "id", name, owner_id
            FROM inventories
            WHERE inventory_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list(&self) -> Result<Vec<Inventory>> {
        let rows = sqlx::query_as!(
            Inventory,
            r#"
            SELECT inventory_id as "id", name, owner_id
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

    pub async fn update_item(&self, item: InventoryItem) -> Result<InventoryItem> {
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

    pub async fn delete_item(&self, inventory_id: i32, ingredient_id: i32) -> Result<()> {
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

    pub async fn get_items(&self, inventory_id: i32) -> Result<Vec<InventoryItemWithName>> {
        let rows = sqlx::query_as!(
            InventoryItemWithName,
            r#"
            SELECT inventory_id, ingredient_id, amount, name
            FROM inventory_ingredients JOIN ingredients USING(ingredient_id)
            WHERE inventory_id = $1
            "#,
            inventory_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
