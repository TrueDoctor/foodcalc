use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use std::str::FromStr;

use crate::{entities::ingredient::*, error::Error, ops::ingredients::IngredientOps};

#[sqlx::test]
async fn test_create_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let ingredient = Ingredient {
        id: -1, // Will be replaced by DB
        name: "Test Ingredient".to_string(),
        energy: BigDecimal::from_str("10.5").unwrap(),
        comment: Some("Test comment".to_string()),
    };

    let created = ops.create(ingredient.clone()).await.unwrap();
    assert_eq!(created.name, ingredient.name);
    assert_eq!(created.energy, ingredient.energy);
    assert_eq!(created.comment, ingredient.comment);
    assert!(created.id > 0);

    // Verify it exists in DB
    let fetched = ops.get(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    // Get existing ingredient from fixture
    let ingredient = ops.get(1).await.unwrap();
    assert_eq!(ingredient.name, "Pasta");
    assert_eq!(ingredient.energy, BigDecimal::from_str("15.7").unwrap());

    // Test getting non-existent ingredient
    let err = ops.get(9999).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_by_name(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let ingredient = ops.get_by_name("Pasta").await.unwrap();
    assert_eq!(ingredient.id, 1);
    assert_eq!(ingredient.energy, BigDecimal::from_str("15.7").unwrap());

    let err = ops.get_by_name("NonExistentIngredient").await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let mut ingredient = ops.get(1).await.unwrap();
    ingredient.name = "Updated Pasta".to_string();
    ingredient.energy = BigDecimal::from_str("16.0").unwrap();
    ingredient.comment = Some("Updated comment".to_string());

    let updated = ops.update(ingredient.clone()).await.unwrap();
    assert_eq!(updated, ingredient);

    // Verify changes persisted
    let fetched = ops.get(1).await.unwrap();
    assert_eq!(fetched, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_ingredient(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    // Create temporary ingredient to delete
    let ingredient = ops
        .create(Ingredient {
            id: -1,
            name: "Ingredient To Delete".to_string(),
            energy: BigDecimal::from(1),
            comment: None,
        })
        .await
        .unwrap();

    // Add some related records
    ops.add_source(IngredientSource {
        id: -1,
        ingredient_id: ingredient.id,
        store_id: 1,
        package_size: BigDecimal::from(1),
        unit_id: 0,
        price: BigDecimal::from(1),
        url: None,
        comment: None,
    })
    .await
    .unwrap();

    ops.add_weight(IngredientWeight {
        ingredient_id: ingredient.id,
        unit_id: 1,
        weight: BigDecimal::from_str("0.001").unwrap(),
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
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(ingredients, sorted);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_sources(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    // Test adding source
    let source = IngredientSource {
        id: -1,
        ingredient_id: 1,
        store_id: 1,
        package_size: BigDecimal::from(2),
        unit_id: 0,
        price: BigDecimal::from_str("5.99").unwrap(),
        url: Some("http://example.com".to_string()),
        comment: Some("Test source".to_string()),
    };

    let created = ops.add_source(source.clone()).await.unwrap();
    assert!(created.id > 0);
    assert_eq!(created.ingredient_id, source.ingredient_id);
    assert_eq!(created.price, source.price);

    // Test updating source
    let mut updated = created.clone();
    updated.price = BigDecimal::from_str("6.99").unwrap();
    let result = ops.update_source(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    // Test fetching sources
    let sources = ops.get_sources(1).await.unwrap();
    assert!(sources.contains(&result));

    // Test deleting source
    ops.delete_source(created.id).await.unwrap();
    let sources = ops.get_sources(1).await.unwrap();
    assert!(!sources.contains(&created));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_weights(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let weight = IngredientWeight {
        ingredient_id: 1,
        unit_id: 4, // pieces
        weight: BigDecimal::from_str("0.1").unwrap(),
    };

    // Test adding weight
    let created = ops.add_weight(weight.clone()).await.unwrap();
    assert_eq!(created, weight);

    // Test updating weight
    let mut updated = weight.clone();
    updated.weight = BigDecimal::from_str("0.15").unwrap();
    let result = ops.update_weight(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    // Test fetching weights
    let weights = ops.get_weights(1).await.unwrap();
    assert!(weights.contains(&updated));

    // Test deleting weight
    ops.delete_weight(1, 4).await.unwrap();
    let weights = ops.get_weights(1).await.unwrap();
    assert!(!weights.contains(&weight));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_categories(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let category = IngredientCategory {
        ingredient_source_id: 1,
        category: "Test Category".to_string(),
    };

    // Test setting category
    let created = ops.set_category(category.clone()).await.unwrap();
    assert_eq!(created, category);

    // Test getting category
    let fetched = ops.get_category(1).await.unwrap().unwrap();
    assert_eq!(fetched, category);

    // Test updating existing category
    let updated = IngredientCategory {
        ingredient_source_id: 1,
        category: "Updated Category".to_string(),
    };
    let result = ops.set_category(updated.clone()).await.unwrap();
    assert_eq!(result, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_ingredient_properties(pool: sqlx::PgPool) {
    let ops = IngredientOps::new(pool.into());

    let property = IngredientProperty {
        ingredient_id: 1,
        property_id: 1,
    };

    // Test adding property
    let created = ops.add_property(property.clone()).await.unwrap();
    assert_eq!(created, property);

    // Test fetching properties
    let properties = ops.get_properties(1).await.unwrap();
    assert!(properties.contains(&property));

    // Test deleting property
    ops.delete_property(1, 1).await.unwrap();
    let properties = ops.get_properties(1).await.unwrap();
    assert!(!properties.contains(&property));
}
