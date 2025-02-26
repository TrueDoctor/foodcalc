use crate::entities::recipe::{Recipe, RecipeIngredient, RecipeStep};
use crate::error::{Error, Result};
use crate::recipe::{RecipeMetaIngredient, RecipeStats, ResolvedRecipe};
use sqlx::PgPool;
use std::sync::Arc;

/// Operations for managing recipes and their components (ingredients, steps, meta-recipes)
///
/// This module provides CRUD operations and specialized queries for:
/// - Basic recipe management
/// - Recipe ingredients with units
/// - Recipe preparation steps with timing
/// - Meta-recipes (recipes that use other recipes)
///
/// All operations use transactions where appropriate to maintain data consistency.
pub struct RecipeOps {
    pool: Arc<PgPool>,
}

impl RecipeOps {
    /// Creates a new RecipeOps instance with the given database pool
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Creates a new recipe
    ///
    /// # Errors
    /// - If database operation fails
    /// - If recipe name already exists
    pub async fn create(&self, recipe: Recipe) -> Result<Recipe> {
        let record = sqlx::query_as!(
            Recipe,
            r#"
            INSERT INTO recipes (name, comment, owner_id)
            VALUES ($1, $2, $3)
            RETURNING recipe_id as id, name, comment, owner_id
            "#,
            recipe.name,
            recipe.comment,
            recipe.owner_id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get(&self, id: i32) -> Result<Recipe> {
        let record = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as id, name, comment, owner_id 
            FROM recipes 
            WHERE recipe_id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        })?;

        Ok(record)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Recipe> {
        let record = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as id, name, comment, owner_id 
            FROM recipes 
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: name.to_string(),
        })?;

