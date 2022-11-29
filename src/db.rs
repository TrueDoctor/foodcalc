use std::borrow::Cow;
use std::fmt::Display;
use std::sync::Arc;

use chrono::Duration;
use sqlx::postgres::types::{PgInterval, PgMoney};
use sqlx::postgres::PgPool;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::types::BigDecimal;

pub const METRO: i32 = 0;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IngredientCreate {
    pub id: Option<i32>,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
}

impl Ingredient {
    pub fn new(ingredient_id: i32, name: String, energy: BigDecimal, comment: Option<String>) -> Self {
        Self {
            ingredient_id,
            name,
            energy,
            comment,
        }
    }
}

impl From<Ingredient> for IngredientCreate {
    fn from(value: Ingredient) -> Self {
        IngredientCreate {
            id: Some(value.ingredient_id),
            name: value.name,
            energy: value.energy,
            comment: value.comment,
        }
    }
}

impl IngredientCreate {
    pub fn to_ingredient(&self) -> eyre::Result<Ingredient> {
        let Some(id) = self.id else {
                return Err(eyre::eyre!("No id found"));
            };
        Ok(Ingredient {
            ingredient_id: id,
            name: self.name.clone(),
            energy: self.energy.clone(),
            comment: self.comment.clone(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recipe {
    pub recipe_id: i32,
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventRecipeIngredient {
    pub ingredient_id: i32,
    pub name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: PgMoney,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubRecipe {
    pub subrecipe_id: i32,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: BigDecimal,
    pub is_subrecipe: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meal {
    pub event_id: i32,
    pub recipe_id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub place_id: i32,
    pub place: String,
    pub start_time: PrimitiveDateTime,
    pub end_time: PrimitiveDateTime,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: PgMoney,
    pub servings: i32,
}

impl Default for Meal {
    fn default() -> Self {
        Self {
            event_id: Default::default(),
            recipe_id: Default::default(),
            name: Default::default(),
            comment: None,
            place_id: Default::default(),
            place: Default::default(),
            start_time: PrimitiveDateTime::parse("1970-01-01 00:00:00", "%F %T").unwrap(),
            end_time: PrimitiveDateTime::parse("1970-01-01 00:00:00", "%F %T").unwrap(),
            weight: Default::default(),
            energy: Default::default(),
            price: PgMoney::from(0),
            servings: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Event {
    pub event_id: i32,
    pub event_name: String,
    pub comment: Option<String>,
    pub budget: Option<PgMoney>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Place {
    pub place_id: i32,
    pub name: String,
    pub comment: Option<String>,
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
    pub store_id: i32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub unit_id: i32,
    pub name: Cow<'static, str>,
}

impl Unit {
    pub const KG: Unit = Unit {
        unit_id: 0,
        name: Cow::Borrowed("kg"),
    };
}

impl Default for Unit {
    fn default() -> Self {
        Self::KG
    }
}

impl std::string::ToString for Unit {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RecipeMetaIngredient {
    Ingredient(Ingredient),
    MetaRecipe(Recipe),
}

impl Default for RecipeMetaIngredient {
    fn default() -> Self {
        Self::Ingredient(Ingredient::default())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RecipeEntry {
    pub ingredient: RecipeMetaIngredient,
    pub amount: BigDecimal,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeStep {
    pub step_id: i32,
    pub step_order: f64,
    pub step_name: String,
    pub step_description: String,
    pub fixed_duration: PgInterval,
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
impl std::string::ToString for RecipeEntry {
    fn to_string(&self) -> String {
        self.ingredient.name().to_string()
    }
}

#[derive(Clone)]
pub struct IngredientSorce {
    pub ingredient_id: i32,
    pub store_id: i32,
    pub package_size: BigDecimal,
    pub unit_id: i32,
    pub price: PgMoney,
    pub url: Option<String>,
    pub comment: Option<String>,
}

pub fn parse_package_size(description: &str) -> Option<(BigDecimal, i32)> {
    use regex::Regex;
    let number_regex = Regex::new(r"^[0-9][0-9,\.]*").expect("failed to compile number regex");
    let number = number_regex.find(description)?;

    let unit_regex = Regex::new(r"(?i)kg|g|l|tl|el|stk|knolle|zehe|ml|bund|pck|pkg|pckg|prise")
        .expect("failed to compile unit regex");
    let unit = unit_regex.find(description)?.as_str();
    let unit_id = match unit.to_lowercase().as_str() {
        "kg" => 0,
        "g" => 1,
        "l" => 2,
        "tl" => 3,
        "el" => 4,
        "stk" | "stück" => 5,
        "knolle" => 6,
        "zehe" => 7,
        "ml" => 8,
        "bund" => 9,
        "pck" | "pkg" | "pcgk" => 10,
        "prise" => 11,
        _ => return None,
    };

    use num::Num;
    match BigDecimal::from_str_radix(number.as_str(), 10) {
        Ok(amount) => {
            log::info!("parsed {description} as {amount} unit:{unit_id} {unit}");
            Some((amount, unit_id))
        },
        Err(_) => {
            log::error!("Failed to parse {description} as package_size");
            None
        },
    }
}

#[derive(Debug)]
pub struct FoodBase {
    pg_pool: Arc<PgPool>,
}

impl FoodBase {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
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
        .fetch_all(&mut conn)
        .await?;
        Ok(steps)
    }

    pub async fn add_ingredient(&self, name: String, energy: BigDecimal, comment: Option<String>) -> eyre::Result<i32> {
        log::debug!("add_ingredient({:?}, {:?}, {:?})", name, energy, comment);
        let ingredient = sqlx::query!(
            r#"
                INSERT INTO ingredients ( name, energy, comment )
                VALUES ( $1, $2, $3 )
                RETURNING ingredient_id
            "#,
            name,
            energy,
            comment
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(ingredient.ingredient_id)
    }

    pub async fn update_ingredient(&self, ingredient: Ingredient) -> eyre::Result<i32> {
        let ingredient = sqlx::query!(
            r#"
                UPDATE ingredients
                SET name = $1, energy = $2, comment = $3
                WHERE ingredient_id = $4
                RETURNING ingredient_id
            "#,
            ingredient.name,
            ingredient.energy,
            ingredient.comment,
            ingredient.ingredient_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(ingredient.ingredient_id)
    }

    pub async fn add_ingredient_source(
        &self,
        ingredient_id: i32,
        store_id: i32,
        weight: BigDecimal,
        price: PgMoney,
        url: Option<String>,
        unit_id: i32,
    ) -> eyre::Result<i32> {
        let ingredient = sqlx::query!(
            r#"
                INSERT INTO ingredient_sources ( ingredient_id, store_id, url, package_size, price, unit_id)
                VALUES ( $1, $2, $3, $4, $5, $6)
                RETURNING ingredient_id
            "#,
            ingredient_id,
            store_id,
            url,
            weight,
            price,
            unit_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(ingredient.ingredient_id)
    }

    pub async fn get_ingredients(&self) -> eyre::Result<Vec<Ingredient>> {
        let records = sqlx::query_as!(Ingredient, r#" SELECT * FROM ingredients ORDER BY ingredient_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?;

        Ok(records)
    }

    pub async fn get_recipes(&self) -> eyre::Result<Vec<Recipe>> {
        let records = sqlx::query_as!(Recipe, r#" SELECT * FROM recipes ORDER BY recipe_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?;

        Ok(records)
    }

    pub async fn get_units(&self) -> eyre::Result<Vec<Unit>> {
        struct FetchUnit {
            pub unit_id: i32,
            pub name: String,
        }
        let records = sqlx::query_as!(FetchUnit, r#" SELECT * FROM units ORDER BY unit_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?
            .into_iter()
            .map(|unit| Unit {
                unit_id: unit.unit_id,
                name: Cow::Owned(unit.name),
            })
            .collect();

        Ok(records)
    }

    pub async fn get_all_meta_ingredients(&self) -> eyre::Result<Vec<RecipeMetaIngredient>> {
        let ingredients = sqlx::query_as!(Ingredient, r#" SELECT * FROM ingredients ORDER BY ingredient_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?
            .into_iter()
            .map(RecipeMetaIngredient::Ingredient);

        let recipes = sqlx::query_as!(Recipe, r#" SELECT * FROM recipes ORDER BY recipe_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?
            .into_iter()
            .map(RecipeMetaIngredient::MetaRecipe);

        let records = recipes.chain(ingredients).collect();
        Ok(records)
    }

    pub async fn get_meta_ingredients(&self, recipe_id: i32) -> eyre::Result<Vec<RecipeEntry>> {
        let ingredients = self.get_recipe_ingredients(recipe_id).await?;
        let mut records = self.get_recipe_meta_ingredients(recipe_id).await?;
        records.extend(ingredients);
        Ok(records)
    }

    pub async fn get_recipe_ingredients(&self, recipe_id: i32) -> eyre::Result<Vec<RecipeEntry>> {
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
                 }| RecipeEntry {
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

    pub async fn get_recipe_meta_ingredients(&self, recipe_id: i32) -> eyre::Result<Vec<RecipeEntry>> {
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
                 }| RecipeEntry {
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

    pub async fn fetch_subrecipes_export(&self, recipe_id: i32, weight: BigDecimal) {
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
        .await
        .unwrap();
        let mut keys = subrecipes.iter().map(|sr| sr.subrecipe_id).collect::<Vec<i32>>();
        keys.dedup();

        let mut text = r#"
            \documentclass[11pt,a4paper]{article}

            \usepackage[T1]{fontenc}
            \usepackage[ngerman]{babel}
            \usepackage[utf8]{inputenc}
            \usepackage{gensymb}

            \usepackage{recipe}

            \begin{document}
            "#
        .to_owned();

        for subrecipe_id in keys {
            let ingredients: Vec<_> = subrecipes.iter().filter(|sr| sr.subrecipe_id == subrecipe_id).collect();
            self.format_subrecipe(&mut text, ingredients);
        }
        use std::fmt::Write;

        writeln!(text, "\\end{{document}}").unwrap();

        let mut file = std::fs::File::create(format!("recipes/{}.tex", subrecipes.first().unwrap().recipe)).unwrap();
        use std::io::prelude::Write as WF;
        file.write_all(text.as_bytes()).unwrap();
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

    // TODO: Human race condition, add proper locking / edit notifications
    pub async fn update_recipe_entries(
        &self,
        recipe: &Recipe,
        entries: impl Iterator<Item = RecipeEntry>,
    ) -> eyre::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        pub async fn insert_recipe_entry<'a>(
            executor: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
            recipe_id: i32,
            entry: RecipeEntry,
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
            recipe.recipe_id,
        )
        .execute(&mut transaction)
        .await?;
        log::debug!("Deleted {} recipe_ingredients", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM meta_recipes
                WHERE parent_id = $1
            "#,
            recipe.recipe_id,
        )
        .execute(&mut transaction)
        .await?;
        log::debug!("Deleted {} meta_recipes", count.rows_affected());

        for entry in entries {
            insert_recipe_entry(&mut transaction, recipe.recipe_id, entry).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub fn format_subrecipe(&self, text: &mut String, subrecipes: Vec<&SubRecipe>) {
        let title = escape_underscore(&subrecipes.first().unwrap().subrecipe);
        let ingredients: Vec<_> = subrecipes.iter().filter(|sr| !sr.is_subrecipe).collect();
        let meta_ingredients: Vec<_> = subrecipes.iter().filter(|sr| sr.is_subrecipe).collect();

        fn escape_underscore(s: &str) -> String {
            s.replace('_', " ")
        }
        use std::fmt::Write;
        writeln!(text, "\\addrecipe{{{title}}}").unwrap();
        for ingredient in meta_ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight.round(3)
            )
            .unwrap();
        }
        for ingredient in ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight.round(3)
            )
            .unwrap();
        }
        writeln!(text, "\\printrecipe{{{title}}}").unwrap();
    }

    pub async fn get_events(&self) -> eyre::Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#" SELECT event_id as "event_id!",
                    event_name as "event_name!",
                    comment as "comment",
                    budget as "budget"
                FROM events
            "#
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn get_event_recipe_ingredients(
        &self,
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: PrimitiveDateTime,
    ) -> eyre::Result<Vec<EventRecipeIngredient>> {
        let records = sqlx::query_as!(
            EventRecipeIngredient,
            r#" SELECT ingredient_id as "ingredient_id!",
                   ingredient as "name!",
                   round(sum(weight) / servings, 2) as "weight!",
                   round(sum(energy) /servings, 2) as "energy!",
                   sum(price) / servings as "price!"
                FROM event_ingredients
                WHERE event_id = $1
                    AND recipe_id = $2
                    AND place_id = $3
                    AND start_time = $4
                GROUP BY ingredient_id, ingredient, servings
                ORDER BY sum(weight) DESC"#,
            event_id,
            recipe_id,
            place_id,
            start_time
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        Ok(records)
    }

    pub async fn get_event_meals(&self, event_id: i32) -> eyre::Result<Vec<Meal>> {
        let records = sqlx::query_as!(
            Meal,
            r#" SELECT
            event_meals.event_id as "event_id!",
            event_meals.recipe_id as "recipe_id!",
             recipe as "name!",
             comment,
             event_meals.place_id as "place_id!",
             place as "place!",
             event_meals.start_time as "start_time!",
             event_meals.end_time as "end_time!",
             round(sum(weight),2) as "weight!",
             round(sum(energy) / event_meals.servings,0) as "energy!",
             sum(price) as "price!",
             event_meals.servings as "servings!"

            FROM event_ingredients
            INNER JOIN event_meals
            ON event_ingredients.event_id=event_meals.event_id
            AND event_ingredients.recipe_id = event_meals.recipe_id
            AND event_ingredients.place_id = event_meals.place_id
            AND event_ingredients.start_time = event_meals.start_time

            WHERE event_meals.event_id = $1
            GROUP BY event_meals.event_id, event_meals.recipe_id, recipe, event_meals.place_id, place, event_meals.start_time, event_meals.servings
            ORDER BY event_meals.start_time "#,
            event_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn fetch_metro_prices(&self, ingredient_id: Option<i32>) -> eyre::Result<()> {
        let sources = self.get_metro_ingredient_sources(ingredient_id).await?;
        for _source in sources {
            #[cfg(feature = "scraping")]
            if let Some(url) = _source.url.clone() {
                if let Some(price) =
                    tokio::task::spawn_blocking(move || super::scraping::fetch_metro_price_python(&url)).await?
                {
                    log::debug!("{} cents", price.0);
                    self.update_ingredient_source_price(_source.ingredient_id, _source.url, price)
                        .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn get_metro_ingredient_sources(&self, ingredient_id: Option<i32>) -> eyre::Result<Vec<IngredientSorce>> {
        let records = match ingredient_id {
            Some(id) => sqlx::query_as!(
                IngredientSorce,
                r#" SELECT * FROM ingredient_sources WHERE store_id = $1 AND ingredient_id = $2 ORDER BY ingredient_id "#,
                METRO,
                id
            )
            .fetch_all(&*self.pg_pool)
            .await?,
            None => sqlx::query_as!(
                IngredientSorce,
                r#" SELECT * FROM ingredient_sources WHERE store_id = $1 ORDER BY ingredient_id "#,
                METRO,
            )
            .fetch_all(&*self.pg_pool)
            .await?,
        };

        Ok(records)
    }

    pub async fn update_ingredient_source_price(
        &self,
        ingredient_id: i32,
        url: Option<String>,
        price: PgMoney,
    ) -> eyre::Result<u64> {
        sqlx::query!(
            r#"
                UPDATE ingredient_sources
                SET price = $3
                WHERE ingredient_id = $1 AND url = $2
            "#,
            ingredient_id,
            url,
            price,
        )
        .execute(&*self.pg_pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(|err| err.into())
    }

    pub async fn add_empty_event(&self) -> eyre::Result<Event> {
        let event = sqlx::query_as!(
            Event,
            r#"
                INSERT INTO events (event_name, comment, budget)
                VALUES ($1, NULL, NULL)
                RETURNING *
            "#,
            "",
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(event)
    }

    pub async fn update_event(&self, event: &Event) -> eyre::Result<Event> {
        let event = sqlx::query_as!(
            Event,
            r#"
                UPDATE events
                SET event_name = $1, comment = $2, budget = $3
                WHERE event_id = $4
                RETURNING *
            "#,
            event.event_name,
            event.comment,
            event.budget,
            event.event_id,
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(event)
    }

    pub async fn update_single_meal(&self, old_meal: Option<Meal>, new_meal: Option<Meal>) -> eyre::Result<()> {
        if let Some(old) = old_meal {
            if let Some(new) = new_meal {
                let count = sqlx::query!(
                    r#"
                    UPDATE event_meals
                    SET event_id = $1,
                        recipe_id = $2,
                        place_id = $3,
                        start_time = $4,
                        end_time = $5,
                        energy_per_serving = $6,
                        servings = $7,
                        comment = $8
                    WHERE
                        event_id = $9 AND
                        recipe_id = $10 AND
                        place_id = $11 AND
                        start_time = $12
                    "#,
                    new.event_id,
                    new.recipe_id,
                    new.place_id,
                    new.start_time,
                    new.end_time,
                    new.energy,
                    new.servings,
                    new.comment,
                    old.event_id,
                    old.recipe_id,
                    old.place_id,
                    old.start_time,
                )
                .execute(&*self.pg_pool)
                .await?
                .rows_affected();

                assert_eq!(count, 1);
            } else {
                let count = sqlx::query!(
                    r#"
                    DELETE FROM event_meals
                    WHERE
                        event_id = $1 AND
                        recipe_id = $2 AND
                        place_id = $3 AND
                        start_time = $4
                    "#,
                    old.event_id,
                    old.recipe_id,
                    old.place_id,
                    old.start_time,
                )
                .execute(&*self.pg_pool)
                .await?
                .rows_affected();

                assert_eq!(count, 1);
            }
        } else if let Some(new) = new_meal {
            let count = sqlx::query!(
            r#"
            INSERT INTO event_meals (event_id, recipe_id, place_id, start_time, end_time, energy_per_serving, servings, comment)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            new.event_id,
            new.recipe_id,
            new.place_id,
            new.start_time,
            new.end_time,
            new.energy,
            new.servings,
            new.comment,
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

            assert_eq!(count, 1);
        }
        Ok(())
    }

    pub async fn update_event_meals(&self, event: &Event, meals: impl Iterator<Item = Meal>) -> eyre::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        pub async fn insert_meal<'a>(
            executor: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
            event_id: i32,
            meal: Meal,
        ) -> sqlx::Result<()> {
            let count = sqlx::query!(
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

            assert_eq!(count, 1);
            Ok(())
        }
        let count = sqlx::query!(
            r#"
                DELETE FROM event_meals
                WHERE event_id = $1
            "#,
            event.event_id,
        )
        .execute(&mut transaction)
        .await?
        .rows_affected();
        log::debug!("Deleted {} event_meals", count);

        for meal in meals {
            insert_meal(&mut transaction, event.event_id, meal).await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_places(&self) -> eyre::Result<Vec<Place>> {
        let records = sqlx::query_as!(
            Place,
            r#" SELECT *
                FROM places
            "#
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }
}

mod tests {
    #[test]
    fn test_unit_parsing() {
        use super::*;
        assert_eq!(Some((BigDecimal::new(1u32.into(), 0), 0)), parse_package_size("1kg"));
        assert_eq!(Some((BigDecimal::new(15u32.into(), 1), 0)), parse_package_size("1.5kg"));
        assert_eq!(Some((BigDecimal::new(1u32.into(), 0), 10)), parse_package_size("1Pkg"));
        assert_eq!(None, parse_package_size("1"));
    }
}
