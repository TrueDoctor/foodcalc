use crate::entities::allergen::DietaryFlags;
use crate::entities::property::Property;
use crate::entities::recipe::{RecipeStep, SubRecipe};
use crate::error::{Error, Result};
use bigdecimal::{BigDecimal, ToPrimitive};
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
    /// True when this sheet represents a food-prep task (cooked ahead of time)
    /// rather than a meal served at the event.
    pub is_prep: bool,
    /// For prep sheets: the meals this prep feeds, as (recipe name, weight share
    /// percentage). Empty for meals and for preps whose downstream meals can't be
    /// resolved.
    pub prep_for: Vec<(String, f64)>,
    /// Subrecipe ids that are prepped ahead for this event. Their own recipe
    /// section is omitted (assumed already made) and their meta-ingredient row is
    /// marked "aus Food Prep". Empty when there is no event/prep context.
    pub prepped_subrecipe_ids: Vec<i32>,
    /// True when the recipe's ingredient amounts have been weighed and verified
    /// accurate to the gram ("eingewogen"); rendered as a badge on the sheet.
    pub weighed_in: bool,
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

/// Prep availability windows for one event, fetched once and queried in memory.
struct PrepWindows {
    /// (prepped recipe id, use_from, use_until).
    windows: Vec<(i32, time::OffsetDateTime, time::OffsetDateTime)>,
}

impl PrepWindows {
    /// Recipe ids whose prep window covers `consumer_time`, excluding
    /// `exclude` (a prep never treats itself as already made).
    fn ids_available_at(
        &self,
        consumer_time: time::OffsetDateTime,
        exclude: Option<i32>,
    ) -> Vec<i32> {
        let mut ids: Vec<i32> = self
            .windows
            .iter()
            .filter(|(id, from, until)| {
                *from <= consumer_time && *until >= consumer_time && Some(*id) != exclude
            })
            .map(|(id, _, _)| *id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }
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
            SELECT event_id, recipe_id, start_time, servings, energy_per_serving
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
        // A meal assumes any prepped subrecipe that is available at its serving
        // time is already made, so those sub-sections are omitted from the print.
        let prepped = self
            .event_prep_windows(row.event_id)
            .await
            .map(|w| w.ids_available_at(row.start_time, None))
            .unwrap_or_default();
        let mut info = self.fetch_recipe_info(row.recipe_id, weight, date).await?;
        info.prepped_subrecipe_ids = prepped;
        Ok(info)
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

        // A prep sheet assumes any *other* prep available at its own prep time is
        // already made — but never itself (that is what this sheet prints), and
        // never a prep made later than this one.
        let prepped = self
            .event_prep_windows(prep.event_id)
            .await
            .map(|w| w.ids_available_at(prep.prep_date, Some(prep.recipe_id)))
            .unwrap_or_default();

        let mut info = self.fetch_recipe_info(prep.recipe_id, weight, date).await?;
        info.is_prep = true;
        info.prep_for = self.fetch_prep_allocation(prep_id).await.unwrap_or_default();
        info.prepped_subrecipe_ids = prepped;
        Ok(info)
    }

    /// Build every recipe sheet for an event — one per meal plus one per food-prep
    /// task — sorted chronologically (preps by prep_date, meals by start_time) so a
    /// single combined PDF reads in kitchen-workflow order.
    pub async fn fetch_event_combined(&self, event_id: i32) -> Result<Vec<RecipeInfo>> {
        let format =
            format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();

        // (sort_key, RecipeInfo). sort_key is the OffsetDateTime driving the order.
        let mut items: Vec<(time::OffsetDateTime, RecipeInfo)> = Vec::new();

        // Fetch prep windows once and resolve attribution in memory for every
        // meal and prep, rather than a query per item.
        let windows = self.event_prep_windows(event_id).await?;

        let meals = crate::ops::meals::MealOps::new(self.pool.clone())
            .get_event_meals(event_id)
            .await?;
        for meal in meals {
            // `meal.weight` is already the scaled total recipe weight (same
            // derivation as calc_energy_to_weight), so reuse it directly.
            let date = meal.start_time.format(&format).unwrap_or_default();
            match self.fetch_recipe_info(meal.recipe_id, meal.weight.clone(), date).await {
                Ok(mut info) => {
                    info.prepped_subrecipe_ids =
                        windows.ids_available_at(meal.start_time, None);
                    items.push((meal.start_time, info));
                }
                // Skip meals whose recipe can't render (e.g. no subrecipes) rather
                // than failing the whole export.
                Err(_) => continue,
            }
        }

        let preps = crate::ops::events::EventOps::new(self.pool.clone())
            .get_food_prep_tasks(event_id)
            .await?;
        // Batch the two expensive recursive-view lookups across all preps once,
        // instead of one query per prep.
        let prep_weights = self.event_prep_weights(event_id).await.unwrap_or_default();
        let prep_allocations = self.event_prep_allocations(event_id).await.unwrap_or_default();
        for prep in preps {
            let weight = prep_weights
                .get(&prep.id)
                .cloned()
                .unwrap_or_else(|| BigDecimal::from(0));
            let date = prep.prep_date.format(&format).unwrap();
            match self.fetch_recipe_info(prep.recipe_id, weight, date).await {
                Ok(mut info) => {
                    info.is_prep = true;
                    info.prep_for = prep_allocations.get(&prep.id).cloned().unwrap_or_default();
                    // Other preps available at this prep's own time, never itself.
                    info.prepped_subrecipe_ids =
                        windows.ids_available_at(prep.prep_date, Some(prep.recipe_id));
                    items.push((prep.prep_date, info));
                }
                Err(_) => continue,
            }
        }

        items.sort_by_key(|(when, _)| *when);
        Ok(items.into_iter().map(|(_, info)| info).collect())
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
                round(COALESCE(weight, 0) * $1, 10) as "weight!",
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

        let weighed_in = recipe_ops.get(recipe_id).await?.weighed_in;

        Ok(RecipeInfo {
            name,
            date,
            subrecipes: recipes,
            properties,
            dietary,
            is_prep: false,
            prep_for: Vec::new(),
            prepped_subrecipe_ids: Vec::new(),
            weighed_in,
        })
    }

