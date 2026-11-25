use sqlx::PgPool;
use std::sync::Arc;

pub mod allergens;
pub mod engelsystem;
pub mod export;
pub mod ingredients;
pub mod metro_sync;
pub mod properties;
pub mod recipes;
pub mod stores;
pub mod units;
// tbd
pub mod events;
pub mod inventories;
pub mod meals;
pub mod users;

#[cfg(test)]
pub mod events_test;
#[cfg(test)]
pub mod ingredients_test;
#[cfg(test)]
pub mod inventories_test;
#[cfg(test)]
pub mod meals_test;
#[cfg(test)]
pub mod recipes_test;
#[cfg(test)]
pub mod stores_test;
#[cfg(test)]
pub mod units_test;
#[cfg(test)]
pub mod users_test;

#[derive(Clone)]
pub(crate) struct FoodLib {
    pool: Arc<PgPool>,
}

impl FoodLib {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
    pub(crate) fn from_shared(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // Provide high-level API that delegates to specific operation modules
    pub(crate) fn recipes(&self) -> recipes::RecipeOps {
        recipes::RecipeOps::new(self.pool.clone())
    }

     pub(crate) fn ingredients(&self) -> ingredients::IngredientOps {
        ingredients::IngredientOps::new(self.pool.clone())
    }

     pub(crate) fn meals(&self) -> meals::MealOps {
        meals::MealOps::new(self.pool.clone())
    }

     pub(crate) fn events(&self) -> events::EventOps {
        events::EventOps::new(self.pool.clone())
    }

     pub(crate) fn inventories(&self) -> inventories::InventoryOps {
        inventories::InventoryOps::new(self.pool.clone())
    }

     pub(crate) fn users(&self) -> users::UserOps {
        users::UserOps::new(self.pool.clone())
    }

     pub(crate) fn units(&self) -> units::UnitOps {
        units::UnitOps::new(self.pool.clone())
    }

     pub(crate) fn stores(&self) -> stores::StoreOps {
        stores::StoreOps::new(self.pool.clone())
    }

     pub(crate) fn export(&self) -> export::ExportOps {
        export::ExportOps::new(self.pool.clone())
    }

     pub(crate) fn properties(&self) -> properties::PropertyOps {
        properties::PropertyOps::new(self.pool.clone())
    }

     pub(crate) fn allergens(&self) -> allergens::AllergenOps {
        allergens::AllergenOps::new(self.pool.clone())
    }

     pub(crate) fn metro_sync(&self) -> metro_sync::MetroSyncOps {
        metro_sync::MetroSyncOps::new(self.pool.clone())
    }

     pub(crate) fn pool_arc(&self) -> Arc<PgPool> {
        self.pool.clone()
    }
}