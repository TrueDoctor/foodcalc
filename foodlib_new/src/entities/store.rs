use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Store {
    pub id: i32,
    pub name: String,
}

// Using composition to model relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreIngredientSummary {
    pub store_id: i32,
    pub store_name: String,
    pub ingredient_count: i64,
    pub average_price: Option<BigDecimal>,
}