    /// All prep availability windows for an event, fetched once so per-consumer
    /// attribution can be resolved in memory (see [`PrepWindows::ids_available_at`])
    /// instead of a query per meal/prep. Uses the same `[use_from, use_until]`
    /// window the shopping-tour views use (`resolved_prep_starts`:
    /// `use_from = COALESCE(use_from, prep_date)`).
    async fn event_prep_windows(&self, event_id: i32) -> Result<PrepWindows> {
        let rows = sqlx::query!(
            r#"
            SELECT recipe_id,
                   COALESCE(use_from, prep_date) as "use_from!",
                   use_until
            FROM food_prep
            WHERE event_id = $1
            "#,
            event_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(PrepWindows {
            windows: rows
                .into_iter()
                .map(|r| (r.recipe_id, r.use_from, r.use_until))
                .collect(),
        })
    }

    /// Total prepped weight per prep for an event, in one query. Avoids the
    /// per-prep `prep_ingredients_with_duplicates` scan (each ~0.6s) in the
    /// combined export. Keyed by prep_id.
    async fn event_prep_weights(
        &self,
        event_id: i32,
    ) -> Result<std::collections::HashMap<i32, BigDecimal>> {
        let rows = sqlx::query!(
            r#"
            SELECT d.prep_id as "prep_id!", COALESCE(SUM(d.weight), 0) as "weight!"
            FROM prep_ingredients_with_duplicates d
            JOIN food_prep fp ON fp.prep_id = d.prep_id
            WHERE fp.event_id = $1
            GROUP BY d.prep_id
            "#,
            event_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| (r.prep_id, r.weight)).collect())
    }

    /// Per-prep downstream meal allocation for an event, in one query over the
    /// recursive prep-attribution view (the export's most expensive query, ~0.85s
    /// each per prep). Returns prep_id -> [(meal recipe name, weight %)].
    async fn event_prep_allocations(
        &self,
        event_id: i32,
    ) -> Result<std::collections::HashMap<i32, Vec<(String, f64)>>> {
        let rows = sqlx::query!(
            r#"
            SELECT pi.prep_id as "prep_id!", r.name as "recipe!",
                   sum(eibptr.weight) as "weight!"
            FROM prep_ingredients pi
            JOIN event_ingredients_before_prep_time_resolve eibptr
              ON eibptr.event_id = pi.event_id
             AND eibptr.ingredient_id = pi.ingredient_id
             AND eibptr.subrecipe_hierarchy = pi.subrecipe_hierarchy
             AND eibptr.meal_id = pi.meal_id
            JOIN event_meals em ON em.meal_id = pi.meal_id
            JOIN recipes r ON r.recipe_id = em.recipe_id
            WHERE pi.event_id = $1
            GROUP BY pi.prep_id, r.name
            "#,
            event_id,
        )
        .fetch_all(&*self.pool)
        .await?;

        // Group by prep, then normalise each prep's rows to percentages.
        let mut raw: std::collections::HashMap<i32, Vec<(String, f64)>> =
            std::collections::HashMap::new();
        for row in rows {
            raw.entry(row.prep_id)
                .or_default()
                .push((row.recipe, row.weight.to_f64().unwrap_or(0.0)));
        }
        let mut out = std::collections::HashMap::new();
        for (prep_id, entries) in raw {
            let total: f64 = entries.iter().map(|(_, w)| w).sum();
            if total <= 0.0 {
                continue;
            }
            let mut alloc: Vec<(String, f64)> = entries
                .into_iter()
                .map(|(name, w)| (name, w / total * 100.0))
                .collect();
            alloc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            out.insert(prep_id, alloc);
        }
        Ok(out)
    }

    /// For a prep task, resolve which event meals consume it and each meal's
    /// share of the prepped weight (as a percentage). Aggregated by downstream
    /// recipe name so multiple servings of the same dish collapse into one entry.
    async fn fetch_prep_allocation(&self, prep_id: i32) -> Result<Vec<(String, f64)>> {
        let rows = sqlx::query!(
            r#"
            SELECT r.name as "recipe!", sum(eibptr.weight) as "weight!"
            FROM prep_ingredients pi
            JOIN event_ingredients_before_prep_time_resolve eibptr
              ON eibptr.event_id = pi.event_id
             AND eibptr.ingredient_id = pi.ingredient_id
             AND eibptr.subrecipe_hierarchy = pi.subrecipe_hierarchy
             AND eibptr.meal_id = pi.meal_id
            JOIN event_meals em ON em.meal_id = pi.meal_id
            JOIN recipes r ON r.recipe_id = em.recipe_id
            WHERE pi.prep_id = $1
            GROUP BY r.name
            "#,
            prep_id
        )
        .fetch_all(&*self.pool)
        .await?;

        let total: f64 = rows.iter().map(|r| r.weight.to_f64().unwrap_or(0.0)).sum();
        if total <= 0.0 {
            return Ok(Vec::new());
        }
        let mut alloc: Vec<(String, f64)> = rows
            .into_iter()
            .map(|r| (r.recipe, r.weight.to_f64().unwrap_or(0.0) / total * 100.0))
            .collect();
        // Largest share first so the most relevant meal reads first.
        alloc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(alloc)
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
