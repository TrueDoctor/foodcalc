use crate::entities::recipe::{RecipeStep, SubRecipe};
use crate::error::{Error, Result};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use time::format_description;

pub struct RecipeInfo {
    pub name: String,
    pub date: String,
    pub subrecipes: Vec<(Vec<SubRecipe>, Vec<RecipeStep>)>,
}

pub struct ExportOps {
    pool: Arc<PgPool>,
}

impl ExportOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn fetch_meal_recipe(&self, meal_id: i32) -> Result<RecipeInfo> {
        let meal = crate::ops::meals::MealOps::new(self.pool.clone())
            .get_meal(meal_id)
            .await?;
        let recipe_id = meal.recipe_id;
        let weight = meal.weight;
        let format = format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();
        let date = meal.start_time.format(&format).unwrap();
        self.fetch_recipe_info(recipe_id, weight, date).await
    }

    pub async fn fetch_food_prep_recipe(&self, prep_id: i32) -> Result<RecipeInfo> {
        let prep = sqlx::query_as!(
            crate::entities::event::FoodPrep,
            "SELECT prep_id as id, event_id, recipe_id, prep_date, use_from, use_until FROM food_prep WHERE prep_id = $1",
            prep_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "FoodPrep",
            id: prep_id.to_string(),
        })?;

        let weight = sqlx::query!(
            r#"SELECT COALESCE(SUM(weight), 0) as "total_weight!" FROM prep_ingredients_with_duplicates WHERE prep_id = $1"#,
            prep_id
        )
        .fetch_one(&*self.pool)
        .await?
        .total_weight;

        let format = format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();
        let date = prep.prep_date.format(&format).unwrap();

        self.fetch_recipe_info(prep.recipe_id, weight, date).await
    }

    pub async fn fetch_subrecipes(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> Result<Vec<SubRecipe>> {
        let mut subrecipes = sqlx::query_as!(
            SubRecipe,
            r#"
            SELECT
                recipe as "recipe!",
                recipe_id as "recipe_id!",
                ingredient as "ingredient!",
                round(weight * $1, 10) as "weight!",
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
        .fetch_all(&*self.pool)
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

    pub async fn fetch_user_input_meal(
        &self,
        recipe_id: i32,
        people: f64,
        calories: u32,
        date: String,
    ) -> Result<RecipeInfo> {
        let total_calories = BigDecimal::from((calories as f64 * people) as u64);
        let weight = self.calc_energy_to_weight(recipe_id, total_calories).await?;
        self.fetch_recipe_info(recipe_id, weight, date).await
    }

    pub async fn fetch_subrecipes_from_user_input(
        &self,
        recipe_id: i32,
        people: f64,
        calories: u32,
    ) -> Result<Vec<SubRecipe>> {
        let total_calories = BigDecimal::from((calories as f64 * people) as u64);
        let weight = self.calc_energy_to_weight(recipe_id, total_calories).await?;
        self.fetch_subrecipes(recipe_id, weight).await
    }

    async fn calc_energy_to_weight(&self, recipe_id: i32, energy: BigDecimal) -> Result<BigDecimal> {
        let recipe_stats = sqlx::query!(
            r#"SELECT weight, energy FROM recipe_stats WHERE recipe_id = $1"#,
            recipe_id,
        )
        .fetch_one(&*self.pool)
        .await?;

        let recipe_weight = recipe_stats.weight.ok_or_else(|| Error::NotFound { entity: "recipe_stats.weight", id: recipe_id.to_string() })?;
        let recipe_energy = recipe_stats.energy.ok_or_else(|| Error::NotFound { entity: "recipe_stats.energy", id: recipe_id.to_string() })?;
        Ok(recipe_weight / recipe_energy * energy)
    }

    async fn fetch_recipe_info(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
        date: String,
    ) -> Result<RecipeInfo> {
        let mut subrecipes = self.fetch_subrecipes(recipe_id, weight).await?;
        sort_subrecipes_topologically(&mut subrecipes);

        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();
        let mut seen: HashSet<i32> = HashSet::new();
        keys.retain(|&x| seen.insert(x));

        let recipe_ops = crate::ops::recipes::RecipeOps::new(self.pool.clone());
        let mut recipes = Vec::with_capacity(keys.len());
        for id in keys {
            let ingredients = subrecipes
                .iter()
                .filter(|sr| sr.subrecipe_id == id)
                .cloned()
                .collect::<Vec<SubRecipe>>();
            let steps = recipe_ops.get_steps(id).await.unwrap_or_default();
            recipes.push((ingredients, steps));
        }

        let name = subrecipes
            .first()
            .ok_or_else(|| Error::Misc("No subrecipe provided".into()))?
            .subrecipe
            .clone();

        Ok(RecipeInfo {
            name,
            date,
            subrecipes: recipes,
        })
    }
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
