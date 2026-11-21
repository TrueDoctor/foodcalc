use crate::entities::types::ids::{
    EventId, FoodPrepId, GroupId, IngredientId, IngredientSourceId, InventoryId, PlaceId, RecipeId,
    StoreId, TourId,
};

use crate::entities::types::amount::Amount;
use crate::entities::types::money::Money;
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
    pub budget: Option<Money>,
    pub group_id: GroupId,
}

/// Aggregated event row for list views: adds the planner-relevant totals
/// (meal count, ingredient cost, time span) to the plain event fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub id: EventId,
    pub name: String,
    pub comment: Option<String>,
    pub budget: Option<Money>,
    pub group_id: GroupId,
    pub meal_count: i64,
    pub total_cost: Money,
    #[serde(with = "time::serde::rfc3339::option")]
    pub first_meal: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_meal: Option<OffsetDateTime>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: EventId(0),
            name: "Test Event".to_string(),
            comment: Some("Test comment".to_string()),
            budget: None,
            group_id: GroupId(-1),
        }
    }
}

/// A scheduled shopping trip for an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShoppingTour {
    pub id: TourId,
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

/// An event ingredient that has no shopping tour covering it yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngredientWithoutTour {
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    pub weight: Option<Amount>,
    pub store_name: String,
}

/// A manually pinned tour item: forces `ingredient_id` onto `tour_id`,
/// overriding the view-derived store/date-window assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TourItemPin {
    pub tour_id: TourId,
    pub ingredient_id: IngredientId,
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
    /// Total kg to buy. `None` when the ingredient's unit weight is unknown
    /// (no `weights` row for the recipe's unit) — the demand is then
    /// unresolvable and the UI shows a placeholder.
    pub weight: Option<Amount>,
    /// Energy in kJ; `None` whenever weight is `None`.
    pub energy: Option<Amount>,
    pub price: Option<Money>,
    pub tour_id: Option<TourId>,
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
    /// `None` when any contributing recipe row has an unknown weight.
    pub weight: Option<Amount>,
    pub price: Option<Money>,
    pub category: Option<String>,
    pub recipe_usage: Vec<(String, Option<Amount>)>,
}

/// Inventory associated with an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInventory {
    pub event_id: EventId,
    pub inventory_id: InventoryId,
}

/// A food prep task plus the prepared recipe's name (timeline feed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodPrepWithRecipe {
    pub id: FoodPrepId,
    pub event_id: EventId,
    pub recipe_id: RecipeId,
    #[serde(with = "time::serde::rfc3339")]
    pub prep_date: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub use_from: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub use_until: OffsetDateTime,
    pub recipe_name: String,
}

/// Per-ingredient coverage of the event's demand by its linked inventories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInventoryCoverage {
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    /// Total demand in kg (from the resolved event shopping list).
    pub demand_kg: Amount,
    /// Amount stocked across the event's linked inventories, in kg.
    pub covered_kg: Amount,
}
