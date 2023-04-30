use sqlx::{postgres::types::PgMoney, types::time::PrimitiveDateTime};

use crate::db::{Event, FoodBase, Meal};

pub fn create_empty(db: &FoodBase) {}

pub fn update_event(db: &FoodBase, event: &Event) {}

pub fn update_meals(db: &FoodBase, event: &Event, meals: impl Iterator<Item = Meal>) {}

pub fn get_event_cost(db: &FoodBase, event_id: i32) {}

pub fn list(db: &FoodBase) {}

pub fn get_recipe_ingredients(
    db: &FoodBase,
    event_id: i32,
    recipe_id: i32,
    place_id: i32,
    start_time: PrimitiveDateTime,
) {
}

pub fn get_meals(db: &FoodBase, event_id: i32) {

}

