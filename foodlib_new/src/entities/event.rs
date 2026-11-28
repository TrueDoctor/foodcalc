use bigdecimal::BigDecimal;
use crate::types::ids::{
    EventId, FoodPrepId, GroupId, IngredientId, IngredientSourceId, InventoryId, PlaceId, RecipeId, ShoppingTourId,
    StoreId,
};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// A physical location/place where meals are served
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: PlaceId,
    pub name: String,
    pub comment: Option<String>,
}

/// Represents an event (e.g., dinner, party) with associated meals and planning
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<BigDecimal>,
    pub group_id: GroupId,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: EventId(0),
            name: "Test Event".to_string(),
            comment: Some("Test comment".to_string()),
            budget: None,
            group_id: GroupId::invalid(),
        }
    }
}

/// Aggregated event row for list views: adds the planner-relevant totals
/// (meal count, ingredient cost, time span) to the plain event fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub id: EventId,
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<BigDecimal>,
    pub group_id: GroupId,
    pub meal_count: i64,
    pub total_cost: BigDecimal,
    #[serde(with = "time::serde::rfc3339::option")]
    pub first_meal: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_meal: Option<OffsetDateTime>,
}

/// A scheduled shopping trip for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShoppingTour {
    pub id: ShoppingTourId,
    pub event_id: EventId,
    #[serde(with = "time::serde::rfc3339")]
    pub tour_date: OffsetDateTime,
    pub store_id: StoreId,
    pub store_name: Option<String>,
}

/// Food preparation task scheduled for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodPrep {
    pub id: FoodPrepId,
    pub event_id: EventId,
    pub recipe_id: RecipeId,
    #[serde(with = "time::serde::rfc3339")]
    pub prep_date: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub use_from: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub use_until: OffsetDateTime,
}

/// Source override for ingredients in an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct SourceOverride {
    pub event_id: EventId,
    pub source_id: IngredientSourceId,
}

/// View of source overrides with additional information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceOverrideView {
    pub event_id: EventId,
    pub ingredient_id: IngredientId,
    pub source_id: IngredientSourceId,
    pub ingredient_name: String,
    pub store_id: StoreId,
    pub store_name: String,
}

/// Item in a shopping list for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ShoppingListItem {
    pub event_id: EventId,
    pub event_name: String,
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: Option<BigDecimal>,
    pub tour_id: Option<ShoppingTourId>,
    pub category: Option<String>,
}

/// A tour shopping-list item plus the per-recipe usage breakdown that makes it
/// up. Built from a single `shopping_tour_ingredients` query (recipe grain),
/// aggregated to ingredient totals so the shopping exports don't need a second
/// pass for the comment column. `recipe_usage` is `(recipe, kg)` ordered by
/// descending amount; `weight`/`price` are the tour totals for the ingredient.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShoppingListItemWithUsage {
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    pub weight: BigDecimal,
    pub price: Option<BigDecimal>,
    pub category: Option<String>,
    pub recipe_usage: Vec<(String, BigDecimal)>,
}

/// Custom type for ingredients without a shopping tour
#[derive(Debug, Clone)]
pub struct IngredientWithoutTour {
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    pub weight: Option<BigDecimal>,
    pub store_name: String,
}

/// Inventory associated with an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInventory {
    pub event_id: EventId,
    pub inventory_id: InventoryId,
}

