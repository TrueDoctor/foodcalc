use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post, MethodRouter},
    Json,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{error::BoxDynError, postgres::types::PgMoney, types::time::PrimitiveDateTime};

use crate::db::{Event, EventRecipeIngredient, FoodBase, Meal};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]

struct SerializeableEvent {
    pub event_id: i32,
    pub event_name: String,
    pub comment: Option<String>,
    pub budget: Option<bigdecimal::BigDecimal>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct SerializeableMeal {
    pub event_id: i32,
    pub recipe_id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub place_id: i32,
    pub place: String,
    pub start_time: String,
    pub end_time: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: BigDecimal,
    pub servings: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct MealIngredient {
    pub ingredient_id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub weight: BigDecimal,
    pub price: BigDecimal,
}

impl TryInto<EventRecipeIngredient> for MealIngredient {
    fn try_into(self) -> Result<EventRecipeIngredient, Self::Error> {
        Ok(EventRecipeIngredient {
            ingredient_id: self.ingredient_id,
            name: self.name,
            energy: self.energy,
            weight: self.weight,
            price: PgMoney::from_bigdecimal(self.price, 2)?,
        })
    }

    type Error = BoxDynError;
}

impl From<EventRecipeIngredient> for MealIngredient {
    fn from(ingredient: EventRecipeIngredient) -> Self {
        Self {
            ingredient_id: ingredient.ingredient_id,
            name: ingredient.name,
            energy: ingredient.energy,
            weight: ingredient.weight,
            price: ingredient.price.to_bigdecimal(2),
        }
    }
}

impl TryInto<Event> for SerializeableEvent {
    fn try_into(self) -> Result<Event, Self::Error> {
        let budget = match self.budget {
            Some(budget) => Some(PgMoney::from_bigdecimal(budget, 2)?),
            None => None,
        };
        Ok(Event {
            event_id: self.event_id,
            event_name: self.event_name,
            comment: self.comment,
            budget: budget,
        })
    }

    type Error = BoxDynError;
}

impl From<Event> for SerializeableEvent {
    fn from(event: Event) -> Self {
        Self {
            event_id: event.event_id,
            event_name: event.event_name,
            comment: event.comment,
            budget: event.budget.map(|budget| budget.to_bigdecimal(2)),
        }
    }
}

impl TryInto<Meal> for SerializeableMeal {
    fn try_into(self) -> Result<Meal, Self::Error> {
        Ok(Meal {
            event_id: self.event_id,
            recipe_id: self.recipe_id,
            name: self.name,
            comment: self.comment,
            place_id: self.place_id,
            place: self.place,
            start_time: PrimitiveDateTime::parse(self.start_time, "%F %T")?,
            end_time: PrimitiveDateTime::parse(self.end_time, "%F %T")?,
            weight: self.weight,
            energy: self.energy,
            price: PgMoney::from_bigdecimal(self.price, 2)?,
            servings: self.servings,
        })
    }

    type Error = BoxDynError;
}

impl From<Meal> for SerializeableMeal {
    fn from(meal: Meal) -> Self {
        Self {
            event_id: meal.event_id,
            recipe_id: meal.recipe_id,
            name: meal.name,
            comment: meal.comment,
            place_id: meal.place_id,
            place: meal.place,
            start_time: meal.start_time.format("%F %T"),
            end_time: meal.end_time.format("%F %T"),
            weight: meal.weight,
            energy: meal.energy,
            price: meal.price.to_bigdecimal(2),
            servings: meal.servings,
        }
    }
}

pub fn create_empty(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    post(|| async move {
        let result = db.add_empty_event().await;
        match result {
            Ok(_) => StatusCode::CREATED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })
}

pub fn update_event(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    post(|Json(event): Json<SerializeableEvent>| async move {
        let event = match event.try_into() {
            Ok(event) => event,
            _ => return StatusCode::BAD_REQUEST,
        };
        let result = db.update_event(&event).await;
        match result {
            Ok(_) => StatusCode::CREATED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })
}

pub fn update_meals(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    post(
        |Path(event_id): Path<i32>, Json(meals): Json<Vec<SerializeableMeal>>| async move {
            let meals = meals
                .into_iter()
                .map(|meal| meal.try_into())
                .collect::<Result<Vec<Meal>, _>>();
            match meals {
                Ok(meals) => {
                    let result = db.update_event_meals(event_id, meals.into_iter()).await;
                    match result {
                        Ok(_) => StatusCode::CREATED,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    }
                }
                _ => return StatusCode::BAD_REQUEST,
            }
        },
    )
}

pub fn get_event_cost(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    get(|Path(event_id): Path<i32>| async move {
        let result = db.get_event_cost(event_id).await;
        match result {
            Ok(cost) => (StatusCode::OK, Json(cost.to_bigdecimal(2))),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(bigdecimal::BigDecimal::default()),
            ),
        }
    })
}

pub fn list(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    get(|| async move {
        let result = db.get_events().await;
        match result {
            Ok(events) => (
                StatusCode::OK,
                Json(
                    events
                        .into_iter()
                        .map(|event| SerializeableEvent::from(event))
                        .collect(),
                ),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn get_meals(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    get(|Path(event_id): Path<i32>| async move {
        let result = db.get_event_meals(event_id).await;
        match result {
            Ok(meals) => (
                StatusCode::OK,
                Json(
                    meals
                        .into_iter()
                        .map(|meal| SerializeableMeal::from(meal))
                        .collect(),
                ),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}


pub fn get_recipe_ingredients(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    #[derive(Clone, Default, Deserialize)]
    struct MealID {
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: String,
    }//*
    get(|Json(meal_id): Json<MealID>| async move {
        let start_time = PrimitiveDateTime::parse(meal_id.start_time, "%F %T");
        match start_time {
            Ok(start_time) => {
                let result = db
                    .get_event_recipe_ingredients(
                        meal_id.event_id,
                        meal_id.recipe_id,
                        meal_id.place_id,
                        start_time,
                    )
                    .await;
                match result {
                    Ok(ingredients) => (
                        StatusCode::OK,
                        Json(
                            ingredients
                                .into_iter()
                                .map(|meal_ingredient| MealIngredient::from(meal_ingredient))
                                .collect::<Vec<MealIngredient>>(),
                        ),
                    ),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
                }
            }
            _ => return (StatusCode::BAD_REQUEST, Json(vec![])),
        }
    })
}


pub fn update_single_meal(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();
    #[derive(Clone, Default, Deserialize)]
    struct MealUpdate {
        old_meal: Option<SerializeableMeal>,
        new_meal: Option<SerializeableMeal>,
    }
    post(
        |Json(meal_update): Json<MealUpdate> | async move {
            let old_meal = match meal_update.old_meal {
                Some(old_meal) => match old_meal.try_into() {
                    Ok(old_meal) => Some(old_meal),
                    _ => return StatusCode::BAD_REQUEST,
                },
                None => None,
            };
            let new_meal = match meal_update.new_meal {
                Some(new_meal) => match new_meal.try_into() {
                    Ok(new_meal) => Some(new_meal),
                    _ => return StatusCode::BAD_REQUEST,
                },
                None => None,
            };
            let result = db.update_single_meal(old_meal, new_meal).await;
            match result {
                Ok(_) => StatusCode::CREATED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        },
    )
}