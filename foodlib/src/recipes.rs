use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgInterval, types::BigDecimal};
use tabled::Tabled;

pub mod export;

use crate::{
    ingredients::{Ingredient, Unit},
    FoodBase,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct Recipe {
    pub recipe_id: i32,
    pub name: String,
    #[tabled(display_with = "crate::util::display_optional")]
    pub comment: Option<String>,
    pub owner_id: i64,
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecipeIngredient {
    pub ingredient_id: i32,
    pub name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: BigDecimal,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubRecipe {
    pub subrecipe_id: i32,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: BigDecimal,
    pub is_subrecipe: bool,
    pub recipe_id: i32,
    pub subrecipe_hierarchy: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecipeMetaIngredient {
    Ingredient(Ingredient),
    MetaRecipe(Recipe),
}

impl Default for RecipeMetaIngredient {
    fn default() -> Self {
        Self::Ingredient(Ingredient::default())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub ingredient: RecipeMetaIngredient,
    pub amount: BigDecimal,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeStep {
    pub step_id: i32,
    pub step_order: f64,
    pub step_name: String,
    pub step_description: String,
    #[serde(
        serialize_with = "crate::util::serialize_interval",
        deserialize_with = "crate::util::deserialize_interval"
    )]
    pub fixed_duration: PgInterval,
    #[serde(
        serialize_with = "crate::util::serialize_interval",
        deserialize_with = "crate::util::deserialize_interval"
    )]
    pub duration_per_kg: PgInterval,
    pub recipe_id: i32,
}

impl Default for RecipeStep {
    fn default() -> Self {
        Self {
            step_id: Default::default(),
            step_order: Default::default(),
            step_name: Default::default(),
            step_description: Default::default(),
            fixed_duration: PgInterval::try_from(std::time::Duration::from_secs(0)).unwrap(),
            duration_per_kg: PgInterval::try_from(std::time::Duration::from_secs(0)).unwrap(),
            recipe_id: Default::default(),
        }
    }
}

impl RecipeMetaIngredient {
    pub fn name(&self) -> &str {
        match self {
            RecipeMetaIngredient::Ingredient(ingredient) => &ingredient.name,
            RecipeMetaIngredient::MetaRecipe(recipe) => &recipe.name,
        }
    }
}

impl Display for RecipeMetaIngredient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Display for RecipeIngredient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ingredient.name())
    }
}

