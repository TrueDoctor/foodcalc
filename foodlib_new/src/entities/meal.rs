use bigdecimal::BigDecimal;
use crate::types::ids::{EventId, IngredientId, MealId, PlaceId, RecipeId};

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
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub servings: i32,
    pub comment: Option<String>,
    /// Whether the meal's recipe has been marked "weighed in" (amounts verified
    /// accurate to the gram). Surfaced as a warning icon in the event detail tab.
    pub weighed_in: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealIngredient {
    pub event_id: EventId,
    pub recipe_id: RecipeId,
    pub ingredient_id: IngredientId,
    pub ingredient: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: BigDecimal,
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
            energy: BigDecimal::from(2400),
            servings: 1,
            weighed_in: false,
        }
    }
}
