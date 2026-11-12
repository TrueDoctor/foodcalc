use crate::entities::recipe::{
    Recipe, RecipeIngredient, RecipeKind, RecipeListItem, RecipeStep,
    RecipeTreeIngredient, RecipeTreeNode,
};
use crate::error::{Error, Result};
use crate::recipe::{RecipeMetaIngredient, RecipeStats, ResolvedRecipe};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::collections::HashMap;
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

fn f64_to_pg_interval(seconds: f64) -> sqlx::postgres::types::PgInterval {
    sqlx::postgres::types::PgInterval {
        months: 0,
        days: 0,
        microseconds: (seconds * 1_000_000.0) as i64,
    }
}

fn pg_interval_to_f64(interval: &sqlx::postgres::types::PgInterval) -> f64 {
    interval.microseconds as f64 / 1_000_000.0
}

struct RecipeStepDB {
    id: i32,
    recipe_id: i32,
    order: f64,
    name: String,
    description: String,
    fixed_duration: sqlx::postgres::types::PgInterval,
    duration_per_kg: sqlx::postgres::types::PgInterval,
}

impl RecipeStepDB {
    fn to_recipe_step(&self) -> RecipeStep {
        RecipeStep {
            id: self.id,
            order: self.order,
            name: self.name.clone(),
            description: self.description.clone(),
            fixed_duration: pg_interval_to_f64(&self.fixed_duration),
            duration_per_kg: pg_interval_to_f64(&self.duration_per_kg),
            recipe_id: self.recipe_id,
        }
    }
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
            INSERT INTO recipes (name, comment, group_id, kind)
            VALUES ($1, $2, $3, $4::text::recipe_kind)
            RETURNING recipe_id as id, name, comment, group_id, weighed_in,
                     kind as "kind: RecipeKind"
            "#,
            recipe.name,
            recipe.comment,
            recipe.group_id,
            recipe.kind.as_str(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get(&self, id: i32) -> Result<Recipe> {
        let record = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as id, name, comment, group_id, weighed_in,
                   kind as "kind: RecipeKind"
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
            SELECT recipe_id as id, name, comment, group_id, weighed_in,
                   kind as "kind: RecipeKind"
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
            SET name = $1, comment = $2, weighed_in = $4, kind = $5::text::recipe_kind
            WHERE recipe_id = $3
            RETURNING recipe_id as id, name, comment, group_id, weighed_in,
                     kind as "kind: RecipeKind"
            "#,
            recipe.name,
            recipe.comment,
            recipe.id,
            recipe.weighed_in,
            recipe.kind.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: recipe.id.to_string(),
        })?;

