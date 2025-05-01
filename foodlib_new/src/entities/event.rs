use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Represents an event (e.g., dinner, party) with associated meals and planning
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<BigDecimal>,
    pub owner_id: i64,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Test Event".to_string(),
            comment: Some("Test comment".to_string()),
            budget: None,
            owner_id: -1,
        }
    }
}

/// A scheduled shopping trip for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShoppingTour {
    pub id: i32,
    pub event_id: i32,
    pub tour_date: OffsetDateTime,
    pub store_id: i32,
    pub store_name: Option<String>,
}

/// Food preparation task scheduled for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodPrep {
    pub id: i32,
    pub event_id: i32,
    pub recipe_id: i32,
    pub prep_date: OffsetDateTime,
    pub use_from: Option<OffsetDateTime>,
    pub use_until: OffsetDateTime,
}

/// Source override for ingredients in an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct SourceOverride {
    pub event_id: i32,
    pub source_id: i32,
}

/// View of source overrides with additional information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceOverrideView {
    pub event_id: i32,
    pub ingredient_id: i32,
    pub source_id: i32,
    pub ingredient_name: String,
    pub store_id: i32,
    pub store_name: String,
}

/// Item in a shopping list for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShoppingListItem {
    pub event_id: i32,
    pub event_name: String,
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: Option<BigDecimal>,
    pub tour_id: Option<i32>,
    pub category: Option<String>,
}

/// Custom type for ingredients without a shopping tour
#[derive(Debug, Clone)]
pub struct IngredientWithoutTour {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub weight: Option<BigDecimal>,
    pub store_name: String,
}

/// Inventory associated with an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInventory {
    pub event_id: i32,
    pub inventory_id: i32,
}
