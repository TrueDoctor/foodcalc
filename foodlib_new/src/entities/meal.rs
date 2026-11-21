use crate::entities::types::ids::{EventId, IngredientId, MealId, PlaceId, RecipeId};

use crate::entities::types::amount::Amount;
use serde::{Deserialize, Serialize};
use time::{macros::time, OffsetDateTime};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meal {
    pub meal_id: MealId,
    pub event_id: EventId,
    pub recipe_id: RecipeId,
    pub name: String,
    pub place_id: PlaceId,
    pub place: String,
    #[serde(with = "time::serde::rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub end_time: OffsetDateTime,
    pub weight: Amount,
    pub energy: Amount,
    pub servings: i32,
    pub comment: Option<String>,
    /// Whether the meal's recipe has been marked "weighed in" (amounts verified
    /// accurate to the gram). Surfaced as a warning icon in the event detail tab.
    pub weighed_in: bool,
}

/// A meal enriched with everything the event planner view shows: the scaled
/// total weight, the recipe's energy density (the scaling base), and the
/// meal's ingredient cost. One query instead of the meal + price + density
/// round trips the old HTMX UI needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerMeal {
    pub meal_id: MealId,
    pub event_id: EventId,
    pub recipe_id: RecipeId,
    pub name: String,
    pub weighed_in: bool,
    pub place_id: PlaceId,
    pub place: String,
    #[serde(with = "time::serde::rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub end_time: OffsetDateTime,
    pub comment: Option<String>,
    pub servings: i32,
    /// Planned energy per portion (kJ) — the scaling input.
    pub energy_per_serving: Amount,
    /// Total cooked weight (kg) implied by `energy_per_serving × servings`.
    pub weight: Amount,
    /// Recipe energy density in kJ/g; `None` if the recipe has no resolvable
    /// ingredients (nothing to scale against).
    pub energy_kj_per_g: Option<Amount>,
    /// Total ingredient cost under the event's source overrides.
    pub price: Amount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealIngredient {
    pub event_id: EventId,
    pub recipe_id: RecipeId,
    pub ingredient_id: IngredientId,
    pub ingredient: String,
    pub weight: Amount,
    pub energy: Amount,
    pub price: Amount,
    pub servings: i32,
    pub meal_id: MealId,
    pub subrecipe_hierarchy: Option<String>,
}

impl Default for Meal {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        let start_time = now.replace_time(time!(12:00));
        Self {
            event_id: EventId::default(),
            meal_id: MealId(-1),
            recipe_id: RecipeId::default(),
            name: "New Meal".to_string(),
            comment: None,
            place_id: PlaceId::default(),
            place: Default::default(),
            start_time,
            end_time: start_time,
            weight: Default::default(),
            energy: Amount::from(2400),
            servings: 1,
            weighed_in: false,
        }
    }
}
