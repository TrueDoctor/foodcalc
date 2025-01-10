// meals_test.rs

use crate::{entities::meal::*, error::Error, ops::meals::MealOps};
use bigdecimal::{BigDecimal, FromPrimitive};
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use std::sync::Arc;
use time::{macros::time, OffsetDateTime};

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_event_meals(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let meals = ops.get_event_meals(1).await.unwrap();
    assert_eq!(meals.len(), 1);

    let meal = &meals[0];
    assert_eq!(meal.event_id, 1);
    assert_eq!(meal.recipe_id, 1);
    assert_eq!(meal.name, "Simple Pasta");
    assert_eq!(meal.servings, 4);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_meal(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let meal = ops.get_meal(1).await.unwrap();
    assert_eq!(meal.event_id, 1);
    assert_eq!(meal.recipe_id, 1);
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
        1,
        2,
        1,
        start_time,
        end_time,
        BigDecimal::from(1800),
        6,
        Some("Updated meal".to_string()),
    )
    .await
    .unwrap();

    let updated_meal = ops.get_meal(1).await.unwrap();
    assert_eq!(updated_meal.recipe_id, 2);
    assert_eq!(updated_meal.place_id, 1);
    assert_eq!(updated_meal.energy, BigDecimal::from(1800));
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
        2,
        3,
        1,
        start_time,
        end_time,
        BigDecimal::from(1500),
        8,
        Some("New meal".to_string()),
    )
    .await
    .unwrap();

    let mut meals = ops.get_event_meals(2).await.unwrap();
    assert_eq!(meals.len(), 2);
    meals.sort_unstable_by_key(|x| -x.meal_id);

    let new_meal = &meals[0];
    assert_eq!(new_meal.event_id, 2);
    assert_eq!(new_meal.recipe_id, 3);
    assert_eq!(new_meal.place_id, 1);
    assert_eq!(new_meal.energy, BigDecimal::from(1500));
    assert_eq!(new_meal.servings, 8);
    assert_eq!(new_meal.comment, Some("New meal".to_string()));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_remove_meal(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    ops.remove_meal(1).await.unwrap();

    let err = ops.get_meal(1).await.unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_remove_meal_by_reference(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    ops.remove_meal_by_reference(
        1,
        1,
        1,
        time::macros::datetime!(2024-12-04 18:00).assume_utc(),
    )
    .await
    .unwrap();

    let err = ops.get_meal(1).await.unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_meal_ingredients(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let ingredients = ops.get_meal_ingredients(1).await.unwrap();
    assert_eq!(dbg!(&ingredients).len(), 4);

    assert!(ingredients
        .iter()
        .any(|x: &MealIngredient| x.ingredient_id == 1));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_event_meals(pool: PgPool) {
    let ops = MealOps::new(Arc::new(pool));

    let now = OffsetDateTime::now_utc();
    let start_time = now.replace_time(time!(12:00:00));
    let end_time = start_time + time::Duration::minutes(30);

    let meals = vec![Meal {
        meal_id: -1,
        event_id: 1,
        recipe_id: 2,
        name: "Updated Pasta".to_string(),
        place_id: 1,
        place: "Main Kitchen".to_string(),
        start_time,
        end_time,
        weight: Default::default(),
        energy: BigDecimal::from(3400),
        price: Default::default(),
        servings: 4,
        comment: Some("Updated meal".to_string()),
    }];

    ops.update_event_meals(1, meals.into_iter()).await.unwrap();

    let updated_meals = ops.get_event_meals(1).await.unwrap();
    assert_eq!(updated_meals.len(), 1);

    let updated_meal = &updated_meals[0];
    assert_eq!(updated_meal.recipe_id, 2);
    assert_eq!(updated_meal.place_id, 1);
    assert_eq!(updated_meal.weight, BigDecimal::new(83.into(), 2));
    assert_eq!(updated_meal.energy, BigDecimal::from(3400));
    assert_eq!(updated_meal.price.round(3) * 1000, BigDecimal::from(5809));
    assert_eq!(updated_meal.servings, 4);
    assert_eq!(updated_meal.comment, Some("Updated meal".to_string()));
}
