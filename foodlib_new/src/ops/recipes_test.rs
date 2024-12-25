use crate::entities::recipe::*;
use crate::ops::recipes::*;
use bigdecimal::BigDecimal;
use core::time;
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;

async fn create_test_recipe(ops: &RecipeOps) -> Recipe {
    ops.create(Recipe {
        id: -1,
        name: "Test Recipe".to_string(),
        comment: Some("Test Comment".to_string()),
    })
    .await
    .expect("Failed to create test recipe")
}

#[sqlx::test]
async fn test_create_recipe(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    let recipe = create_test_recipe(&ops).await;
    assert_eq!(recipe.name, "Test Recipe");
    assert_eq!(recipe.comment, Some("Test Comment".to_string()));
    assert!(recipe.id > 0);
}

#[sqlx::test]
async fn test_get_recipe(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    let created = create_test_recipe(&ops).await;
    let fetched = ops.get(created.id).await.expect("Failed to get recipe");

    assert_eq!(created.id, fetched.id);
    assert_eq!(created.name, fetched.name);
    assert_eq!(created.comment, fetched.comment);
}

#[sqlx::test]
async fn test_update_recipe(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    let mut recipe = create_test_recipe(&ops).await;
    recipe.name = "Updated Recipe".to_string();

    let updated = ops
        .update(recipe.clone())
        .await
        .expect("Failed to update recipe");
    assert_eq!(updated.name, "Updated Recipe");
    assert_eq!(updated.id, recipe.id);
}

#[sqlx::test]
async fn test_delete_recipe(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    let recipe = create_test_recipe(&ops).await;
    ops.delete(recipe.id)
        .await
        .expect("Failed to delete recipe");

    let result = ops.get(recipe.id).await;
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_recipe_steps(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    let recipe = create_test_recipe(&ops).await;

    // Add step
    let step = RecipeStep {
        id: -1,
        recipe_id: recipe.id,
        order: 1.0,
        name: "Test Step".to_string(),
        description: "Test Description".to_string(),
        fixed_duration: time::Duration::from_secs(600).try_into().unwrap(), // 10 minutes
        duration_per_kg: time::Duration::from_secs(300).try_into().unwrap(), // 5 minutes per kg
    };

    let created_step = ops
        .add_step(step.clone())
        .await
        .expect("Failed to add step");

    // Get steps
    let steps = ops.get_steps(recipe.id).await.expect("Failed to get steps");

    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].name, step.name);

    // Update step
    let mut updated_step = created_step;
    updated_step.name = "Updated Step".to_string();

    ops.update_step(updated_step.clone())
        .await
        .expect("Failed to update step");

    // Delete step
    ops.delete_step(recipe.id, updated_step.id)
        .await
        .expect("Failed to delete step");

    let steps = ops.get_steps(recipe.id).await.expect("Failed to get steps");
    assert!(steps.is_empty());
}