impl FoodBase {
    pub async fn get_recipe_ingredients(
        &self,
        recipe_id: i32,
    ) -> eyre::Result<Vec<RecipeIngredient>> {
        struct RecipeIngredientWeight {
            ingredient_id: i32,
            name: String,
            comment: Option<String>,
            energy: BigDecimal,
            amount: BigDecimal,
            unit_id: i32,
            unit_name: String,
        }
        let records = sqlx::query_as!(
            RecipeIngredientWeight,
            r#" SELECT ingredient_id, ingredients.name, energy, comment, amount, unit_id, units.name as "unit_name!"
                FROM recipe_ingredients
                JOIN ingredients USING(ingredient_id)
                JOIN units USING(unit_id)
                WHERE recipe_ingredients.recipe_id = $1
                ORDER BY ingredient_id  "#,
            recipe_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        let records = records
            .into_iter()
            .map(
                |RecipeIngredientWeight {
                     ingredient_id,
                     name,
                     comment,
                     energy,
                     unit_name,
                     unit_id,
                     amount,
                 }| RecipeIngredient {
                    ingredient: RecipeMetaIngredient::Ingredient(Ingredient {
                        ingredient_id,
                        name,
                        comment,
                        energy,
                        owner_id: -1,
                    }),
                    amount,
                    unit: Unit {
                        name: Cow::Owned(unit_name),
                        unit_id,
                    },
                },
            )
            .collect();

        Ok(records)
    }

    pub async fn get_recipe_meta_ingredients(
        &self,
        recipe_id: i32,
    ) -> eyre::Result<Vec<RecipeIngredient>> {
        struct RecipeIngredientWeight {
            recipe_id: i32,
            name: String,
            comment: Option<String>,
            weight: BigDecimal,
        }
        let records = sqlx::query_as!(
            RecipeIngredientWeight,
            r#" SELECT recipe_id, name,  comment, weight as "weight!"
                FROM meta_recipes
                JOIN recipes ON(recipe_id = child_id)
                WHERE parent_id = $1
                ORDER BY recipe_id  "#,
            recipe_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        let records = records
            .into_iter()
            .map(
                |RecipeIngredientWeight {
                     recipe_id,
                     name,
                     comment,
                     weight,
                 }| RecipeIngredient {
                    ingredient: RecipeMetaIngredient::MetaRecipe(Recipe {
                        owner_id: -1,
                        recipe_id,
                        name,
                        comment,
                    }),
                    amount: weight,
                    unit: Unit {
                        name: Cow::Borrowed("kg"),
                        unit_id: 0,
                    },
                },
            )
            .collect();

        Ok(records)
    }

    pub async fn add_recipe_ingredient(
        &self,
        recipe_id: i32,
        ingredient_id: i32,
        amount: BigDecimal,
        unit_id: i32,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id)
                VALUES ($1, $2, $3, $4)
            "#,
            recipe_id,
            ingredient_id,
            amount,
            unit_id,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Inserted {} recipe_ingredients", count.rows_affected());
        Ok(())
    }

    pub async fn update_recipe_ingredient(
        &self,
        recipe_id: i32,
        ingredient_id: i32,
        amount: BigDecimal,
        unit: i32,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                UPDATE recipe_ingredients
                SET amount = $3, unit_id = $4
                WHERE recipe_id = $1 AND ingredient_id = $2
            "#,
            recipe_id,
            ingredient_id,
            amount,
            unit,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Updated {} recipe_ingredients", count.rows_affected());
        Ok(())
    }

    pub async fn delete_recipe_ingredient(
        &self,
        recipe_id: i32,
        ingredient_id: i32,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                DELETE FROM recipe_ingredients
                WHERE recipe_id = $1 AND ingredient_id = $2
            "#,
            recipe_id,
            ingredient_id,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Deleted {} recipe_ingredients", count.rows_affected());
        Ok(())
    }

    pub async fn delete_recipe_meta_ingredient(
        &self,
        recipe_id: i32,
        meta_recipe_id: i32,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                DELETE FROM meta_recipes
                WHERE parent_id = $1 AND child_id = $2
            "#,
            recipe_id,
            meta_recipe_id,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Deleted {} meta_recipes", count.rows_affected());
        Ok(())
    }

    pub async fn add_recipe_meta_ingredient(
        &self,
        recipe_id: i32,
        meta_recipe_id: i32,
        weight: BigDecimal,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                INSERT INTO meta_recipes (parent_id, child_id, weight)
                VALUES ($1, $2, $3)
            "#,
            recipe_id,
            meta_recipe_id,
            weight,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Inserted {} meta_recipes", count.rows_affected());
        Ok(())
    }

    pub async fn update_recipe_meta_ingredient(
        &self,
        recipe_id: i32,
        meta_recipe_id: i32,
        weight: BigDecimal,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                UPDATE meta_recipes
                SET weight = $3
                WHERE parent_id = $1 AND child_id = $2
            "#,
            recipe_id,
            meta_recipe_id,
            weight,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Updated {} meta_recipes", count.rows_affected());
        Ok(())
    }

    pub async fn add_recipe_step(&self, step: &RecipeStep) -> eyre::Result<RecipeStep> {
        let step = sqlx::query_as!(
            RecipeStep,
            r#"
                INSERT INTO steps (step_order, step_name, step_description, recipe_id, fixed_duration, duration_per_kg)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
            "#,
            step.step_order,
            step.step_name,
            step.step_description,
            step.recipe_id,
            step.fixed_duration,
            step.duration_per_kg,
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(step)
    }

    pub async fn update_recipe_step(&self, step: &RecipeStep) -> eyre::Result<RecipeStep> {
        let step = sqlx::query_as!(
            RecipeStep,
            r#"
                UPDATE steps
                SET step_order = $1, step_name = $2, step_description = $3, fixed_duration = $4, duration_per_kg = $5
                WHERE step_id = $6
                RETURNING *
            "#,
            step.step_order,
            step.step_name,
            step.step_description,
            step.fixed_duration,
            step.duration_per_kg,
            step.step_id,
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(step)
    }

    pub async fn delete_step(&self, recipe_id: i32, step_id: i32) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                DELETE FROM steps
                WHERE recipe_id = $1 AND step_id = $2
            "#,
            recipe_id,
            step_id,
        )
        .execute(&*self.pg_pool)
        .await?;
        log::debug!("Deleted {} steps", count.rows_affected());
        Ok(())
    }

    pub async fn delete_recipe(&self, recipe_id: i32) -> eyre::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        let count = sqlx::query!(
            r#"
                DELETE FROM recipe_ingredients
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} recipe_ingredients", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM meta_recipes
                WHERE parent_id = $1 OR child_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} meta_recipes", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM steps
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} steps", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM event_meals
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} event_meals", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM recipes
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} recipes", count.rows_affected());

        transaction.commit().await?;
        Ok(())
    }

    pub async fn update_recipe(&self, recipe: &Recipe) -> eyre::Result<Recipe> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"
                UPDATE recipes
                SET name = $1, comment = $2
                WHERE recipe_id = $3
                RETURNING *
            "#,
            recipe.name,
            recipe.comment,
            recipe.recipe_id,
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(recipe)
    }

    pub async fn add_recipe(&self, name: &str, comment: &Option<String>) -> eyre::Result<Recipe> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"
                INSERT INTO recipes (name, comment)
                VALUES ($1, $2)
                RETURNING *
            "#,
            name,
            comment.clone(),
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(recipe)
    }

    pub async fn insert_recipe(&self, recipe: &Recipe) -> eyre::Result<Recipe> {
        self.add_recipe(&recipe.name, &recipe.comment).await
    }

    // TODO: Human race condition, add proper locking / edit notifications
    pub async fn update_recipe_entries(
        &self,
        recipe_id: i32,
        entries: impl Iterator<Item = RecipeIngredient>,
    ) -> eyre::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        pub async fn insert_recipe_entry<'a>(
            executor: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
            recipe_id: i32,
            entry: RecipeIngredient,
        ) -> sqlx::Result<()> {
            let count = match entry.ingredient {
                RecipeMetaIngredient::Ingredient(ingredient) => sqlx::query!(
                    r#"
                            INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id)
                            VALUES ($1, $2, $3, $4)
                        "#,
                    recipe_id,
                    ingredient.ingredient_id,
                    entry.amount,
                    entry.unit.unit_id,
                )
                .execute(executor)
                .await?
                .rows_affected(),
                RecipeMetaIngredient::MetaRecipe(recipe) => sqlx::query!(
                    r#"
                            INSERT INTO meta_recipes (parent_id, child_id, weight)
                            VALUES ($1, $2, $3)
                        "#,
                    recipe_id,
                    recipe.recipe_id,
                    entry.amount,
                )
                .execute(executor)
                .await?
                .rows_affected(),
            };
            assert_eq!(count, 1);

            Ok(())
        }

        let count = sqlx::query!(
            r#"
                DELETE FROM recipe_ingredients
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} recipe_ingredients", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM meta_recipes
                WHERE parent_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} meta_recipes", count.rows_affected());

        for entry in entries {
            insert_recipe_entry(&mut *transaction, recipe_id, entry).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    // TODO: Human race condition, add proper locking / edit notifications
    pub async fn update_recipe_steps(
        &self,
        recipe_id: i32,
        entries: impl Iterator<Item = RecipeStep>,
    ) -> eyre::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        pub async fn insert_recipe_step<'a>(
            executor: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
            recipe_id: i32,
            entry: RecipeStep,
        ) -> sqlx::Result<()> {
            let count = sqlx::query!(
                r#"
                            INSERT INTO steps (step_order, step_name, step_description, recipe_id, fixed_duration, duration_per_kg)
                            VALUES ($1, $2, $3, $4, $5, $6)
                        "#,
                entry.step_order,
                entry.step_name,
                entry.step_description,
                recipe_id,
                entry.fixed_duration,
                entry.duration_per_kg,
            )
            .execute(executor)
            .await?
            .rows_affected();
            assert_eq!(count, 1);

            Ok(())
        }

        let count = sqlx::query!(
            r#"
                DELETE FROM steps
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut *transaction)
        .await?;
        log::debug!("Deleted {} steps", count.rows_affected());

        for entry in entries {
            insert_recipe_step(&mut *transaction, recipe_id, entry).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_recipe_steps(&self, recipe_id: i32) -> eyre::Result<Vec<RecipeStep>> {
        let mut conn = self.pg_pool.acquire().await?;
        let steps = sqlx::query_as!(
            RecipeStep,
            r#"
            SELECT
                step_id,
                step_order,
                step_name,
                step_description,
                fixed_duration,
                duration_per_kg,
                recipe_id
            FROM steps
            WHERE recipe_id = $1
            ORDER BY step_order
            "#,
            recipe_id
        )
        .fetch_all(&mut *conn)
        .await?;
        Ok(steps)
    }

    pub async fn get_recipes(&self) -> eyre::Result<Vec<Recipe>> {
        let records = sqlx::query_as!(Recipe, r#" SELECT * FROM recipes ORDER BY recipe_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?;

        Ok(records)
    }

    pub async fn get_recipe(&self, recipe_id: i32) -> eyre::Result<Recipe> {
        let records = sqlx::query_as!(
            Recipe,
            r#" SELECT * FROM recipes WHERE recipe_id = $1 ORDER BY recipe_id "#,
            recipe_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(records)
    }

    pub async fn get_recipe_from_string_reference(&self, reference: String) -> Option<Recipe> {
        let recipe_id = reference.parse::<i32>().unwrap_or(-1);

        let records = sqlx::query_as!(
            Recipe,
            r#" 
                SELECT * FROM recipes 
                WHERE recipe_id = $1 OR name = $2
                ORDER BY recipe_id
            "#,
            recipe_id,
            reference
        )
        .fetch_one(&*self.pg_pool)
        .await;

        records.ok()
    }
}
