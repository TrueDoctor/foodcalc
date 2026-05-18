use crate::entities::allergen::DietaryFlags;
use crate::entities::property::Property;
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
    /// Aggregated allergen properties from all ingredients (including subrecipes).
    pub properties: Vec<Property>,
    /// Derived dietary flags computed from `properties` via `allergens::dietary_flags`.
    pub dietary: DietaryFlags,
}

/// Lighter-weight summary for dish labels and event overviews — just identity + allergens.
#[derive(Debug, Clone)]
pub struct MealAllergenInfo {
    pub meal_id: i32,
    pub recipe_id: i32,
    pub recipe_name: String,
    pub place: String,
    pub servings: i32,
    pub start_time: time::OffsetDateTime,
    pub properties: Vec<Property>,
    pub dietary: DietaryFlags,
}

#[derive(Debug, Clone)]
pub struct EventAllergenInfo {
    pub event_id: i32,
    pub event_name: String,
    pub meals: Vec<MealAllergenInfo>,
}

pub struct ExportOps {
    pool: Arc<PgPool>,
}

impl ExportOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn fetch_meal_recipe(&self, meal_id: i32) -> Result<RecipeInfo> {
        // Avoid `MealOps::get_meal` which aggregates over `event_ingredients` (a deep
        // recursive view) — that query takes ~3 minutes on this DB. We only need
        // recipe_id, start_time, and a scaling weight; weight is derived from
        // `servings * energy_per_serving / recipe_energy`, identical to what the
        // heavy aggregate computes but reading only `event_meals` + `recipe_stats`.
        let row = sqlx::query!(
            r#"
            SELECT recipe_id, start_time, servings, energy_per_serving
            FROM event_meals
            WHERE meal_id = $1
            "#,
            meal_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Meal",
            id: meal_id.to_string(),
        })?;
        let total_energy = BigDecimal::from(row.servings) * row.energy_per_serving;
        let weight = self.calc_energy_to_weight(row.recipe_id, total_energy).await?;
        let format = format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();
        let date = row.start_time.format(&format).unwrap();
        self.fetch_recipe_info(row.recipe_id, weight, date).await
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
        if recipe_energy == BigDecimal::from(0) {
            return Err(Error::Validation { message: format!("recipe {recipe_id} has zero energy") });
        }
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

        let properties = crate::ops::properties::PropertyOps::new(self.pool.clone())
            .get_recipe_properties(recipe_id)
            .await?
            .properties;
        let dietary = crate::ops::allergens::dietary_flags(&properties);

        Ok(RecipeInfo {
            name,
            date,
            subrecipes: recipes,
            properties,
            dietary,
        })
    }

    /// Fetch the allergen/dietary summary for every meal of an event, used to
    /// render dish labels and the event-wide allergen overview.
    pub async fn fetch_event_allergens(&self, event_id: i32) -> Result<EventAllergenInfo> {
        let event_name = sqlx::query_scalar!(
            r#"SELECT event_name FROM events WHERE event_id = $1"#,
            event_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| Error::NotFound {
            entity: "event",
            id: event_id.to_string(),
        })?;

        let rows = sqlx::query!(
            r#"
            SELECT
                em.meal_id as "meal_id!",
                em.recipe_id,
                r.name as recipe_name,
                p.name as place,
                em.servings,
                em.start_time as "start_time: time::OffsetDateTime"
            FROM event_meals em
            JOIN recipes r ON r.recipe_id = em.recipe_id
            JOIN places p ON p.place_id = em.place_id
            WHERE em.event_id = $1
            ORDER BY em.start_time, r.name
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        let property_ops = crate::ops::properties::PropertyOps::new(self.pool.clone());
        let mut meals = Vec::with_capacity(rows.len());
        for row in rows {
            let properties = property_ops
                .get_recipe_properties(row.recipe_id)
                .await
                .map(|rp| rp.properties)
                .unwrap_or_default();
            let dietary = crate::ops::allergens::dietary_flags(&properties);
            meals.push(MealAllergenInfo {
                meal_id: row.meal_id,
                recipe_id: row.recipe_id,
                recipe_name: row.recipe_name,
                place: row.place,
                servings: row.servings,
                start_time: row.start_time,
                properties,
                dietary,
            });
        }

        Ok(EventAllergenInfo {
            event_id,
            event_name,
            meals,
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
