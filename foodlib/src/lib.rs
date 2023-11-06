use std::sync::Arc;

use sqlx::PgPool;

mod events;
mod ingredients;
mod recipes;
mod util;

pub use events::*;
pub use ingredients::*;
pub use recipes::*;
pub use util::*;

#[derive(Debug, Clone)]
pub struct FoodBase {
    pub(crate) pg_pool: Arc<PgPool>,
}

impl FoodBase {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }

    pub async fn acquire(&self) -> sqlx::pool::PoolConnection<sqlx::Postgres> {
        self.pg_pool.acquire().await.unwrap()
    }
}
