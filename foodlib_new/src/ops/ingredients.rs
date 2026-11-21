// foodlib_new/src/ops/ingredients.rs

use crate::entities::types::amount::Amount;
use crate::entities::types::money::Money;
use crate::{
    entities::ingredient::*,
    entities::recipe::{Recipe, RecipeKind},
    entities::types::ids::{
        GroupId, IngredientId, IngredientSourceId, PropertyId, RecipeId, StoreId, UnitId, UserId,
    },
    error::Result,
    Error,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct IngredientOps {
    pool: Arc<PgPool>,
}

#[derive(Debug, Default)]
pub struct IngredientRepoFilter<'a> {
    pub group_ids: Option<&'a [GroupId]>,
    pub search: Option<&'a str>,
    pub property_id: Option<PropertyId>,
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
            INSERT INTO ingredients (name, energy, comment, group_id)
            VALUES ($1, $2, $3, $4)
            RETURNING ingredient_id as "id: IngredientId", name, energy, comment, group_id as "group_id: GroupId"
            "#,
            ingredient.name,
            ingredient.energy.get().clone(),
            ingredient.comment,
            ingredient.group_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get(&self, id: IngredientId) -> Result<Ingredient> {
        let row = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT 
                ingredient_id as "id: IngredientId", 
                name, 
                energy as "energy: Amount", 
                group_id as "group_id: GroupId",
                comment 
            FROM ingredients 
            WHERE ingredient_id = $1
            "#,
            id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Ingredient> {
        sqlx::query_as!(
            Ingredient,
            r#"
            SELECT 
                ingredient_id as "id: IngredientId", 
                name, 
                energy as "energy: Amount", 
                group_id as "group_id: GroupId",
                comment 
            FROM ingredients 
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Ingredient",
            id: name.to_string(),
        })
    }

    pub async fn usages(&self, ingredient_id: IngredientId) -> Result<Vec<Recipe>> {
        let row = sqlx::query_as!(
            Recipe,
            r#"
            SELECT
                recipe_id as "id: RecipeId",
                name,
                group_id as "group_id: GroupId",
                comment,
                weighed_in,
                kind as "kind: RecipeKind"
            FROM recipes JOIN recipe_ingredients USING(recipe_id)
            WHERE ingredient_id = $1
            "#,
            ingredient_id.get()
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
            RETURNING ingredient_id as "id: IngredientId", name, energy, comment, group_id as "group_id: GroupId"
            "#,
            ingredient.name,
            ingredient.energy.get().clone(),
            ingredient.comment,
            ingredient.id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn set_group(&self, ingredient_id: IngredientId, group_id: GroupId) -> Result<()> {
        sqlx::query!(
            r#"UPDATE ingredients SET group_id = $1 WHERE ingredient_id = $2"#,
            group_id.get(),
            ingredient_id.get()
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: IngredientId) -> Result<()> {
        // Start transaction to handle cascading deletes
        let mut tx = self.pool.begin().await?;

        // Delete from recipe_ingredients first
        sqlx::query!(
            r#"DELETE FROM recipe_ingredients WHERE ingredient_id = $1"#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        // Delete from ingredient_sources
        sqlx::query!(
            r#"DELETE FROM ingredient_sources WHERE ingredient_id = $1"#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        // Delete from weights
        sqlx::query!(r#"DELETE FROM weights WHERE ingredient_id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        // Delete from ingredient_properties
        sqlx::query!(
            r#"DELETE FROM ingredient_properties WHERE ingredient_id = $1"#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        // Finally delete the ingredient
        sqlx::query!(r#"DELETE FROM ingredients WHERE ingredient_id = $1"#, id.get())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Lists ingredients whose group the user belongs to
    pub async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Ingredient>> {
        let rows = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT ingredient_id as "id: IngredientId", name, energy, group_id as "group_id: GroupId", comment
            FROM ingredients
            WHERE group_id IN (
                SELECT group_id FROM user_group_roles WHERE user_id = $1
            )
            ORDER BY name
            "#,
            user_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_with_sources_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<IngredientWithSource>> {
        let rows = sqlx::query_as!(
            IngredientWithSource,
            r#"
            SELECT
                ingredient_id as "id: IngredientId", name, energy, group_id as "group_id: GroupId", comment,
                exists (select * from ingredient_sources as iss where iss.ingredient_id = ingredients.ingredient_id) as "has_sources!"
            FROM ingredients
            WHERE group_id IN (
                SELECT group_id FROM user_group_roles WHERE user_id = $1
            )
            ORDER BY name
            "#,
            user_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list(&self) -> Result<Vec<Ingredient>> {
        let rows = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT 
                ingredient_id as "id: IngredientId", 
                name, 
                energy as "energy: Amount", 
                group_id as "group_id: GroupId",
                comment 
            FROM ingredients 
            ORDER BY ingredient_id
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    #[deprecated(
        note = "Use `list_with_sources_filtered` instead"
    )]
    pub async fn list_with_sources(&self) -> Result<Vec<IngredientWithSource>> {
        let rows = sqlx::query_as!(
            IngredientWithSource,
            r#"
            SELECT
                ingredient_id as "id: IngredientId",
                name,
                energy as "energy: Amount",
                group_id as "group_id: GroupId",
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

    pub async fn list_filtered_with_sources(
        &self,
        filter: IngredientRepoFilter<'_>,
    ) -> Result<Vec<IngredientWithSource>> {

        fn escape_like(input: &str) -> String {
            input
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        }

        let group_ids: Option<Vec<i32>> = filter
            .group_ids
            .map(|ids| ids.iter().map(|id| id.get()).collect());

        let search_pattern = filter
            .search
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", escape_like(s)));

        let rows = sqlx::query_as!(
            IngredientWithSource,
            r#"
            SELECT
                i.ingredient_id AS "id: IngredientId",
                i.name,
                i.energy AS "energy: Amount",
                i.group_id AS "group_id: GroupId",
                i.comment,
                EXISTS (
                    SELECT 1
                    FROM ingredient_sources AS iss
                    WHERE iss.ingredient_id = i.ingredient_id
                ) AS "has_sources!"
            FROM ingredients AS i
            WHERE ($1::int4[] IS NULL OR i.group_id = ANY($1))
              AND ($2::text IS NULL OR i.name ILIKE $2)
              AND (
                  $3::int4 IS NULL
                  OR EXISTS (
                      SELECT 1
                      FROM ingredient_properties AS ip
                      WHERE ip.ingredient_id = i.ingredient_id
                        AND ip.property_id = $3
                  )
              )
            ORDER BY i.ingredient_id
            "#,
            group_ids.as_deref(),
            search_pattern,
            filter.property_id as Option<PropertyId>,
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
                ingredient_source_id as "id: IngredientSourceId",
                ingredient_id as "ingredient_id: IngredientId",
                store_id as "store_id: StoreId",
                package_size as "package_size: Amount",
                unit_id as "unit_id: UnitId",
                price as "price: Money",
                url,
                comment
            "#,
            source.ingredient_id.get(),
            source.store_id.get(),
            source.package_size.get().clone(),
            source.unit_id.get(),
            source.price.get().clone(),
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
                ingredient_source_id as "id: IngredientSourceId",
                ingredient_id as "ingredient_id: IngredientId",
                store_id as "store_id: StoreId",
                package_size as "package_size: Amount",
                unit_id as "unit_id: UnitId",
                price as "price: Money",
                url,
                comment
            "#,
            source.store_id.get(),
            source.package_size.get().clone(),
            source.unit_id.get(),
            source.price.get().clone(),
            source.url,
            source.comment,
            source.id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_source(&self, id: IngredientSourceId) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // First delete any category information
        sqlx::query!(
            r#"DELETE FROM metro_categories WHERE ingredient_source_id = $1"#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        // Then delete the source
        sqlx::query!(
            r#"DELETE FROM ingredient_sources WHERE ingredient_source_id = $1"#,
            id.get()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_sources(&self, ingredient_id: IngredientId) -> Result<Vec<IngredientSource>> {
        let rows = sqlx::query_as!(
            IngredientSource,
            r#"
            SELECT 
                ingredient_source_id as "id: IngredientSourceId",
                ingredient_id as "ingredient_id: IngredientId",
                store_id as "store_id: StoreId",
                package_size as "package_size: Amount",
                unit_id as "unit_id: UnitId",
                price as "price: Money",
                url,
                comment
            FROM ingredient_sources
            WHERE ingredient_id = $1
            "#,
            ingredient_id.get()
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
            RETURNING ingredient_id as "ingredient_id: IngredientId", unit_id as "unit_id: UnitId", weight
            "#,
            weight.ingredient_id.get(),
            weight.unit_id.get(),
            weight.weight.get().clone(),
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
            RETURNING ingredient_id as "ingredient_id: IngredientId", unit_id as "unit_id: UnitId", weight
            "#,
            weight.weight.get().clone(),
            weight.ingredient_id.get(),
            weight.unit_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_weight(&self, ingredient_id: IngredientId, unit_id: UnitId) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM weights WHERE ingredient_id = $1 AND unit_id = $2"#,
            ingredient_id.get(),
            unit_id.get()
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_weights(&self, ingredient_id: IngredientId) -> Result<Vec<IngredientWeight>> {
        let rows = sqlx::query_as!(
            IngredientWeight,
            r#"
            SELECT ingredient_id as "ingredient_id: IngredientId", unit_id as "unit_id: UnitId", weight
            FROM weights
            WHERE ingredient_id = $1
            "#,
            ingredient_id.get()
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
            RETURNING ingredient_id as "ingredient_id: IngredientId", property_id as "property_id: PropertyId"
            "#,
            property.ingredient_id.get(),
            property.property_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_property(&self, ingredient_id: IngredientId, property_id: PropertyId) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM ingredient_properties 
            WHERE ingredient_id = $1 AND property_id = $2
            "#,
            ingredient_id.get(),
            property_id.get()
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_properties(&self, ingredient_id: IngredientId) -> Result<Vec<IngredientProperty>> {
        let rows = sqlx::query_as!(
            IngredientProperty,
            r#"
            SELECT ingredient_id as "ingredient_id: IngredientId", property_id as "property_id: PropertyId"
            FROM ingredient_properties
            WHERE ingredient_id = $1
            "#,
            ingredient_id.get()
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
            RETURNING ingredient_source_id as "ingredient_source_id: IngredientSourceId", category
            "#,
            category.ingredient_source_id.get(),
            category.category,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_category(&self, source_id: IngredientSourceId) -> Result<Option<IngredientCategory>> {
        let row = sqlx::query_as!(
            IngredientCategory,
            r#"
            SELECT ingredient_source_id as "ingredient_source_id: IngredientSourceId", category
            FROM metro_categories
            WHERE ingredient_source_id = $1
            "#,
            source_id.get()
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_sources(&self, ingredient_id: Option<IngredientId>) -> Result<Vec<IngredientSource>> {
        let rows = match ingredient_id {
            Some(id) => sqlx::query_as!(
                IngredientSource,
                r#"
                SELECT ingredient_source_id as "id: IngredientSourceId", ingredient_id as "ingredient_id: IngredientId", store_id as "store_id: StoreId", package_size, unit_id as "unit_id: UnitId", price, url, comment
                FROM ingredient_sources WHERE ingredient_id = $1 ORDER BY ingredient_source_id
                "#,
                id.get()
            )
            .fetch_all(&*self.pool)
            .await?,
            None => sqlx::query_as!(
                IngredientSource,
                r#"
                SELECT ingredient_source_id as "id: IngredientSourceId", ingredient_id as "ingredient_id: IngredientId", store_id as "store_id: StoreId", package_size, unit_id as "unit_id: UnitId", price, url, comment
                FROM ingredient_sources ORDER BY ingredient_id
                "#,
            )
            .fetch_all(&*self.pool)
            .await?,
        };
        Ok(rows)
    }
}
impl IngredientOps {
    /// Ids aller Zutaten, die eine bestimmte Eigenschaft tragen — Grundlage
    /// für den Property-Filter im Zutaten-Picker (kein Voll-Laden nötig).
    pub async fn ingredient_ids_for_property(&self, property_id: PropertyId) -> Result<Vec<i32>> {
        let rows = sqlx::query_scalar!(
            r#"
            SELECT ingredient_id
            FROM ingredient_properties
            WHERE property_id = $1
            "#,
            property_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
