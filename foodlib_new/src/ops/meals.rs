use crate::{
    entities::meal::*,
    error::{Error, Result},
};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct MealOps {
    pool: Arc<PgPool>,
}

impl MealOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get all meals for a given event.
    pub async fn get_event_meals(&self, event_id: i32) -> Result<Vec<Meal>> {
        let meals = sqlx::query_as!(
            Meal,
            r#"
            SELECT
                event_meals.meal_id,
                event_meals.event_id,
                event_meals.recipe_id,
                recipes.name as "name!",
                event_meals.place_id,
                places.name as "place!",
                event_meals.start_time,
                event_meals.end_time,
                round(sum(event_ingredients.weight), 2) as "weight!",
                round(sum(event_ingredients.energy) / event_meals.servings, 0) as "energy!",
                sum(event_ingredients.price) as "price!",
                event_meals.servings,
                event_meals.comment
            FROM event_meals
            LEFT JOIN event_ingredients ON event_meals.meal_id = event_ingredients.meal_id
            LEFT JOIN recipes ON event_meals.recipe_id = recipes.recipe_id
            LEFT JOIN places ON event_meals.place_id = places.place_id
            WHERE event_meals.event_id = $1
            GROUP BY event_meals.meal_id, recipes.name, places.name, event_meals.servings
            ORDER BY event_meals.start_time
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(meals)
    }

    /// Get a single meal by its ID.
    pub async fn get_meal(&self, meal_id: i32) -> Result<Meal> {
        let meal = sqlx::query_as!(
            Meal,
            r#"
            SELECT
                event_meals.meal_id,
                event_meals.event_id,
                event_meals.recipe_id,
                recipes.name as "name!",
                event_meals.place_id,
                places.name as "place!",
                event_meals.start_time,
                event_meals.end_time,
                round(sum(event_ingredients.weight), 2) as "weight!",
                round(sum(event_ingredients.energy) / event_meals.servings, 0) as "energy!",
                sum(event_ingredients.price) as "price!",
                event_meals.comment,
                event_meals.servings
            FROM event_meals
            LEFT JOIN event_ingredients ON event_meals.meal_id = event_ingredients.meal_id
            LEFT JOIN recipes ON event_meals.recipe_id = recipes.recipe_id
            LEFT JOIN places ON event_meals.place_id = places.place_id
            WHERE event_meals.meal_id = $1
            GROUP BY event_meals.meal_id, recipes.name, places.name, event_meals.servings
            "#,
            meal_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        meal.ok_or(Error::NotFound {
            entity: "Meal",
            id: meal_id.to_string(),
        })
    }

    /// Get all meals.
    pub async fn get_all_meals(&self) -> Result<Vec<Meal>> {
        let meals = sqlx::query_as!(
            Meal,
            r#"
            SELECT
                event_meals.meal_id,
                event_meals.event_id,
                event_meals.recipe_id,
                recipes.name as "name!",
                event_meals.place_id,
                places.name as "place!",
                event_meals.start_time,
                event_meals.end_time,
                round(sum(event_ingredients.weight), 2) as "weight!",
                event_meals.energy_per_serving as energy,
                sum(event_ingredients.price) as "price!",
                event_meals.comment,
                event_meals.servings
            FROM event_meals
            LEFT JOIN event_ingredients ON event_meals.meal_id = event_ingredients.meal_id
            LEFT JOIN recipes ON event_meals.recipe_id = recipes.recipe_id
            LEFT JOIN places ON event_meals.place_id = places.place_id
            GROUP BY event_meals.meal_id, recipes.name, places.name, event_meals.servings
            ORDER BY event_meals.start_time
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(meals)
    }

    /// Update a single meal.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_meal(
        &self,
        meal_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
        energy: BigDecimal,
        servings: i32,
        comment: Option<String>,
    ) -> Result<()> {
        let count = sqlx::query!(
            r#"
                UPDATE event_meals
                SET recipe_id = $1,
                    place_id = $2,
                    start_time = $3,
                    end_time = $4,
                    energy_per_serving = $5,
                    servings = $6,
                    comment = $7
                WHERE
                    meal_id = $8
            "#,
            recipe_id,
            place_id,
            start_time,
            end_time,
            energy,
            servings,
            comment,
            meal_id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        if count != 1 {
            return Err(Error::NotFound {
                entity: "Meal",
                id: meal_id.to_string(),
            });
        }

        Ok(())
    }

    /// Add a new meal.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_meal(
        &self,
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
        energy: BigDecimal,
        servings: i32,
        comment: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO event_meals (event_id, recipe_id, place_id, start_time, end_time, energy_per_serving, servings, comment)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            event_id,
            recipe_id,
            place_id,
            start_time,
            end_time,
            energy,
            servings,
            comment,
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        Ok(())
    }

    /// Remove a meal.
    pub async fn remove_meal(&self, meal_id: i32) -> Result<()> {
        let count = sqlx::query!(
            r#"
                DELETE FROM event_meals
                WHERE
                    meal_id = $1
            "#,
            meal_id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        if count != 1 {
            return Err(Error::NotFound {
                entity: "Meal",
                id: meal_id.to_string(),
            });
        }

        Ok(())
    }

    /// Remove a meal by its event, recipe, place, and start time.
    pub async fn remove_meal_by_reference(
        &self,
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: OffsetDateTime,
    ) -> Result<()> {
        let count = sqlx::query!(
            r#"
                DELETE FROM event_meals
                WHERE
                    event_id = $1 AND
                    recipe_id = $2 AND
                    place_id = $3 AND
                    start_time = $4
            "#,
            event_id,
            recipe_id,
            place_id,
            start_time,
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        if count != 1 {
            return Err(Error::NotFound {
                entity: "Meal",
                id: format!("{},{},{},{}", event_id, recipe_id, place_id, start_time),
            });
        }

        Ok(())
    }

    /// Get all ingredients for a given meal.
    pub async fn get_meal_ingredients(&self, meal_id: i32) -> Result<Vec<MealIngredient>> {
        let ingredients = sqlx::query_as!(
            MealIngredient,
            r#"
            SELECT
                event_id as "event_id!",
                recipe_id as "recipe_id!",
                ingredient_id as "ingredient_id!",
                ingredient as "ingredient!",
                weight as "weight!",
                energy as "energy!",
                price as "price!",
                servings as "servings!",
                meal_id as "meal_id!",
                subrecipe_hierarchy
            FROM event_ingredients
            WHERE meal_id = $1
            ORDER BY weight DESC
            "#,
            meal_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(ingredients)
    }

    /// Update the meals for a given event.
    pub async fn update_event_meals(
        &self,
        event_id: i32,
        meals: impl Iterator<Item = Meal>,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        async fn insert_meal<'a>(
            executor: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
            event_id: i32,
            meal: Meal,
        ) -> sqlx::Result<()> {
            sqlx::query!(
                r#"
                    INSERT INTO event_meals (event_id, recipe_id, place_id, comment, energy_per_serving, servings, start_time, end_time)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                event_id,
                meal.recipe_id,
                meal.place_id,
                meal.comment,
                meal.energy,
                meal.servings,
                meal.start_time,
                meal.end_time
            )
            .execute(executor)
            .await?
            .rows_affected();

            Ok(())
        }

        let count = sqlx::query!(
            r#"
                DELETE FROM event_meals
                WHERE event_id = $1
            "#,
            event_id,
        )
        .execute(&mut *transaction)
        .await?
        .rows_affected();

        log::debug!("Deleted {} event_meals", count);

        for meal in meals {
            insert_meal(&mut *transaction, event_id, meal).await?;
        }

        transaction.commit().await?;
        Ok(())
    }
}
