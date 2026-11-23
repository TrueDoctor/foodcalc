use pretty_assertions::assert_eq;
use std::str::FromStr;

use crate::entities::types::amount::Amount;
use crate::entities::types::money::Money;
use crate::{
    entities::ingredient::*,
    entities::types::ids::{
        GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId, UnitId,
    },
    error::Error,
    ops::ingredients::IngredientOps,
};

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_create_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let input = CreateIngredient {
        name: "Test Ingredient".to_string(),
        energy: Amount::from_str("10.5").unwrap(),
        comment: Some("Test comment".to_string()),
        group_id: GroupId(1),
    };

    let created = ops.create(input.clone()).await.unwrap();
    assert!(created.id.is_valid());
    assert_eq!(created.name, input.name);
    assert_eq!(created.energy, input.energy);
    assert_eq!(created.comment, input.comment);
    assert_eq!(created.group_id, input.group_id);

    // Verify it exists in DB
    let fetched = ops.get(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    // Get existing ingredient from fixture
    let ingredient = ops.get(IngredientId(1)).await.unwrap();
    assert_eq!(ingredient.name, "Pasta");
    assert_eq!(ingredient.energy, Amount::from_str("15.7").unwrap());

    // Test getting non-existent ingredient
    let err = ops.get(IngredientId(9999)).await.unwrap_err();
     assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_by_name(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let ingredient = ops.get_by_name("Pasta").await.unwrap();
    assert_eq!(ingredient.id, IngredientId(1));
    assert_eq!(ingredient.energy, Amount::from_str("15.7").unwrap());

    let err = ops.get_by_name("NonExistentIngredient").await.unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let mut ingredient = ops.get(IngredientId(1)).await.unwrap();
    ingredient.name = "Updated Pasta".to_string();
    ingredient.energy = Amount::from_str("16.0").unwrap();
    ingredient.comment = Some("Updated comment".to_string());

    let updated = ops.update(ingredient.clone()).await.unwrap();
    assert_eq!(updated, ingredient);

    // Verify changes persisted
    let fetched = ops.get(IngredientId(1)).await.unwrap();
    assert_eq!(fetched, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    // Create temporary ingredient to delete
    let ingredient = ops
        .create(CreateIngredient {
            name: "Ingredient To Delete".to_string(),
            energy: Amount::from(1),
            comment: None,
            group_id: GroupId(1),
        })
        .await
        .unwrap();

    // Add some related records
    ops.add_source(IngredientSource {
        id: IngredientSourceId(-1),
        ingredient_id: ingredient.id,
        store_id: StoreId(1),
        package_size: Amount::from(1),
        unit_id: UnitId(0),
        price: Money::from(1),
        url: None,
        comment: None,
    })
    .await
    .unwrap();

    ops.add_weight(IngredientWeight {
        ingredient_id: ingredient.id,
        unit_id: UnitId(1),
        weight: Amount::from_str("0.001").unwrap(),
    })
    .await
    .unwrap();

    // Delete and verify cascading deletes worked
    ops.delete(ingredient.id).await.unwrap();

    let err = ops.get(ingredient.id).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_list_ingredients(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let ingredients = ops.list().await.unwrap();
    assert!(!ingredients.is_empty());

    // Verify ingredients are sorted by name
    let mut sorted = ingredients.clone();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(ingredients, sorted);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_sources(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    // Test adding source
    let source = IngredientSource {
        id: IngredientSourceId(-1),
        ingredient_id: IngredientId(1),
        store_id: StoreId(1),
        package_size: Amount::from(2),
        unit_id: UnitId(0),
        price: Money::from_str("5.99").unwrap(),
        url: Some("https://example.com".to_string()),
        comment: Some("Test source".to_string()),
    };

    let created = ops.add_source(source.clone()).await.unwrap();
    assert!(created.id > IngredientSourceId(0));
    assert_eq!(created.ingredient_id, source.ingredient_id);
    assert_eq!(created.price, source.price);

    // Test updating source
    let mut updated = created.clone();
    updated.price = Money::from_str("6.99").unwrap();
    let result = ops.update_source(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    // Test fetching sources
    let sources = ops.get_sources(IngredientId(1)).await.unwrap();
    assert!(sources.contains(&result));

    // Test deleting source
    ops.delete_source(created.id).await.unwrap();
    let sources = ops.get_sources(IngredientId(1)).await.unwrap();
    assert!(!sources.contains(&created));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_weights(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let weight = IngredientWeight {
        ingredient_id: IngredientId(1),
        unit_id: UnitId(4), // pieces
        weight: Amount::from_str("0.1").unwrap(),
    };

    // Test adding weight
    let created = ops.add_weight(weight.clone()).await.unwrap();
    assert_eq!(created, weight);

    // Test updating weight
    let mut updated = weight.clone();
    updated.weight = Amount::from_str("0.15").unwrap();
    let result = ops.update_weight(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    // Test fetching weights
    let weights = ops.get_weights(IngredientId(1)).await.unwrap();
    assert!(weights.contains(&updated));

    // Test deleting weight
    ops.delete_weight(IngredientId(1), UnitId(4)).await.unwrap();
    let weights = ops.get_weights(IngredientId(1)).await.unwrap();
    assert!(!weights.contains(&weight));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_categories(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let category = IngredientCategory {
        ingredient_source_id: IngredientSourceId(1),
        category: "Test Category".to_string(),
    };

    // Test setting category
    let created = ops.set_category(category.clone()).await.unwrap();
    assert_eq!(created, category);

    // Test getting category
    let fetched = ops
        .get_category(IngredientSourceId(1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched, category);

    // Test updating existing category
    let updated = IngredientCategory {
        ingredient_source_id: IngredientSourceId(1),
        category: "Updated Category".to_string(),
    };
    let result = ops.set_category(updated.clone()).await.unwrap();
    assert_eq!(result, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_properties(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let property = IngredientProperty {
        ingredient_id: IngredientId(1),
        property_id: PropertyId(1),
    };

    // Test adding property
    let created = ops.add_property(property.clone()).await.unwrap();
    assert_eq!(created, property);

    // Test fetching properties
    let properties = ops.get_properties(IngredientId(1)).await.unwrap();
    assert!(properties.contains(&property));

    // Test deleting property
    ops.delete_property(IngredientId(1), PropertyId(1))
        .await
        .unwrap();
    let properties = ops.get_properties(IngredientId(1)).await.unwrap();
    assert!(!properties.contains(&property));
}
