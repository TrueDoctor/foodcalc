use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// A physical location/place where meals are served
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
}

/// Represents an event (e.g., dinner, party) with associated meals and planning
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<BigDecimal>,
    pub group_id: i32,
}

/// Aggregated event row for list views: adds the planner-relevant totals
/// (meal count, ingredient cost, time span) to the plain event fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<BigDecimal>,
    pub group_id: i32,
    pub meal_count: i64,
    pub total_cost: BigDecimal,
    #[serde(with = "time::serde::rfc3339::option")]
    pub first_meal: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_meal: Option<OffsetDateTime>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Test Event".to_string(),
            comment: Some("Test comment".to_string()),
            budget: None,
            group_id: -1,
        }
    }
}

/// A scheduled shopping trip for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShoppingTour {
    pub id: i32,
    pub event_id: i32,
    #[serde(with = "time::serde::rfc3339")]
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

/// An event ingredient that has no shopping tour covering it yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngredientWithoutTour {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub weight: Option<BigDecimal>,
    pub store_name: String,
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

/// A tour shopping-list item plus the per-recipe usage breakdown that makes it
/// up. Built from a single `shopping_tour_ingredients` query (recipe grain),
/// aggregated to ingredient totals so the shopping exports don't need a second
/// pass for the comment column. `recipe_usage` is `(recipe, kg)` ordered by
/// descending amount; `weight`/`price` are the tour totals for the ingredient.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShoppingListItemWithUsage {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub weight: BigDecimal,
    pub price: Option<BigDecimal>,
    pub category: Option<String>,
    pub recipe_usage: Vec<(String, BigDecimal)>,
}

/// Inventory associated with an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInventory {
    pub event_id: i32,
    pub inventory_id: i32,
}
