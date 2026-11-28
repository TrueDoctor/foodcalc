use crate::entities::recipe::{Recipe, RecipeIngredient, RecipeStep};
use crate::error::{Error, Result};
use crate::recipe::{RecipeMetaIngredient, RecipeStats, ResolvedRecipe};
use crate::types::ids::{GroupId, IngredientId, RecipeId, RecipeStepId, UnitId, UserId};
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
            INSERT INTO recipes (name, comment, group_id)
            VALUES ($1, $2, $3)
            RETURNING recipe_id as "id: RecipeId", name, comment, group_id as "group_id: GroupId", weighed_in
            "#,
            recipe.name,
            recipe.comment,
            recipe.group_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get(&self, id: RecipeId) -> Result<Recipe> {
        let record = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as "id: RecipeId", name, comment, group_id as "group_id: GroupId", weighed_in
            FROM recipes
            WHERE recipe_id = $1
            "#,
            id.get()
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
            SELECT recipe_id as "id: RecipeId", name, comment, group_id as "group_id: GroupId", weighed_in
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
            SET name = $1, comment = $2, weighed_in = $4
            WHERE recipe_id = $3
            RETURNING recipe_id as "id: RecipeId", name, comment, group_id as "group_id: GroupId", weighed_in
            "#,
            recipe.name,
            recipe.comment,
            recipe.id.get(),
            recipe.weighed_in
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: recipe.id.to_string(),
        })?;

        Ok(record)
    }

    pub async fn set_group(&self, recipe_id: RecipeId, group_id: GroupId) -> Result<()> {
        sqlx::query!(
            r#"UPDATE recipes SET group_id = $1 WHERE recipe_id = $2"#,
            group_id.get(),
            recipe_id.get()
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: RecipeId) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(r#"DELETE FROM recipe_ingredients WHERE recipe_id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        sqlx::query!(r#"DELETE FROM steps WHERE recipe_id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        sqlx::query!(
            r#"DELETE FROM meta_recipes WHERE parent_id = $1 OR child_id = $1"#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(r#"DELETE FROM event_meals WHERE recipe_id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        let result = sqlx::query!(r#"DELETE FROM recipes WHERE recipe_id = $1"#, id.get())
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
    pub async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Recipe>> {
        let records = sqlx::query_as!(
            Recipe,
            r#"
            SELECT recipe_id as "id: RecipeId", name, comment, group_id as "group_id: GroupId", weighed_in
            FROM recipes
            WHERE group_id IN (
                SELECT group_id FROM user_groups WHERE user_id = $1
            )
            ORDER BY name
            "#,
            user_id.get()
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
            SELECT recipe_id as "id: RecipeId", name, comment, group_id as "group_id: GroupId", weighed_in
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
            ingredient.recipe_id.get(),
            ingredient.ingredient_id.get(),
            ingredient.amount,
            ingredient.unit_id.get(),
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
            ingredient.recipe_id.get(),
            ingredient.ingredient_id.get(),
            ingredient.amount,
            ingredient.unit_id.get(),
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

    pub async fn delete_ingredient(&self, recipe_id: RecipeId, ingredient_id: IngredientId) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM recipe_ingredients
            WHERE recipe_id = $1 AND ingredient_id = $2
            "#,
            recipe_id.get(),
            ingredient_id.get(),
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

    pub async fn get_ingredients(&self, recipe_id: RecipeId) -> Result<Vec<RecipeIngredient>> {
        let records = sqlx::query_as!(
            RecipeIngredient,
            r#"
            SELECT recipe_id as "recipe_id: RecipeId", ingredient_id as "ingredient_id: IngredientId", amount as "amount", unit_id as "unit_id: UnitId", name
            FROM recipe_ingredients JOIN ingredients USING(ingredient_id)
            WHERE recipe_id = $1
            ORDER BY ingredient_id
            "#,
            recipe_id.get()
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
            step.recipe_id.get(),
            step.order,
            step.name,
            step.description,
            duration_to_pg_interval(step.fixed_duration),
            duration_to_pg_interval(step.duration_per_kg)
        )
        .fetch_one(&*self.pool)
        .await?;

        let recipe_step: RecipeStep = record.into();
        Ok(recipe_step)
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
            step.id.get(),
            step.order,
            step.name,
            step.description,
            duration_to_pg_interval(step.fixed_duration),
            duration_to_pg_interval(step.duration_per_kg),
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "RecipeStep",
            id: step.id.to_string(),
        })?;

        let recipe_step: RecipeStep = record.into();
        Ok(recipe_step)
    }

    pub async fn delete_step(&self, recipe_id: RecipeId, step_id: RecipeStepId) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM steps
            WHERE recipe_id = $1 AND step_id = $2
            "#,
            recipe_id.get(),
            step_id.get(),
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

    pub async fn get_steps(&self, recipe_id: RecipeId) -> Result<Vec<RecipeStep>> {
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
            recipe_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        let records: Vec<RecipeStep> = records.into_iter().map(|r| r.into()).collect();
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
    pub async fn get_meta_ingredients(&self, recipe_id: RecipeId) -> Result<Vec<RecipeMetaIngredient>> {
        let records = sqlx::query_as!(
            RecipeMetaIngredient,
            r#"
            SELECT
                parent_id as "parent_id: RecipeId",
                child_id as "child_id: RecipeId",
                name,
                weight
            FROM meta_recipes JOIN recipes ON(child_id = recipe_id)
            WHERE parent_id = $1
            ORDER BY child_id
            "#,
            recipe_id.get()
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
            meta.parent_id.get(),
            meta.child_id.get(),
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
            meta.parent_id.get(),
            meta.child_id.get(),
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

    pub async fn delete_meta_ingredient(&self, parent_id: RecipeId, child_id: RecipeId) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM meta_recipes
            WHERE parent_id = $1 AND child_id = $2
            "#,
            parent_id.get(),
            child_id.get(),
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
    pub async fn get_resolved_recipe(&self, recipe_id: RecipeId) -> Result<Vec<ResolvedRecipe>> {
        let records = sqlx::query_as!(
            ResolvedRecipe,
            r#"
            SELECT
                r.recipe_id as "recipe_id: RecipeId",
                r.name as recipe_name,
                i.ingredient_id as "ingredient_id: IngredientId",
                i.name as ingredient_name,
                coalesce(rr.weight, 0) as "weight!",
                sr.recipe_id as "subrecipe_id: Option<RecipeId>",
                sr.name as subrecipe_name,
                rr.acc as hierarchy_path
            FROM resolved_recipes rr
            JOIN recipes r ON r.recipe_id = rr.recipe_id
            LEFT JOIN ingredients i ON i.ingredient_id = rr.ingredient_id
            LEFT JOIN recipes sr ON sr.recipe_id = rr.subrecipe_id
            WHERE r.recipe_id = $1
            ORDER BY COALESCE(sr.name, i.name)
            "#,
            recipe_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_recipe_stats(&self, recipe_id: RecipeId) -> Result<RecipeStats> {
        sqlx::query_as!(
            RecipeStats,
            r#"
            SELECT
                recipe_id as "recipe_id!: RecipeId",
                COALESCE(weight, 0) as "weight!",
                COALESCE(energy, 0) as "energy!"
            FROM recipe_stats
            WHERE recipe_id = $1
            "#,
            recipe_id.get()
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
                SELECT r.recipe_id, r.name, r.comment, r.group_id, r.weighed_in,
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
                recipe_id as "id: RecipeId",
                name,
                comment,
                group_id as "group_id: GroupId",
                weighed_in
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
        recipe_id: RecipeId,
        regular_ingredients: Vec<RecipeIngredient>,
        meta_ingredients: Vec<RecipeMetaIngredient>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete all existing ingredients and meta recipes
        sqlx::query!(
            "DELETE FROM recipe_ingredients WHERE recipe_id = $1",
            recipe_id.get()
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!("DELETE FROM meta_recipes WHERE parent_id = $1", recipe_id.get())
            .execute(&mut *tx)
            .await?;

        // Insert new ingredients
        for ingredient in regular_ingredients {
            sqlx::query!(
                r#"
            INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id)
            VALUES ($1, $2, $3, $4)
            "#,
                recipe_id.get(),
                ingredient.ingredient_id.get(),
                ingredient.amount,
                ingredient.unit_id.get(),
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
                recipe_id.get(),
                meta.child_id.get(),
                meta.weight,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_recipe_steps(&self, recipe_id: RecipeId, steps: Vec<RecipeStep>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete existing steps
        sqlx::query!(r#"DELETE FROM steps WHERE recipe_id = $1"#, recipe_id.get())
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
                recipe_id.get(),
                step.order,
                step.name,
                step.description,
                duration_to_pg_interval(step.fixed_duration),
                duration_to_pg_interval(step.duration_per_kg),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
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

impl From<RecipeStepDB> for RecipeStep {
    fn from(row: RecipeStepDB) -> Self {
        Self {
            id: RecipeStepId(row.id),
            recipe_id: RecipeId(row.recipe_id),
            order: row.order,
            name: row.name,
            description: row.description,
            fixed_duration: pg_interval_to_duration(&row.fixed_duration),
            duration_per_kg: pg_interval_to_duration(&row.duration_per_kg),
        }
    }
}

fn pg_interval_to_duration(interval: &sqlx::postgres::types::PgInterval) -> time::Duration {
    time::Duration::days(interval.days as i64)
        + time::Duration::microseconds(interval.microseconds)
}

fn duration_to_pg_interval(duration: time::Duration) -> sqlx::postgres::types::PgInterval {
    sqlx::postgres::types::PgInterval {
        months: 0,
        days: 0,
        microseconds: duration.whole_microseconds() as i64,
    }
}