use std::sync::Arc;

use sqlx::PgPool;

mod events;
mod ingredients;
mod inventories;
mod meals;
mod recipes;
mod users;
mod util;

pub use events::*;
pub use ingredients::*;
pub use inventories::*;
pub use meals::*;
pub use recipes::*;
pub use users::*;

#[derive(Debug, Clone)]
pub struct FoodBase {
    pub(crate) pg_pool: Arc<PgPool>,
}

impl FoodBase {
    pub fn new_with_pool(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }

    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pg_pool = PgPool::connect(database_url).await?;
        Ok(Self::new_with_pool(pg_pool))
    }

    pub async fn acquire(&self) -> sqlx::pool::PoolConnection<sqlx::Postgres> {
        self.pg_pool.acquire().await.unwrap()
    }
}