        Ok(record)
    }

    pub async fn set_group(&self, recipe_id: i32, group_id: i32) -> Result<()> {
        sqlx::query!(
            r#"UPDATE recipes SET group_id = $1 WHERE recipe_id = $2"#,
            group_id,
            recipe_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
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

    /// Lists recipes whose group the user belongs to
    pub async fn list_for_user(&self, user_id: i64) -> Result<Vec<Recipe>> {
        let records = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as id, name, comment, group_id, weighed_in,
                   kind as "kind: RecipeKind"
            FROM recipes
            WHERE group_id IN (
                SELECT group_id FROM user_groups WHERE user_id = $1
            )
            ORDER BY name
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Lists all recipes (admin only)
    pub async fn list(&self) -> Result<Vec<Recipe>> {
        let records = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as id, name, comment, group_id, weighed_in,
                   kind as "kind: RecipeKind"
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
            RecipeStepDB,
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
            f64_to_pg_interval(step.fixed_duration),
            f64_to_pg_interval(step.duration_per_kg),
        )
            .fetch_one(&*self.pool)
            .await?;

        Ok(record.to_recipe_step())
    }

    pub async fn update_step(&self, step: RecipeStep) -> Result<RecipeStep> {
        let record = sqlx::query_as!(
            RecipeStepDB,
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
            f64_to_pg_interval(step.fixed_duration),
            f64_to_pg_interval(step.duration_per_kg),
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "RecipeStep",
            id: step.id.to_string(),
        })?;

        Ok(record.to_recipe_step())
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
            RecipeStepDB,
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

        Ok(
            records.into_iter()
                .map(|record| record.to_recipe_step())
                .collect()
        )
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
                SELECT r.recipe_id, r.name, r.comment, r.group_id, r.weighed_in, r.kind,
                       COUNT(DISTINCT ri.ingredient_id) as matching_ingredients,
                       (SELECT COUNT(DISTINCT ingredient_id)
                        FROM recipe_ingredients
                        WHERE recipe_id = r.recipe_id) as total_ingredients
                FROM recipes r
                JOIN recipe_ingredients ri ON r.recipe_id = ri.recipe_id
                WHERE ri.ingredient_id = ANY($1)
                GROUP BY r.recipe_id, r.name, r.comment, r.weighed_in
            )
            SELECT
                recipe_id as id,
                name,
                comment,
                group_id,
                weighed_in,
                kind as "kind: RecipeKind"
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
                f64_to_pg_interval(step.fixed_duration),
                f64_to_pg_interval(step.duration_per_kg),
            )
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Lists all recipes together with their aggregated stats from the
    /// `recipe_stats` view (weight kg / energy kJ). Empty recipes get 0/0,
    /// which lets callers compute the energy density (kJ/g) safely.
    pub async fn list_with_stats(&self) -> Result<Vec<RecipeListItem>> {
        let records = sqlx::query_as!(
            RecipeListItem,
            r#"
            SELECT
                r.recipe_id as id,
                r.name,
                r.comment,
                r.group_id,
                r.kind as "kind: RecipeKind",
                r.weighed_in,
                COALESCE(s.weight, 0) as "weight!",
                COALESCE(s.energy, 0) as "energy!"
            FROM recipes r
            LEFT JOIN recipe_stats s ON s.recipe_id = r.recipe_id
            ORDER BY r.name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Deep-copies a recipe (ingredients, steps, sub-recipes) into a new,
    /// independent recipe owned by `group_id`. Returns the new recipe id.
    pub async fn duplicate(
        &self,
        recipe_id: i32,
        new_name: Option<String>,
        new_group_id: i32,
    ) -> Result<i32> {
        let mut tx = self.pool.begin().await?;

        let source = sqlx::query!(
            r#"SELECT name, comment, kind as "kind: RecipeKind", weighed_in FROM recipes WHERE recipe_id = $1"#,
            recipe_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: recipe_id.to_string(),
        })?;

        let name = new_name.unwrap_or_else(|| format!("{} (Kopie)", source.name));

        let new_id: i32 = sqlx::query_scalar!(
            r#"
            INSERT INTO recipes (name, comment, group_id, weighed_in, kind)
            VALUES ($1, $2, $3, $4, $5::text::recipe_kind)
            RETURNING recipe_id
            "#,
            name,
            source.comment,
            new_group_id,
            source.weighed_in,
            source.kind.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id)
            SELECT $1, ingredient_id, amount, unit_id
            FROM recipe_ingredients WHERE recipe_id = $2
            "#,
            new_id,
            recipe_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO steps (recipe_id, step_order, step_name, step_description, fixed_duration, duration_per_kg)
            SELECT $1, step_order, step_name, step_description, fixed_duration, duration_per_kg
            FROM steps WHERE recipe_id = $2
            "#,
            new_id,
            recipe_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO meta_recipes (parent_id, child_id, weight)
            SELECT $1, child_id, weight FROM meta_recipes WHERE parent_id = $2
            "#,
            new_id,
            recipe_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(new_id)
    }

    /// The full recursion tree of a recipe: the recipe itself with its
    /// direct ingredients (unit-converted weights via `resolved_recipes`)
    /// and every used sub-recipe, recursively. Cycle-safe via a depth cap
    /// and path de-duplication.
    ///
    /// Structure: a depth-first walk over `meta_recipes` yields one path per
    /// node (`"<root>.<sub>.<sub>"`); `resolved_recipes.acc` uses the same
    /// id chains, so ingredient leaves attach to nodes by matching path.
    pub async fn get_recipe_tree(&self, root_id: i32) -> Result<RecipeTreeNode> {
        #[derive(Debug)]
        struct WalkRow {
            path: String,
            node_id: i32,
            parent_weight: Option<BigDecimal>,
        }
        let walk = sqlx::query_as!(
            WalkRow,
            r#"
            WITH RECURSIVE walk(path, node_id, parent_weight, depth) AS (
                SELECT $1::text, $2::int, NULL::numeric, 0
                UNION ALL
                SELECT w.path || '.' || m.child_id, m.child_id, m.weight, w.depth + 1
                FROM walk w
                JOIN meta_recipes m ON m.parent_id = w.node_id
                WHERE w.depth < 10
                  AND position('.' || m.child_id::text || '.' IN w.path || '.') = 0
            )
            SELECT path AS "path!", node_id AS "node_id!", parent_weight
            FROM walk
            "#,
            root_id.to_string(),
            root_id
        )
        .fetch_all(&*self.pool)
        .await?;

        // Node recipe data (name, kind) for every visited node.
        let node_ids: Vec<i32> = walk.iter().map(|row| row.node_id).collect();
        let mut name_by_id: HashMap<i32, (String, RecipeKind)> = HashMap::new();
        for id in &node_ids {
            if let Ok(recipe) = self.get(*id).await {
                name_by_id.insert(*id, (recipe.name, recipe.kind));
            }
        }

        // Leaf rows: the raw ingredient list of every walked node with the
        // base weight (amount × kg-per-unit) — the recipe's own scale, as
        // shown in the tree view (event scaling happens per meal elsewhere).
        #[derive(Debug)]
        struct LeafRow {
            acc: String,
            ingredient_id: i32,
            ingredient: Option<String>,
            weight: Option<BigDecimal>,
            amount: BigDecimal,
            unit_id: i32,
            unit_name: Option<String>,
        }
        let leaves = sqlx::query_as!(
            LeafRow,
            r#"
            WITH RECURSIVE walk(path, node_id, parent_weight, depth) AS (
                SELECT $1::text, $2::int, NULL::numeric, 0
                UNION ALL
                SELECT w.path || '.' || m.child_id, m.child_id, m.weight, w.depth + 1
                FROM walk w
                JOIN meta_recipes m ON m.parent_id = w.node_id
                WHERE w.depth < 10
                  AND position('.' || m.child_id::text || '.' IN w.path || '.') = 0
            )
            SELECT walk.path AS "acc!",
                   ri.ingredient_id AS "ingredient_id!",
                   i.name AS ingredient,
                   (ri.amount * w.weight) AS "weight?",
                   ri.amount AS "amount!",
                   ri.unit_id AS "unit_id!",
                   u.name AS unit_name
            FROM walk
            JOIN recipe_ingredients ri ON ri.recipe_id = walk.node_id
            LEFT JOIN ingredients i ON i.ingredient_id = ri.ingredient_id
            LEFT JOIN weights w
              ON w.ingredient_id = ri.ingredient_id AND w.unit_id = ri.unit_id
            LEFT JOIN units u ON u.unit_id = ri.unit_id
            "#,
            root_id.to_string(),
            root_id
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut leaves_by_acc: HashMap<String, Vec<RecipeTreeIngredient>> = HashMap::new();
        for leaf in &leaves {
            leaves_by_acc
                .entry(leaf.acc.clone())
                .or_default()
                .push(RecipeTreeIngredient {
                    ingredient_id: leaf.ingredient_id,
                    name: leaf.ingredient.clone(),
                    amount: leaf.amount.clone(),
                    unit_id: leaf.unit_id,
                    unit_name: leaf.unit_name.clone(),
                    weight: leaf.weight.clone(),
                });
        }

        // Pass A: build all nodes keyed by path.
        let mut nodes: HashMap<String, RecipeTreeNode> = HashMap::new();
        for row in &walk {
            let (name, kind) = name_by_id
                .get(&row.node_id)
                .cloned()
                .unwrap_or_else(|| ("?".into(), RecipeKind::Meal));
            nodes.insert(
                row.path.clone(),
                RecipeTreeNode {
                    recipe_id: row.node_id,
                    name,
                    kind,
                    weight: row.parent_weight.clone(),
                    ingredients: leaves_by_acc.remove(&row.path).unwrap_or_default(),
                    children: vec![],
                },
            );
        }

        // Pass B: attach deepest-first so every parent has collected all its
        // children before it is moved into its own parent.
        let mut attach_order: Vec<&WalkRow> = walk.iter().collect();
        attach_order.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
        let mut root_node: Option<RecipeTreeNode> = None;
        for row in attach_order {
            let node = nodes.remove(&row.path).expect("node built in pass A");
            let parent_path = row
                .path
                .rsplit_once('.')
                .map(|(parent, _)| parent.to_string());
            match parent_path {
                Some(parent_path) => {
                    if let Some(parent) = nodes.get_mut(&parent_path) {
                        parent.children.push(node);
                    }
                }
                None => root_node = Some(node),
            }
        }

        root_node.ok_or(Error::NotFound {
            entity: "Recipe",
            id: root_id.to_string(),
        })
    }
}
