// foodlib_new/src/ops/ingredients.rs

use crate::{entities::ingredient::*, error::Result, recipe::Recipe};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct IngredientOps {
    pool: Arc<PgPool>,
}

impl IngredientOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    // Basic CRUD operations
    pub async fn create(&self, ingredient: Ingredient) -> Result<Ingredient> {
        let row = sqlx::query_as!(
            Ingredient,
            r#"
            INSERT INTO ingredients (name, energy, comment, owner_id)
            VALUES ($1, $2, $3, $4)
            RETURNING ingredient_id as "id", name, energy, comment, owner_id
            "#,
            ingredient.name,
            ingredient.energy,
            ingredient.comment,
            ingredient.owner_id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get(&self, id: i32) -> Result<Ingredient> {
        let row = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT 
                ingredient_id as "id", 
                name, 
                energy, 
                owner_id,
                comment 
            FROM ingredients 
            WHERE ingredient_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Ingredient> {
        let row = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT 
                ingredient_id as "id", 
                name, 
                energy, 
                owner_id,
                comment 
            FROM ingredients 
            WHERE name = $1
            "#,
            name
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn usages(&self, ingredient_id: i32) -> Result<Vec<Recipe>> {
        let row = sqlx::query_as!(
            Recipe,
            r#"
            SELECT 
                recipe_id as "id", 
                name, 
                owner_id,
                comment 
            FROM recipes JOIN recipe_ingredients USING(recipe_id)
            WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update(&self, ingredient: Ingredient) -> Result<Ingredient> {
        let row = sqlx::query_as!(
            Ingredient,
            r#"
            UPDATE ingredients
            SET name = $1, energy = $2, comment = $3
            WHERE ingredient_id = $4
            RETURNING ingredient_id as "id", name, energy, comment, owner_id
            "#,
            ingredient.name,
            ingredient.energy,
            ingredient.comment,
            ingredient.id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        // Start transaction to handle cascading deletes
        let mut tx = self.pool.begin().await?;

        // Delete from recipe_ingredients first
        sqlx::query!(
            r#"DELETE FROM recipe_ingredients WHERE ingredient_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // Delete from ingredient_sources
        sqlx::query!(
            r#"DELETE FROM ingredient_sources WHERE ingredient_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // Delete from weights
        sqlx::query!(r#"DELETE FROM weights WHERE ingredient_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        // Delete from ingredient_properties
        sqlx::query!(
            r#"DELETE FROM ingredient_properties WHERE ingredient_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // Finally delete the ingredient
        sqlx::query!(r#"DELETE FROM ingredients WHERE ingredient_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Ingredient>> {
        let rows = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT 
                ingredient_id as "id", 
                name, 
                energy, 
                owner_id,
                comment 
            FROM ingredients 
            ORDER BY ingredient_id
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_with_sources(&self) -> Result<Vec<IngredientWithSource>> {
        let rows = sqlx::query_as!(
            IngredientWithSource,
            r#"
            SELECT 
                ingredient_id as "id", 
                name, 
                energy, 
                owner_id,
                comment,
                exists (select * from ingredient_sources as iss where iss.ingredient_id = ingredients.ingredient_id) as "has_sources!" 
            FROM ingredients 
            ORDER BY ingredient_id
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    // Source operations
    pub async fn add_source(&self, source: IngredientSource) -> Result<IngredientSource> {
        let row = sqlx::query_as!(
            IngredientSource,
            r#"
            INSERT INTO ingredient_sources (
                ingredient_id, store_id, package_size, unit_id, price, url, comment
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                ingredient_source_id as "id",
                ingredient_id,
                store_id,
                package_size,
                unit_id,
                price,
                url,
                comment
            "#,
            source.ingredient_id,
            source.store_id,
            source.package_size,
            source.unit_id,
            source.price,
            source.url,
            source.comment,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_source(&self, source: IngredientSource) -> Result<IngredientSource> {
        let row = sqlx::query_as!(
            IngredientSource,
            r#"
            UPDATE ingredient_sources
            SET 
                store_id = $1,
                package_size = $2,
                unit_id = $3,
                price = $4,
                url = $5,
                comment = $6
            WHERE ingredient_source_id = $7
            RETURNING 
                ingredient_source_id as "id",
                ingredient_id,
                store_id,
                package_size,
                unit_id,
                price,
                url,
                comment
            "#,
            source.store_id,
            source.package_size,
            source.unit_id,
            source.price,
            source.url,
            source.comment,
            source.id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_source(&self, id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // First delete any category information
        sqlx::query!(
            r#"DELETE FROM metro_categories WHERE ingredient_source_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // Then delete the source
        sqlx::query!(
            r#"DELETE FROM ingredient_sources WHERE ingredient_source_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_sources(&self, ingredient_id: i32) -> Result<Vec<IngredientSource>> {
        let rows = sqlx::query_as!(
            IngredientSource,
            r#"
            SELECT 
                ingredient_source_id as "id",
                ingredient_id,
                store_id,
                package_size,
                unit_id,
                price,
                url,
                comment
            FROM ingredient_sources
            WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    // Weight operations
    pub async fn add_weight(&self, weight: IngredientWeight) -> Result<IngredientWeight> {
        let row = sqlx::query_as!(
            IngredientWeight,
            r#"
            INSERT INTO weights (ingredient_id, unit_id, weight)
            VALUES ($1, $2, $3)
            RETURNING ingredient_id, unit_id, weight
            "#,
            weight.ingredient_id,
            weight.unit_id,
            weight.weight,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_weight(&self, weight: IngredientWeight) -> Result<IngredientWeight> {
        let row = sqlx::query_as!(
            IngredientWeight,
            r#"
            UPDATE weights
            SET weight = $1
            WHERE ingredient_id = $2 AND unit_id = $3
            RETURNING ingredient_id, unit_id, weight
            "#,
            weight.weight,
            weight.ingredient_id,
            weight.unit_id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_weight(&self, ingredient_id: i32, unit_id: i32) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM weights WHERE ingredient_id = $1 AND unit_id = $2"#,
            ingredient_id,
            unit_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_weights(&self, ingredient_id: i32) -> Result<Vec<IngredientWeight>> {
        let rows = sqlx::query_as!(
            IngredientWeight,
            r#"
            SELECT ingredient_id, unit_id, weight
            FROM weights
            WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    // Property operations
    pub async fn add_property(&self, property: IngredientProperty) -> Result<IngredientProperty> {
        let row = sqlx::query_as!(
            IngredientProperty,
            r#"
            INSERT INTO ingredient_properties (ingredient_id, property_id)
            VALUES ($1, $2)
            RETURNING ingredient_id, property_id
            "#,
            property.ingredient_id,
            property.property_id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_property(&self, ingredient_id: i32, property_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM ingredient_properties 
            WHERE ingredient_id = $1 AND property_id = $2
            "#,
            ingredient_id,
            property_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_properties(&self, ingredient_id: i32) -> Result<Vec<IngredientProperty>> {
        let rows = sqlx::query_as!(
            IngredientProperty,
            r#"
            SELECT ingredient_id, property_id
            FROM ingredient_properties
            WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    // Category operations
    pub async fn set_category(&self, category: IngredientCategory) -> Result<IngredientCategory> {
        let row = sqlx::query_as!(
            IngredientCategory,
            r#"
            INSERT INTO metro_categories (ingredient_source_id, category)
            VALUES ($1, $2)
            ON CONFLICT (ingredient_source_id) DO UPDATE 
            SET category = $2
            RETURNING ingredient_source_id, category
            "#,
            category.ingredient_source_id,
            category.category,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_category(&self, source_id: i32) -> Result<Option<IngredientCategory>> {
        let row = sqlx::query_as!(
            IngredientCategory,
            r#"
            SELECT ingredient_source_id, category
            FROM metro_categories
            WHERE ingredient_source_id = $1
            "#,
            source_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row)
    }
}
