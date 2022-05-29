

use sqlx::postgres::types::PgMoney;
use sqlx::types::BigDecimal;

pub mod handler;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize, // Launch to initialize the application
    UpdateData, // Fetch Ingredients from the database
    AddIngredientSource {
        ingredient_id: i32,
        store_id: i32,
        url: String,
        weight: BigDecimal,
        price: PgMoney,
        unit: i32,
    },
}
