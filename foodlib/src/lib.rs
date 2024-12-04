use std::sync::Arc;

use foodlib_new::{error::Error as NewError, FoodLib};
use sqlx::PgPool;

mod events;
mod ingredients;
mod inventories;
mod meals;
mod recipes;
#[cfg(feature = "typst")]
pub mod typst;
mod users;
mod util;

pub use events::*;
pub use ingredients::*;
pub use inventories::*;
pub use meals::*;
pub use recipes::*;
pub use users::*;

type PrimitiveDateTime = time::OffsetDateTime;

#[derive(Clone)]
pub struct FoodBase {
    pg_pool: Arc<PgPool>,
    new_lib: FoodLib,
}

impl FoodBase {
    pub fn new_with_pool(pg_pool: PgPool) -> Self {
        let pool = Arc::new(pg_pool);
        Self {
            pg_pool: pool.clone(),
            new_lib: FoodLib::from_shared(pool),
        }
    }

    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pg_pool = PgPool::connect(database_url).await?;
        Ok(Self::new_with_pool(pg_pool))
    }

    pub async fn acquire(&self) -> sqlx::pool::PoolConnection<sqlx::Postgres> {
        self.pg_pool.acquire().await.unwrap()
    }

    pub fn pool(&self) -> &PgPool {
        &self.pg_pool
    }
}
