use crate::types::ids::StoreId;
use crate::types::money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Store {
    pub id: StoreId,
    pub name: String,
}

// Using composition to model relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreIngredientSummary {
    pub store_id: StoreId,
    pub store_name: String,
    pub ingredient_count: i64,
    pub average_price: Option<Money>,
}
