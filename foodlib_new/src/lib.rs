pub use entities::*;
pub use error::{Error, Result};
use sqlx::PgPool;
use std::sync::Arc;

#[cfg(feature = "axum")]
pub mod auth;
pub mod entities;
pub mod error;
pub mod ops;

#[derive(Clone)]
pub struct FoodLib {
    pool: Arc<PgPool>,
}

impl FoodLib {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
    pub fn from_shared(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // Provide high-level API that delegates to specific operation modules
    pub fn recipes(&self) -> ops::recipes::RecipeOps {
        ops::recipes::RecipeOps::new(self.pool.clone())
    }

    pub fn ingredients(&self) -> ops::ingredients::IngredientOps {
        ops::ingredients::IngredientOps::new(self.pool.clone())
    }

    pub fn meals(&self) -> ops::meals::MealOps {
        ops::meals::MealOps::new(self.pool.clone())
    }

    pub fn events(&self) -> ops::events::EventOps {
        ops::events::EventOps::new(self.pool.clone())
    }

    pub fn inventories(&self) -> ops::inventories::InventoryOps {
        ops::inventories::InventoryOps::new(self.pool.clone())
    }

    pub fn users(&self) -> ops::users::UserOps {
        ops::users::UserOps::new(self.pool.clone())
    }

    pub fn units(&self) -> ops::units::UnitOps {
        ops::units::UnitOps::new(self.pool.clone())
    }

    pub fn stores(&self) -> ops::stores::StoreOps {
        ops::stores::StoreOps::new(self.pool.clone())
    }
}