        Ok(record)
    }

    pub async fn update(&self, recipe: Recipe) -> Result<Recipe> {
        let record = sqlx::query_as!(
            Recipe,
            r#"
            UPDATE recipes 
            SET name = $1, comment = $2
            WHERE recipe_id = $3
            RETURNING recipe_id as id, name, comment, owner_id
            "#,
            recipe.name,
            recipe.comment,
            recipe.id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: recipe.id.to_string(),
        })?;

        Ok(record)
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(r#"DELETE FROM recipe_ingredients WHERE recipe_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!(r#"DELETE FROM steps WHERE recipe_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!(
            r#"DELETE FROM meta_recipes WHERE parent_id = $1 OR child_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(r#"DELETE FROM event_meals WHERE recipe_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query!(r#"DELETE FROM recipes WHERE recipe_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "Recipe",
                id: id.to_string(),
            });
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Recipe>> {
        let records = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as id, name, comment, owner_id 
            FROM recipes 
            ORDER BY recipe_id
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    // Recipe Ingredients
    pub async fn add_ingredient(&self, ingredient: RecipeIngredient) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id)
            VALUES ($1, $2, $3, $4)
            "#,
            ingredient.recipe_id,
            ingredient.ingredient_id,
            ingredient.amount,
            ingredient.unit_id,
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_ingredient(
        &self,
        ingredient: RecipeIngredient,
    ) -> Result<RecipeIngredient> {
        let result = sqlx::query!(
            r#"
            UPDATE recipe_ingredients 
            SET amount = $3, unit_id = $4
            WHERE recipe_id = $1 AND ingredient_id = $2
            "#,
            ingredient.recipe_id,
            ingredient.ingredient_id,
            ingredient.amount,
            ingredient.unit_id,
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "RecipeIngredient",
                id: format!(
                    "recipe:{} ingredient:{}",
                    ingredient.recipe_id, ingredient.ingredient_id
                ),
            });
        }

        Ok(ingredient)
    }

    pub async fn delete_ingredient(&self, recipe_id: i32, ingredient_id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM recipe_ingredients 
            WHERE recipe_id = $1 AND ingredient_id = $2
            "#,
            recipe_id,
            ingredient_id,
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "RecipeIngredient",
                id: format!("recipe:{} ingredient:{}", recipe_id, ingredient_id),
            });
        }

        Ok(())
    }

    pub async fn get_ingredients(&self, recipe_id: i32) -> Result<Vec<RecipeIngredient>> {
        let records = sqlx::query_as!(
            RecipeIngredient,
            r#"
            SELECT recipe_id, ingredient_id, amount, unit_id, name
            FROM recipe_ingredients JOIN ingredients USING(ingredient_id)
            WHERE recipe_id = $1
            ORDER BY ingredient_id
            "#,
            recipe_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    // Recipe Steps
    pub async fn add_step(&self, step: RecipeStep) -> Result<RecipeStep> {
        let record = sqlx::query_as!(
            RecipeStep,
            r#"
            INSERT INTO steps (recipe_id, step_order, step_name, step_description, 
                             fixed_duration, duration_per_kg)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                step_id as id, recipe_id, step_order as "order", 
                step_name as name, step_description as description,
                fixed_duration, duration_per_kg
            "#,
            step.recipe_id,
            step.order,
            step.name,
            step.description,
            step.fixed_duration,
            step.duration_per_kg,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    pub async fn update_step(&self, step: RecipeStep) -> Result<RecipeStep> {
        let record = sqlx::query_as!(
            RecipeStep,
            r#"
            UPDATE steps 
            SET step_order = $2, step_name = $3, step_description = $4,
                fixed_duration = $5,
                duration_per_kg = $6
            WHERE step_id = $1
            RETURNING 
                step_id as id, recipe_id, step_order as "order",
                step_name as name, step_description as description,
                fixed_duration, duration_per_kg
            "#,
            step.id,
            step.order,
            step.name,
            step.description,
            step.fixed_duration,
            step.duration_per_kg,
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "RecipeStep",
            id: step.id.to_string(),
        })?;

        Ok(record)
    }

    pub async fn delete_step(&self, recipe_id: i32, step_id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM steps 
            WHERE recipe_id = $1 AND step_id = $2
            "#,
            recipe_id,
            step_id,
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "RecipeStep",
                id: format!("recipe:{} step:{}", recipe_id, step_id),
            });
        }

        Ok(())
    }

    pub async fn get_steps(&self, recipe_id: i32) -> Result<Vec<RecipeStep>> {
        let records = sqlx::query_as!(
            RecipeStep,
            r#"
            SELECT 
                step_id as id, recipe_id, step_order as "order",
                step_name as name, step_description as description,
                fixed_duration, duration_per_kg
            FROM steps
            WHERE recipe_id = $1
            ORDER BY step_order
            "#,
            recipe_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    // Meta Recipes
    /// Retrieves direct meta-recipe relationships for a recipe.
    ///
    /// Unlike `get_resolved_recipe` which returns all ingredients recursively,
    /// this function only returns the immediate sub-recipes (direct children)
    /// of the given recipe. Each meta-recipe entry represents a direct
    /// relationship where one recipe is used as an ingredient in another.
    ///
    /// # Arguments
    /// * `recipe_id` - ID of the parent recipe
    ///
    /// # Returns
    /// List of meta-recipe relationships, containing:
    /// - Parent recipe ID (the recipe that contains others)
    /// - Child recipe ID (the recipe being used as an ingredient)
    /// - Weight (amount of the child recipe to use)
    ///
    /// # Errors
    /// - If database operation fails
    pub async fn get_meta_ingredients(&self, recipe_id: i32) -> Result<Vec<RecipeMetaIngredient>> {
        let records = sqlx::query_as!(
            RecipeMetaIngredient,
            r#"
            SELECT 
                parent_id,
                child_id,
                name,
                weight
            FROM meta_recipes JOIN recipes ON(child_id = recipe_id)
            WHERE parent_id = $1
            ORDER BY child_id
            "#,
            recipe_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    pub async fn add_meta_ingredient(&self, meta: RecipeMetaIngredient) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO meta_recipes (parent_id, child_id, weight)
            VALUES ($1, $2, $3)
            "#,
            meta.parent_id,
            meta.child_id,
            meta.weight,
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_meta_ingredient(
        &self,
        meta: RecipeMetaIngredient,
    ) -> Result<RecipeMetaIngredient> {
        let result = sqlx::query!(
            r#"
            UPDATE meta_recipes 
            SET weight = $3
            WHERE parent_id = $1 AND child_id = $2
            "#,
            meta.parent_id,
            meta.child_id,
            meta.weight,
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "MetaRecipe",
                id: format!("parent:{} child:{}", meta.parent_id, meta.child_id),
            });
        }

        Ok(meta)
    }

    pub async fn delete_meta_ingredient(&self, parent_id: i32, child_id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM meta_recipes 
            WHERE parent_id = $1 AND child_id = $2
            "#,
            parent_id,
            child_id,
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "MetaRecipe",
                id: format!("parent:{} child:{}", parent_id, child_id),
            });
        }

        Ok(())
    }

    // Advanced operations
    pub async fn get_resolved_recipe(&self, recipe_id: i32) -> Result<Vec<ResolvedRecipe>> {
        let records = sqlx::query_as!(
            ResolvedRecipe,
            r#"
            SELECT 
                r.recipe_id,
                r.name as recipe_name,
                i.ingredient_id,
                i.name as ingredient_name,
                coalesce(rr.weight, 0) as "weight!",
                sr.recipe_id as subrecipe_id,
                sr.name as subrecipe_name,
                rr.acc as hierarchy_path
            FROM resolved_recipes rr
            JOIN recipes r ON r.recipe_id = rr.recipe_id
            LEFT JOIN ingredients i ON i.ingredient_id = rr.ingredient_id
            LEFT JOIN recipes sr ON sr.recipe_id = rr.subrecipe_id
            WHERE r.recipe_id = $1
            ORDER BY COALESCE(sr.name, i.name)
            "#,
            recipe_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_recipe_stats(&self, recipe_id: i32) -> Result<RecipeStats> {
        sqlx::query_as!(
            RecipeStats,
            r#"
            SELECT 
                recipe_id as "recipe_id!",
                COALESCE(weight, 0) as "weight!",
                COALESCE(energy, 0) as "energy!"
            FROM recipe_stats
            WHERE recipe_id = $1
            "#,
            recipe_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "RecipeStats",
            id: recipe_id.to_string(),
        })
    }

    pub async fn search_by_ingredients(
        &self,
        ingredients: &[i32],
        limit: Option<i64>,
    ) -> Result<Vec<Recipe>> {
        let records = sqlx::query_as!(
            Recipe,
            r#"
            WITH matching_recipes AS (
                SELECT r.recipe_id, r.name, r.comment, r.owner_id,
                       COUNT(DISTINCT ri.ingredient_id) as matching_ingredients,
                       (SELECT COUNT(DISTINCT ingredient_id) 
                        FROM recipe_ingredients 
                        WHERE recipe_id = r.recipe_id) as total_ingredients
                FROM recipes r
                JOIN recipe_ingredients ri ON r.recipe_id = ri.recipe_id
                WHERE ri.ingredient_id = ANY($1)
                GROUP BY r.recipe_id, r.name, r.comment
            )
            SELECT 
                recipe_id as id,
                name,
                comment,
                owner_id
            FROM matching_recipes
            ORDER BY 
                matching_ingredients::float / total_ingredients DESC,
                name ASC
            LIMIT $2
            "#,
            ingredients,
            limit.unwrap_or(10)
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Updates all ingredients and sub-recipes for a recipe.
    /// Replaces all existing entries with the new ones.
    pub async fn update_recipe_entries(
        &self,
        recipe_id: i32,
        regular_ingredients: Vec<RecipeIngredient>,
        meta_ingredients: Vec<RecipeMetaIngredient>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete all existing ingredients and meta recipes
        sqlx::query!(
            "DELETE FROM recipe_ingredients WHERE recipe_id = $1",
            recipe_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!("DELETE FROM meta_recipes WHERE parent_id = $1", recipe_id)
            .execute(&mut *tx)
            .await?;

        // Insert new ingredients
        for ingredient in regular_ingredients {
            sqlx::query!(
                r#"
            INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id)
            VALUES ($1, $2, $3, $4)
            "#,
                recipe_id,
                ingredient.ingredient_id,
                ingredient.amount,
                ingredient.unit_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        // Insert new meta recipes
        for meta in meta_ingredients {
            sqlx::query!(
                r#"
            INSERT INTO meta_recipes (parent_id, child_id, weight)
            VALUES ($1, $2, $3)
            "#,
                recipe_id,
                meta.child_id,
                meta.weight,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_recipe_steps(&self, recipe_id: i32, steps: Vec<RecipeStep>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete existing steps
        sqlx::query!(r#"DELETE FROM steps WHERE recipe_id = $1"#, recipe_id)
            .execute(&mut *tx)
            .await?;

        // Insert new steps
        for step in steps {
            sqlx::query!(
                r#"
                INSERT INTO steps (
                    recipe_id, step_order, step_name, step_description,
                    fixed_duration, duration_per_kg
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                recipe_id,
                step.order,
                step.name,
                step.description,
                step.fixed_duration,
                step.duration_per_kg,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::recipe::*;
    use bigdecimal::BigDecimal;
    use core::time;
    use sqlx::PgPool;
    use std::str::FromStr;

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
}
