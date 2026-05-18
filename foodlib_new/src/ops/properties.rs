// foodlib_new/src/ops/properties.rs

use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    entities::property::*,
    error::{Error, Result},
};

#[derive(Clone)]
pub struct PropertyOps {
    pool: Arc<PgPool>,
}

impl PropertyOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, property: Property) -> Result<Property> {
        let row = sqlx::query_as!(
            Property,
            r#"
            INSERT INTO food_properties (name)
            VALUES ($1)
            RETURNING property_id as "id", name
            "#,
            property.name,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get(&self, id: i32) -> Result<Property> {
        let row = sqlx::query_as!(
            Property,
            r#"
            SELECT property_id as "id", name
            FROM food_properties 
            WHERE property_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Property> {
        let row = sqlx::query_as!(
            Property,
            r#"
            SELECT property_id as "id", name
            FROM food_properties 
            WHERE name = $1
            "#,
            name
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update(&self, property: Property) -> Result<Property> {
        let row = sqlx::query_as!(
            Property,
            r#"
            UPDATE food_properties
            SET name = $1
            WHERE property_id = $2
            RETURNING property_id as "id", name
            "#,
            property.name,
            property.id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // First delete ingredient associations
        sqlx::query!(
            r#"
            DELETE FROM ingredient_properties 
            WHERE property_id = $1
            "#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // Then delete the property
        sqlx::query!(
            r#"
            DELETE FROM food_properties 
            WHERE property_id = $1
            "#,
            id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Property>> {
        let rows = sqlx::query_as!(
            Property,
            r#"
            SELECT property_id as "id", name
            FROM food_properties
            ORDER BY name
            "#,
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    // Property assignment operations
    pub async fn assign_to_ingredient(&self, ingredient_id: i32, property_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO ingredient_properties (ingredient_id, property_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            ingredient_id,
            property_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_from_ingredient(&self, ingredient_id: i32, property_id: i32) -> Result<()> {
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

    pub async fn get_ingredient_properties(
        &self,
        ingredient_id: i32,
    ) -> Result<IngredientProperties> {
        let ingredient_name: String = sqlx::query_scalar!(
            r#"SELECT name FROM ingredients WHERE ingredient_id = $1"#,
            ingredient_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| Error::NotFound {
            entity: "ingredient",
            id: ingredient_id.to_string(),
        })?;

        let properties = sqlx::query_as!(
            Property,
            r#"
            SELECT fp.property_id as "id", fp.name
            FROM ingredient_properties ip
            JOIN food_properties fp ON fp.property_id = ip.property_id
            WHERE ip.ingredient_id = $1
            ORDER BY fp.name
            "#,
            ingredient_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(IngredientProperties {
            ingredient_id,
            ingredient_name,
            properties,
        })
    }

    // Recipe property analysis
    pub async fn get_recipe_properties(&self, recipe_id: i32) -> Result<RecipeProperties> {
        let recipe_name: String = sqlx::query_scalar!(
            r#"SELECT name FROM recipes WHERE recipe_id = $1"#,
            recipe_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| Error::NotFound {
            entity: "recipe",
            id: recipe_id.to_string(),
        })?;

        let properties = sqlx::query_as!(
            Property,
            r#"
            SELECT DISTINCT fp.property_id as "id", fp.name
            FROM resolved_recipes r
            JOIN ingredient_properties ip ON ip.ingredient_id = r.ingredient_id
            JOIN food_properties fp ON fp.property_id = ip.property_id
            WHERE r.recipe_id = $1
            ORDER BY fp.name
            "#,
            recipe_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(RecipeProperties {
            recipe_id,
            recipe_name,
            properties,
        })
    }
}