#[sqlx::test]
async fn test_meta_recipes(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    let parent_recipe = create_test_recipe(&ops).await;
    let child_recipe = ops
        .create(Recipe {
            id: -1,
            name: "Child Recipe".to_string(),
            comment: None,
        })
        .await
        .expect("Failed to create child recipe");

    // Add meta recipe
    let meta = RecipeMetaIngredient {
        parent_id: parent_recipe.id,
        child_id: child_recipe.id,
        weight: BigDecimal::from_str("2.0").unwrap(),
    };

    ops.add_meta_ingredient(meta.clone())
        .await
        .expect("Failed to add meta recipe");

    // Update meta recipe
    let updated_meta = RecipeMetaIngredient {
        weight: BigDecimal::from_str("3.0").unwrap(),
        ..meta.clone()
    };

    ops.update_meta_ingredient(updated_meta)
        .await
        .expect("Failed to update meta recipe");

    // Delete meta recipe
    ops.delete_meta_ingredient(parent_recipe.id, child_recipe.id)
        .await
        .expect("Failed to delete meta recipe");
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_recipe_ingredients(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    // Simple Pasta (ID: 1) already has an ingredient and meta-recipe
    let ingredients = ops
        .get_ingredients(1)
        .await
        .expect("Failed to get ingredients");

    assert_eq!(ingredients.len(), 1);
    assert_eq!(ingredients[0].ingredient_id, 1); // Pasta
    assert_eq!(ingredients[0].amount, BigDecimal::from(500)); // 500g
    assert_eq!(ingredients[0].unit_id, 1); // grams

    // Add new ingredient
    let new_ingredient = RecipeIngredient {
        recipe_id: 1,
        ingredient_id: 4, // Olive Oil
        amount: BigDecimal::from_str("2.0").unwrap(),
        unit_id: 3, // mL
    };

    ops.add_ingredient(new_ingredient.clone())
        .await
        .expect("Failed to add ingredient");

    // Verify ingredients after addition
    let ingredients = ops
        .get_ingredients(1)
        .await
        .expect("Failed to get ingredients");
    assert_eq!(ingredients.len(), 2);

    // Update ingredient
    let updated_ingredient = RecipeIngredient {
        amount: BigDecimal::from_str("3.0").unwrap(),
        ..new_ingredient.clone()
    };

    ops.update_ingredient(updated_ingredient)
        .await
        .expect("Failed to update ingredient");

    // Delete ingredient
    ops.delete_ingredient(1, 4)
        .await
        .expect("Failed to delete ingredient");

    let ingredients = ops
        .get_ingredients(1)
        .await
        .expect("Failed to get ingredients");
    assert_eq!(ingredients.len(), 1);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_search_by_ingredients(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    // Search for recipes containing flour (id: 5) and eggs (id: 7)
    // Should find Basic Cake (id: 2)
    let results = ops
        .search_by_ingredients(&[5, 7], Some(10))
        .await
        .expect("Failed to search recipes");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, 2); // Basic Cake
    assert_eq!(results[0].name, "Basic Cake");

    // Search for recipes containing tomatoes (id: 2)
    // Should find Tomato Sauce (id: 3)
    let results = ops
        .search_by_ingredients(&[2], Some(10))
        .await
        .expect("Failed to search recipes");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, 3); // Tomato Sauce

    // Search for pasta (id: 1) and tomatoes (id: 2)
    // Should find both Simple Pasta (id: 1) and Tomato Sauce (id: 3)
    let results = ops
        .search_by_ingredients(&[1, 2], Some(10))
        .await
        .expect("Failed to search recipes");

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.id == 1)); // Simple Pasta
    assert!(results.iter().any(|r| r.id == 3)); // Tomato Sauce
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_recipe_entries(pool: PgPool) {
    let ops = RecipeOps::new(Arc::new(pool));

    // Update Simple Pasta (ID: 1)
    let regular_ingredients = vec![
        RecipeIngredient {
            recipe_id: 1,
            ingredient_id: 1,              // Pasta
            amount: BigDecimal::from(400), // 400g
            unit_id: 1,                    // grams
        },
        RecipeIngredient {
            recipe_id: 1,
            ingredient_id: 4,             // Olive Oil
            amount: BigDecimal::from(30), // 30mL
            unit_id: 3,                   // milliliters
        },
    ];

    let meta_ingredients = vec![RecipeMetaIngredient {
        parent_id: 1,
        child_id: 3,                                  // Tomato Sauce
        weight: BigDecimal::from_str("0.4").unwrap(), // 400g
    }];

    // Update recipe entries
    ops.update_recipe_entries(1, regular_ingredients.clone(), meta_ingredients.clone())
        .await
        .expect("Failed to update recipe entries");

    // Verify regular ingredients
    let ingredients = ops
        .get_ingredients(1)
        .await
        .expect("Failed to get ingredients");
    assert_eq!(ingredients.len(), 2);

    // Sort ingredients by ID to make comparison reliable
    let mut ingredients: Vec<_> = ingredients
        .into_iter()
        .map(|i| (i.ingredient_id, i))
        .collect();
    ingredients.sort_by_key(|i| i.0);
    let ingredients: Vec<_> = ingredients.into_iter().map(|i| i.1).collect();

    // Check pasta ingredient
    assert_eq!(ingredients[0].ingredient_id, 1);
    assert_eq!(ingredients[0].amount, BigDecimal::from(400));
    assert_eq!(ingredients[0].unit_id, 1);

    // Check olive oil ingredient
    assert_eq!(ingredients[1].ingredient_id, 4);
    assert_eq!(ingredients[1].amount, BigDecimal::from(30));
    assert_eq!(ingredients[1].unit_id, 3);

    // Verify meta recipes (direct relationships only)
    let meta_recipes = ops
        .get_meta_ingredients(1)
        .await
        .expect("Failed to get meta ingredients");

    assert_eq!(meta_recipes.len(), 1);
    assert_eq!(meta_recipes[0].parent_id, 1);
    assert_eq!(meta_recipes[0].child_id, 3); // Tomato Sauce
    assert_eq!(meta_recipes[0].weight, BigDecimal::from_str("0.4").unwrap());

    // Verify meta recipe
    let resolved = ops
        .get_resolved_recipe(1)
        .await
        .expect("Failed to get resolved recipe");

    let resolved_recipes: Vec<_> = resolved
        .iter()
        .filter(|r| r.subrecipe_id.is_some())
        .collect();

    assert_eq!(resolved_recipes.len(), 5);
}
