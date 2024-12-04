use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use time::{macros::time, OffsetDateTime};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meal {
    pub meal_id: i32,
    pub event_id: i32,
    pub recipe_id: i32,
    pub name: String,
    pub place_id: i32,
    pub place: String,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: BigDecimal,
    pub servings: i32,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealIngredient {
    pub event_id: i32,
    pub recipe_id: i32,
    pub ingredient_id: i32,
    pub ingredient: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: BigDecimal,
    pub servings: i32,
    pub meal_id: i32,
    pub subrecipe_hierarchy: Option<String>,
}

impl Default for Meal {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        let start_time = now.replace_time(time!(12:00));
        Self {
            event_id: Default::default(),
            meal_id: -1,
            recipe_id: Default::default(),
            name: "New Meal".to_string(),
            comment: None,
            place_id: Default::default(),
            place: Default::default(),
            start_time,
            end_time: start_time,
            weight: Default::default(),
            energy: BigDecimal::from(2400),
            price: BigDecimal::from(0),
            servings: 1,
        }
    }
}
