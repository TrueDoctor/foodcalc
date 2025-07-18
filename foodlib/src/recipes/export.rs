use std::collections::HashSet;

use super::RecipeStep;

use bigdecimal::ToPrimitive;
use sqlx::postgres::types::PgInterval;

use sqlx;
use time::format_description;

use super::SubRecipe;

use sqlx::types::BigDecimal;

use crate::{FoodBase, FoodPrep};

pub struct RecipeInfo {
    pub name: String,
    pub date: String,
    pub subrecipes: Vec<(Vec<SubRecipe>, Vec<RecipeStep>)>,
}

impl FoodBase {
    pub async fn fetch_subrecipes(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> eyre::Result<Vec<SubRecipe>> {
        let mut subrecipes = sqlx::query_as!(
            SubRecipe,
            r#"
                SELECT
                    recipe as "recipe!",
                    recipe_id as "recipe_id!",
                    ingredient as "ingredient!",
                    round(weight * $1, 10)  as "weight!",
                    subrecipe as "subrecipe!",
                    is_subrecipe as "is_subrecipe!",
                    subrecipe_id as "subrecipe_id!",
                    subrecipe_hierarchy
                FROM subrecipes as s
                WHERE recipe_id = $2
                ORDER BY recipe, subrecipe_id, ingredient

            "#,
            weight,
            recipe_id,
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        subrecipes.sort_by_key(|s| {
            s.subrecipe_hierarchy
                .clone()
                .unwrap_or_default()
                .chars()
                .filter(|x| *x == '.')
                .count()
        });
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

    pub async fn fetch_user_input_meal(
        &self,
        recipe_id: i32,
        people: f64,
        calories: u32,
        date: String,
    ) -> eyre::Result<RecipeInfo> {
        let total_calories = BigDecimal::from((calories as f64 * people) as u64);
        let weight = self
            .calc_energy_to_weight(recipe_id, total_calories)
            .await
            .unwrap_or_default();
        self.fetch_recipes_infos(recipe_id, weight, date).await
    }

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

    pub async fn fetch_meal_recipe(&self, meal_id: i32) -> eyre::Result<RecipeInfo> {
        let meal = self.get_event_meal(meal_id).await?;
        let recipe_id = meal.recipe_id;
        let weight = meal.weight;
        let format = format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();
        let date = meal.start_time.format(&format).unwrap();
        self.fetch_recipes_infos(recipe_id, weight, date).await
    }

    pub async fn fetch_food_prep_recipe(&self, prep_id: i32) -> eyre::Result<RecipeInfo> {
        let prep = self.get_food_prep(prep_id).await?;
        let recipe_id = prep.recipe_id;

        // Get the total weight from prep ingredients
        let weight = self.get_food_prep_total_weight(prep_id).await?;

        let format = format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();
        let date = prep.prep_date.format(&format).unwrap();

        self.fetch_recipes_infos(recipe_id, weight, date).await
    }

    async fn get_food_prep_total_weight(&self, prep_id: i32) -> eyre::Result<BigDecimal> {
        let total_weight = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(weight), 0) as "total_weight!"
            FROM prep_ingredients_with_duplicates
            WHERE prep_id = $1
            "#,
            prep_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(total_weight.total_weight)
    }

    async fn get_food_prep(&self, prep_id: i32) -> eyre::Result<FoodPrep> {
        let prep = sqlx::query_as!(
            FoodPrep,
            "SELECT * FROM food_prep WHERE prep_id = $1",
            prep_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(prep)
    }

    async fn fetch_recipes_infos(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
        date: String,
    ) -> eyre::Result<RecipeInfo> {
        let mut subrecipes = self.fetch_subrecipes(recipe_id, weight).await?;
        sort_subrecipes_topologically(&mut subrecipes);

        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();

        let mut seen: HashSet<i32> = HashSet::new();
        keys.retain(|&x| seen.insert(x));

        let mut recipes = Vec::<(Vec<SubRecipe>, Vec<RecipeStep>)>::with_capacity(keys.len());
        for id in keys {
            let ingredients = subrecipes
                .iter()
                .filter(|sr| sr.subrecipe_id == id)
                .map(|sr| sr.to_owned())
                .collect::<Vec<SubRecipe>>();
            let steps = self.get_recipe_steps(id).await.unwrap_or_default();
            recipes.insert(recipes.len(), (ingredients, steps));
        }

        let name = subrecipes
            .first()
            .ok_or(eyre::eyre!("No subrecipe provided"))?
            .subrecipe
            .to_owned();
        Ok(RecipeInfo {
            name,
            date,
            subrecipes: recipes,
        })
    }
    //pub async fn fetch_recipe_info(&self, recipe_id: i32, weight: BigDecimal) -> eyre::Result<Vec<RecipeInfo>> {
    //    let subrecipes =
    //    let title =
    //
    //}

    pub async fn format_subrecipes_markdown(&self, mut subrecipes: Vec<SubRecipe>) -> String {
        sort_subrecipes_topologically(&mut subrecipes);
        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();

        let mut seen: HashSet<i32> = HashSet::new();
        keys.retain(|&x| seen.insert(x));

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

            format_recipe_markdown(ingredients, &mut text, steps, weight);
            subrecipe_markdown.push(text);
        }
        subrecipe_markdown.join("\n")
    }

    // pub async fn generate_recipes_typst(&self, subrecipes: &[SubRecipe]) -> eyre::Result<Vec<u8>> {
    //        crate::typst::export_recipes(subrecipes, self).await
    //    }
}

fn sort_subrecipes_topologically(subrecipes: &mut [SubRecipe]) {
    subrecipes.sort_by_key(|s| {
        s.subrecipe_hierarchy
            .clone()
            .unwrap_or_default()
            .split('.')
            .position(|id| id == s.subrecipe_id.to_string())
            .map(|x| x + 1)
            .unwrap_or(0)
    });
}

fn format_recipe_markdown(
    ingredients: Vec<&SubRecipe>,
    text: &mut String,
    steps: Vec<RecipeStep>,
    weight: BigDecimal,
) {
    for ingredient in ingredients {
        text.push_str(&format!(
            "* {:.3}kg {}\n",
            ingredient.weight, ingredient.ingredient
        ));
    }

    if !steps.is_empty() {
        for (i, step) in steps.into_iter().enumerate() {
            pub(crate) fn to_minutes(duration: PgInterval) -> f64 {
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
}
