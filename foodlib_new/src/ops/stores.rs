// foodlib_new/src/ops/stores.rs

use sqlx::PgPool;
use std::sync::Arc;

use crate::{entities::store::*, error::Result, entities::types::ids::StoreId};

#[derive(Clone)]
pub struct StoreOps {
    pool: Arc<PgPool>,
}

impl StoreOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, store: Store) -> Result<Store> {
        let row = sqlx::query_as!(
            Store,
            r#"
            INSERT INTO stores (name)
            VALUES ($1)
            RETURNING store_id as "id", name
            "#,
            store.name,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get(&self, id: StoreId) -> Result<Store> {
        let row = sqlx::query_as!(
            Store,
            r#"
            SELECT store_id as "id", name 
            FROM stores 
            WHERE store_id = $1
            "#,
            id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Store> {
        let row = sqlx::query_as!(
            Store,
            r#"
            SELECT store_id as "id", name 
            FROM stores 
            WHERE name = $1
            "#,
            name
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update(&self, store: Store) -> Result<Store> {
        let row = sqlx::query_as!(
            Store,
            r#"
            UPDATE stores
            SET name = $1
            WHERE store_id = $2
            RETURNING store_id as "id", name
            "#,
            store.name,
            store.id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, id: StoreId) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // First handle ingredient sources
        sqlx::query!(
            r#"
            DELETE FROM ingredient_sources 
            WHERE store_id = $1
            "#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        // Delete shopping tours
        sqlx::query!(
            r#"
            DELETE FROM shopping_tours
            WHERE store_id = $1
            "#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        // Finally delete the store
        sqlx::query!(
            r#"
            DELETE FROM stores 
            WHERE store_id = $1
            "#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Store>> {
        let rows = sqlx::query_as!(
            Store,
            r#"
            SELECT store_id as "id", name 
            FROM stores
            ORDER BY name
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
