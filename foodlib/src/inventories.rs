use std::fmt::Display;
use tabled::Tabled;

use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

use crate::FoodBase;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct Inventory {
    pub inventory_id: i32,
    pub name: String,
    pub owner_id: i64,
}

#[derive(Debug, Clone)]
pub struct IngredientWithWeight {
    pub ingredient_id: i32,
    pub name: String,
    pub amount: BigDecimal,
}

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryCreate {
    pub id: Option<i32>,
    pub name: String,
}

impl Inventory {
    pub fn new(inventory_id: i32, name: String, owner_id: i64) -> Self {
        Self {
            inventory_id,
            name,
            owner_id,
        }
    }
}

impl From<Inventory> for InventoryCreate {
    fn from(value: Inventory) -> Self {
        InventoryCreate {
            id: Some(value.inventory_id),
            name: value.name,
        }
    }
}

impl InventoryCreate {
    pub fn to_inventory(&self) -> eyre::Result<Inventory> {
        let Some(id) = self.id else {
            return Err(eyre::eyre!("No id found"));
        };
        Ok(Inventory {
            inventory_id: id,
            name: self.name.clone(),
            owner_id: -1,
        })
    }
}

pub struct InventoryIngredient {
    pub inventory_id: i32,
    pub ingredient_id: i32,
    pub amount: BigDecimal,
}

impl FoodBase {
    pub async fn add_inventory(&self, name: String) -> eyre::Result<i32> {
        log::debug!("add_inventory({:?})", name);
        let inventory = sqlx::query!(
            r#"
                INSERT INTO inventories ( name )
                VALUES ( $1 )
                RETURNING inventory_id 
            "#,
            name
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(inventory.inventory_id)
    }

    pub async fn update_inventory(&self, inventory: Inventory) -> eyre::Result<i32> {
        let inventory = sqlx::query!(
            r#"
                UPDATE inventories
                SET name = $1
                WHERE inventory_id = $2
                RETURNING inventory_id
            "#,
            inventory.name,
            inventory.inventory_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(inventory.inventory_id)
    }

    pub async fn delete_inventory(&self, inventory_id: i32) -> eyre::Result<()> {
        dbg!(format!("Trying to delete inventory {}", inventory_id));
        let mut transaction = self.pg_pool.begin().await?;

        sqlx::query!(
            r#"
                DELETE FROM event_inventories
                WHERE inventory_id = $1
            "#,
            inventory_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
                DELETE FROM inventory_ingredients
                WHERE inventory_id = $1
            "#,
            inventory_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
                DELETE FROM inventories
                WHERE inventory_id = $1
            "#,
            inventory_id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_inventories(&self) -> eyre::Result<Vec<Inventory>> {
        let records = sqlx::query_as!(
            Inventory,
            r#" SELECT * FROM inventories ORDER BY inventory_id "#,
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        Ok(records)
    }

    pub async fn get_inventory_from_string_reference(
        &self,
        reference: String,
    ) -> Option<Inventory> {
        let inventory_id = reference.parse::<i32>().unwrap_or(-1);
        let records = sqlx::query_as!(
            Inventory,
            r#" 
                SELECT * FROM inventories 
                WHERE name = $1 OR inventory_id = $2
                ORDER BY inventory_id
            "#,
            reference,
            inventory_id
        )
        .fetch_one(&*self.pg_pool)
        .await;

        if let Ok(record) = records {
            Some(record)
        } else {
            None
        }
    }

    pub async fn get_inventory_from_id(&self, id: i32) -> Option<Inventory> {
        let records = sqlx::query_as!(
            Inventory,
            r#" 
                SELECT * FROM inventories 
                WHERE inventory_id = $1
                ORDER BY inventory_id
            "#,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await;

        if let Ok(record) = records {
            Some(record)
        } else {
            None
        }
    }

    pub async fn get_filtered_inventory_contents(
        &self,
        inventory_id: i32,
        filter: Option<String>,
    ) -> eyre::Result<Vec<IngredientWithWeight>> {
        let filter_string = format!("%{}%", filter.unwrap_or_default());
        let records = sqlx::query_as!(
            IngredientWithWeight,
            r#" 
                SELECT ingredient_id, name, amount FROM ingredients
                INNER JOIN inventory_ingredients USING (ingredient_id) 
                WHERE inventory_id = $1 AND LOWER(name) LIKE LOWER($2)
                ORDER BY name
            "#,
            inventory_id,
            filter_string
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        Ok(dbg!(records))
    }

    pub async fn delete_inventory_item(&self, item: InventoryIngredient) -> eyre::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM inventory_ingredients WHERE 
                inventory_id = $1 AND
                ingredient_id = $2
            "#,
            item.inventory_id,
            item.ingredient_id
        )
        .execute(&*self.pg_pool)
        .await?;

        Ok(())
    }

    pub async fn update_inventory_item(&self, values: InventoryIngredient) -> eyre::Result<()> {
        // TODO: Maybe change so it will only update once, when the inventory is quit
        let mut transaction = self.pg_pool.begin().await?;

        sqlx::query!(
            r#"
                DELETE FROM inventory_ingredients WHERE 
                inventory_id = $1 AND
                ingredient_id = $2
            "#,
            values.inventory_id,
            values.ingredient_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
                INSERT INTO inventory_ingredients ( inventory_id, ingredient_id, amount ) VALUES
                    ( $1, $2, $3 )
            "#,
            values.inventory_id,
            values.ingredient_id,
            values.amount
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }
}

mod tests {}
