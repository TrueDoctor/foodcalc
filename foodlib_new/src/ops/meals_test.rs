// meals_test.rs

use std::str::FromStr;

use crate::{
    entities::meal::*,
    error::Error,
    entities::types::ids::{EventId, IngredientId, MealId, PlaceId, RecipeId},
    ops::meals::MealOps,
};

use crate::entities::types::amount::Amount;
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use std::sync::Arc;
use time::{macros::time, OffsetDateTime};

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_event_meals(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let meals = ops.get_event_meals(EventId(1)).await.unwrap();
    assert_eq!(meals.len(), 1);

    let meal = &meals[0];
    assert_eq!(meal.event_id, EventId(1));
    assert_eq!(meal.recipe_id, RecipeId(1));
    assert_eq!(meal.name, "Simple Pasta");
    assert_eq!(meal.servings, 4);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_meal(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let meal = ops.get_meal(MealId(1)).await.unwrap();
    assert_eq!(meal.event_id, EventId(1));
    assert_eq!(meal.recipe_id, RecipeId(1));
    assert_eq!(meal.name, "Simple Pasta");
    assert_eq!(meal.servings, 4);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_all_meals(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let meals = ops.get_all_meals().await.unwrap();
    assert_eq!(meals.len(), 2);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_meal(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let now = OffsetDateTime::now_utc();
    let start_time = now.replace_time(time!(12:00:00));
    let end_time = start_time + time::Duration::minutes(30);

    ops.update_meal(
        MealId(1),
        RecipeId(1),
        PlaceId(1),
        start_time,
        end_time,
        Amount::from(1800),
        6,
        Some("Updated meal".to_string()),
    )
    .await
    .unwrap();

    let updated_meal = ops.get_meal(MealId(1)).await.unwrap();
    assert_eq!(updated_meal.recipe_id, RecipeId(2));
    assert_eq!(updated_meal.place_id, PlaceId(1));
    assert_eq!(updated_meal.energy, Amount::from(1800));
    assert_eq!(updated_meal.servings, 6);
    assert_eq!(updated_meal.comment, Some("Updated meal".to_string()));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_add_meal(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let now = OffsetDateTime::now_utc();
    let start_time = now.replace_time(time!(12:30:00));
    let end_time = start_time + time::Duration::minutes(60);

    ops.add_meal(
        EventId(2),
        RecipeId(3),
        PlaceId(1),
        start_time,
        end_time,
        Amount::from(1500),
        8,
        Some("New meal".to_string()),
    )
    .await
    .unwrap();

    let mut meals = ops.get_event_meals(EventId(2)).await.unwrap();
    assert_eq!(meals.len(), 2);
    meals.sort_unstable_by_key(|x| MealId(-x.meal_id.get()));

    let new_meal = &meals[0];
    assert_eq!(new_meal.event_id, EventId(2));
    assert_eq!(new_meal.recipe_id, RecipeId(3));
    assert_eq!(new_meal.place_id, PlaceId(1));
    assert_eq!(new_meal.energy, Amount::from(1500));
    assert_eq!(new_meal.servings, 8);
    assert_eq!(new_meal.comment, Some("New meal".to_string()));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_remove_meal(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    ops.remove_meal(MealId(1)).await.unwrap();

    let err = ops.get_meal(MealId(1)).await.unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_remove_meal_by_reference(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    ops.remove_meal_by_reference(
        EventId(1),
        RecipeId(1),
        PlaceId(1),
        time::macros::datetime!(2024-12-04 18:00).assume_utc(),
    )
    .await
    .unwrap();

    let err = ops.get_meal(MealId(1)).await.unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_meal_ingredients(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let ingredients = ops.get_meal_ingredients(MealId(1)).await.unwrap();
    assert_eq!(dbg!(&ingredients).len(), 4);

    assert!(ingredients
        .iter()
        .any(|x: &MealIngredient| x.ingredient_id == IngredientId(1)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_event_meals(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let now = OffsetDateTime::now_utc();
    let start_time = now.replace_time(time!(12:00:00));
    let end_time = start_time + time::Duration::minutes(30);

    let meals = vec![Meal {
        meal_id: MealId(-1),
        event_id: EventId(1),
        recipe_id: RecipeId(2),
        name: "Updated Pasta".to_string(),
        place_id: PlaceId(1),
        place: "Main Kitchen".to_string(),
        start_time,
        end_time,
        weight: Default::default(),
        energy: Amount::from(3400),
        servings: 4,
        comment: Some("Updated meal".to_string()),
        weighed_in: false,
    }];

    ops.update_event_meals(EventId(1), meals.into_iter()).await.unwrap();

    let updated_meals = ops.get_event_meals(EventId(1)).await.unwrap();
    assert_eq!(updated_meals.len(), 1);

    let updated_meal = &updated_meals[0];
    assert_eq!(updated_meal.recipe_id, RecipeId(2));
    assert_eq!(updated_meal.place_id, PlaceId(1));
    assert_eq!(updated_meal.weight, Amount::from_str("1.02").unwrap());
    assert_eq!(updated_meal.energy, Amount::from(3400));
    let price = ops.get_meal_price(updated_meal.meal_id).await.unwrap();
    assert_eq!(price.round(3) * 1000, Amount::from(5809));
    assert_eq!(updated_meal.servings, 4);
    assert_eq!(updated_meal.comment, Some("Updated meal".to_string()));
}
