use std::{borrow::Cow, fmt::Display};

use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::types::{PgInterval, PgMoney},
    types::BigDecimal,
};
use tabled::Tabled;

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
    #[serde(
        serialize_with = "crate::util::serialize_money",
        deserialize_with = "crate::util::deserialize_money"
    )]
    pub price: PgMoney,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubRecipe {
    pub subrecipe_id: i32,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: BigDecimal,
    pub is_subrecipe: bool,
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

impl std::string::ToString for RecipeMetaIngredient {
    fn to_string(&self) -> String {
        self.name().to_string()
    }
}
impl std::string::ToString for RecipeIngredient {
    fn to_string(&self) -> String {
        self.ingredient.name().to_string()
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
        unit: Unit,
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
            unit.unit_id,
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

    pub async fn fetch_subrecipes(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> eyre::Result<Vec<SubRecipe>> {
        let subrecipes = sqlx::query_as!(
            SubRecipe,
            r#"
                SELECT
                    recipe as "recipe!",
                    ingredient as "ingredient!",
                    round(weight * $2, 10)  as "weight!",
                    subrecipe as "subrecipe!",
                    is_subrecipe as "is_subrecipe!",
                    subrecipe_id as "subrecipe_id!"
                FROM subrecipes
                WHERE recipe_id = $1
                ORDER BY recipe, subrecipe_id, ingredient

            "#,
            recipe_id,
            weight,
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(subrecipes)
    }

    pub async fn fetch_subrecipes_from_user_input(
        &self,
        recipe_id: i32,
        people: f64,
        calories: u32,
    ) -> eyre::Result<Vec<SubRecipe>> {
        let total_calories = BigDecimal::from((calories as f64 * people) as u64);
        let weight = self
            .calc_energy_to_weight(recipe_id, total_calories)
            .await
            .unwrap_or_default();
        self.fetch_subrecipes(recipe_id, weight).await
    }

    //pub async fn fetch_subrecipes_from_meal(&self, meal_id: i32) -> eyre::Result<()> {
    //    let meal = self.get_meal
    //    let weight = meal.weight;
    //    let recipe_id = meal.recipe_id;
    //    self.fetch_subrecipes(recipe_id, weight).await
    //}

    pub async fn calc_energy_to_weight(
        &self,
        recipe_id: i32,
        energy: BigDecimal,
    ) -> eyre::Result<BigDecimal> {
        let recipe_stats = sqlx::query!(
            r#"
                SELECT
                    weight, energy
                    FROM recipe_stats
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        let recipe_weight = recipe_stats.weight.unwrap();
        let recipe_energy = recipe_stats.energy.unwrap();
        Ok(recipe_weight / recipe_energy * energy)
    }

    pub async fn format_subrecipes_markdown(&self, subrecipes: Vec<SubRecipe>) -> String {
        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();
        keys.dedup();

        let mut subrecipe_markdown = Vec::new();
        for subrecipe_id in keys {
            let mut text = String::new();
            let ingredients: Vec<_> = subrecipes
                .iter()
                .filter(|sr| sr.subrecipe_id == subrecipe_id)
                .collect();
            let steps = self
                .get_recipe_steps(subrecipe_id)
                .await
                .unwrap_or_default();
            let title = ingredients
                .first()
                .ok_or(eyre::eyre!("No subrecipe provided"))
                .unwrap()
                .subrecipe
                .clone();
            text.push_str(&format!("# {}\n", title));
            let weight: BigDecimal = ingredients
                .iter()
                .map(|ingredient| ingredient.weight.clone())
                .sum();

            for ingredient in ingredients {
                text.push_str(&format!(
                    "* {:.3}kg {}\n",
                    ingredient.weight, ingredient.ingredient
                ));
            }

            if !steps.is_empty() {
                for (i, step) in steps.into_iter().enumerate() {
                    fn to_minutes(duration: PgInterval) -> f64 {
                        duration.microseconds as f64 / 1_000_000. / 60.
                    }
                    let fixed_duration = to_minutes(step.fixed_duration);
                    let duration_per_kg = to_minutes(step.duration_per_kg);
                    let scaled_duration = duration_per_kg * weight.to_f64().unwrap_or_default();
                    let duration = fixed_duration + scaled_duration;

                    text.push_str(&format!(
                        "## {}. {} ({:.3} + {:.3} = {:.3} min)\n{}\n",
                        i + 1,
                        step.step_name,
                        fixed_duration,
                        scaled_duration,
                        duration,
                        step.step_description
                    ));
                }
            }
            subrecipe_markdown.push(text);
        }
        subrecipe_markdown.join("\n")
    }

    pub async fn format_subrecipes_latex(&self, subrecipes: Vec<SubRecipe>) -> String {
        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();
        keys.dedup();

        let mut text = r"
            \documentclass[11pt,a4paper]{article}


            \usepackage[ngerman]{babel}
            \usepackage{ifxetex}

            \ifxetex
              \usepackage{fontspec}
            \else
              \usepackage[T1]{fontenc}
              \usepackage[utf8]{inputenc}
              \usepackage{lmodern}
            \fi

            \usepackage{gensymb}

            \usepackage{recipe}

            \begin{document}
            "
        .to_owned();

        for subrecipe_id in keys {
            let ingredients: Vec<_> = subrecipes
                .iter()
                .filter(|sr| sr.subrecipe_id == subrecipe_id)
                .collect();
            let steps = self
                .get_recipe_steps(subrecipe_id)
                .await
                .unwrap_or_default();
            self.format_subrecipe(&mut text, ingredients, steps)
                .unwrap_or_else(|e| log::error!("{e}"));
        }
        text.push_str("\\end{document}");
        text
    }

    pub async fn format_recipe_latex_from_user_input(
        &self,
        recipe_id: i32,
        people: f64,
        energy: u32,
    ) -> eyre::Result<String> {
        let subrecipes = self
            .fetch_subrecipes_from_user_input(recipe_id, people, energy)
            .await?;
        Ok(self.format_subrecipes_latex(subrecipes).await)
    }

    // TODO Should probabyl use fetch_subrecipes and format_subrecipes_latex
    pub async fn save_recipe_export(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> Result<(), eyre::Error> {
        use std::io::Write;
        let text = self
            .format_subrecipes_latex(self.fetch_subrecipes(recipe_id, weight).await?)
            .await;

        let title = self
            .get_recipe_from_string_reference(recipe_id.to_string())
            .await
            .unwrap()
            .name;

        #[cfg(feature = "tectonic")]
        {
            let pdf_data = compile_pdf(text).await?;
            println!("Output PDF size is {} bytes", pdf_data.len());

            let create_result = std::fs::create_dir("recipes/out");
            if let Err(e) = create_result {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(eyre::eyre!("failed to create output directory: {}", e));
                }
            }
            let mut file = std::fs::File::create(format!("recipes/out/{}.pdf", title))?;
            file.write_all(&pdf_data)?;
        }
        #[cfg(not(feature = "tectonic"))]
        {
            let mut file = std::fs::File::create(format!(
                "recipes/{}.tex",
                title.replace(' ', "_").to_lowercase()
            ))
            .unwrap();
            file.write_all(text.as_bytes()).unwrap();
        }
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

    pub async fn insert_recipe(&self, recipe: &Recipe) -> eyre::Result<Recipe> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"
                INSERT INTO recipes (name, comment)
                VALUES ($1, $2)
                RETURNING *
            "#,
            recipe.name,
            recipe.comment,
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(recipe)
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

    pub fn format_subrecipe(
        &self,
        text: &mut String,
        subrecipes: Vec<&SubRecipe>,
        steps: Vec<RecipeStep>,
    ) -> Result<(), eyre::Error> {
        let title = escape_underscore(
            &subrecipes
                .first()
                .ok_or(eyre::eyre!("No subrecipe provided"))?
                .subrecipe,
        );
        let ingredients: Vec<_> = subrecipes.iter().filter(|sr| !sr.is_subrecipe).collect();
        let meta_ingredients: Vec<_> = subrecipes.iter().filter(|sr| sr.is_subrecipe).collect();
        let weight: BigDecimal = ingredients
            .iter()
            .map(|ingredient| ingredient.weight.clone())
            .sum();

        fn escape_underscore(s: &str) -> String {
            s.replace('_', " ")
        }
        use std::fmt::Write;
        writeln!(text, "\\addrecipe{{{title}}}")?;
        for ingredient in meta_ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight.round(3)
            )?;
        }
        for ingredient in ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight.round(3)
            )?;
        }
        for step in steps {
            fn to_minutes(duration: PgInterval) -> f64 {
                duration.microseconds as f64 / 1_000_000. / 60.
            }
            let duration = to_minutes(step.duration_per_kg) * weight.to_f64().unwrap_or_default()
                + to_minutes(step.fixed_duration);
            writeln!(
                text,
                "\\addstep{{{}}}{{{}}}{{{}}}{{{:.3} min}}",
                title, step.step_name, step.step_description, duration
            )?;
        }
        writeln!(text, "\\printrecipe{{{title}}}")?;
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
#[cfg(not(feature = "tectonic"))]
pub async fn compile_pdf(_: String) -> Result<Vec<u8>, eyre::ErrReport> {
    Err(eyre::eyre!("tectonic feature not enabled"))
}

#[cfg(feature = "tectonic")]
pub async fn compile_pdf(text: String) -> Result<Vec<u8>, eyre::ErrReport> {
    use std::path::Path;
    use tectonic::driver::ProcessingSessionBuilder;
    use tectonic::status;
    use tokio::task::spawn_blocking;
    let mut status = status::NoopStatusBackend::default();
    let mut files = {
        spawn_blocking(move || {
            let auto_create_config_file = false;
            let config = tectonic::config::PersistentConfig::open(auto_create_config_file).unwrap();

            let only_cached = false;
            let bundle = config.default_bundle(only_cached, &mut status).unwrap();

            let format_cache_path = config.format_cache_path().unwrap();

            let mut sb = ProcessingSessionBuilder::default();
            sb.filesystem_root(Path::new("../recipes"))
                .primary_input_buffer(text.as_bytes())
                .tex_input_name("texput.tex")
                .format_name("latex")
                .keep_logs(false)
                .keep_intermediates(false)
                .format_cache_path(format_cache_path)
                .bundle(bundle)
                .do_not_write_output_files()
                .print_stdout(false);
            let mut sess = sb
                .create(&mut status)
                .expect("failed to initialize the LaTeX processing session");
            if let Err(e) = sess.run(&mut status) {
                log::error!("failed to run the LaTeX processing session: {}", e);
            }
            sess.into_file_data()
        })
        .await?
    };
    let Some(pdf) = files.remove("texput.pdf") else {
        return Err(eyre::eyre!(
            "LaTeX didn't report failure, but no PDF was created (??)"
        ));
    };
    let pdf_data = pdf.data;
    Ok(pdf_data)
}
