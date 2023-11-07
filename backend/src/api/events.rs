use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{error::BoxDynError, postgres::types::PgMoney};

use crate::MyAppState;
use foodlib::{Event, EventRecipeIngredient, Meal};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MealIngredient {
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

pub async fn create_empty(State(state): State<MyAppState>) -> impl axum::response::IntoResponse {
    let result = state.db_connection.add_empty_event().await;
    match result {
        Ok(event) => (StatusCode::CREATED, Json(event.event_id)),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(0)),
    }
}

pub async fn update_event(
    State(state): State<MyAppState>,
    Json(event): Json<Event>,
) -> impl axum::response::IntoResponse {
    let result = state.db_connection.update_event(&event).await;
    match result {
        Ok(_) => StatusCode::CREATED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn update_meals(
    State(state): State<MyAppState>,
    Path(event_id): Path<i32>,
    Json(meals): Json<Vec<Meal>>,
) -> impl axum::response::IntoResponse {
    let result = state
        .db_connection
        .update_event_meals(event_id, meals.into_iter())
        .await;
    match result {
        Ok(_) => StatusCode::CREATED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn get_event_cost(
    State(state): State<MyAppState>,
    Path(event_id): Path<i32>,
) -> impl axum::response::IntoResponse {
    let result = state.db_connection.get_event_cost(event_id).await;
    match result {
        Ok(cost) => (StatusCode::OK, Json(cost.to_bigdecimal(2))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(bigdecimal::BigDecimal::default()),
        ),
    }
}

pub async fn list(State(state): State<MyAppState>) -> impl axum::response::IntoResponse {
    let result = state.db_connection.get_events().await;
    println!("{:?}", result);
    match result {
        Ok(events) => (
            StatusCode::OK,
            Json(events.into_iter().map(Event::from).collect()),
        ),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

pub async fn get_meals(
    State(state): State<MyAppState>,
    Path(event_id): Path<i32>,
) -> impl axum::response::IntoResponse {
    let result = state.db_connection.get_event_meals(event_id).await;
    match result {
        Ok(meals) => (
            StatusCode::OK,
            Json(meals.into_iter().map(Meal::from).collect()),
        ),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[derive(Clone, Deserialize)]
pub struct MealID {
    event_id: i32,
    recipe_id: i32,
    place_id: i32,
    start_time: NaiveDateTime,
}

pub async fn get_recipe_ingredients(
    State(state): State<MyAppState>,
    Json(meal_id): Json<MealID>,
) -> impl axum::response::IntoResponse {
    let result = state
        .db_connection
        .get_event_recipe_ingredients(
            meal_id.event_id,
            meal_id.recipe_id,
            meal_id.place_id,
            meal_id.start_time,
        )
        .await;
    match result {
        Ok(ingredients) => (
            StatusCode::OK,
            Json(
                ingredients
                    .into_iter()
                    .map(MealIngredient::from)
                    .collect::<Vec<MealIngredient>>(),
            ),
        ),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    }
}

#[derive(Clone, Default, Deserialize)]
pub struct MealUpdate {
    old_meal: Option<Meal>,
    new_meal: Option<Meal>,
}

pub async fn update_single_meal(
    State(state): State<MyAppState>,
    Json(meal_update): Json<MealUpdate>,
) -> impl axum::response::IntoResponse {
    let result = state
        .db_connection
        .update_single_meal(meal_update.old_meal, meal_update.new_meal)
        .await;
    match result {
        Ok(_) => StatusCode::CREATED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
