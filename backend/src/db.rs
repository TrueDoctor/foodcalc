use std::borrow::Cow;
use std::fmt::Display;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

use num::FromPrimitive;
use num::ToPrimitive;
use sqlx::postgres::types::{PgInterval, PgMoney};
use sqlx::postgres::PgPool;

use sqlx::types::BigDecimal;

pub const METRO: i32 = 0;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngredientCreate {
    pub id: Option<i32>,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
}

impl Ingredient {
    pub fn new(
        ingredient_id: i32,
        name: String,
        energy: BigDecimal,
        comment: Option<String>,
    ) -> Self {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub recipe_id: i32,
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecipeIngredient {
    pub ingredient_id: i32,
    pub name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    #[serde(
        serialize_with = "crate::db::serialize_money",
        deserialize_with = "crate::db::deserialize_money"
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meal {
    pub event_id: i32,
    pub recipe_id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub place_id: i32,
    pub place: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    #[serde(
        serialize_with = "crate::db::serialize_money",
        deserialize_with = "crate::db::deserialize_money"
    )]
    pub price: PgMoney,
    pub servings: i32,
}

fn serialize_money<S>(money: &PgMoney, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&money.0.to_string())
}

fn deserialize_money<'de, D>(deserializer: D) -> Result<PgMoney, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let money = PgMoney::from_bigdecimal(FromStr::from_str(&s).unwrap(), 2);
    Ok(money.unwrap())
}

impl Default for Meal {
    fn default() -> Self {
        let time = chrono::Local::now();
        let date = time.date_naive();
        let time = chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let start_time = NaiveDateTime::new(date, time);
        Self {
            event_id: Default::default(),
            recipe_id: Default::default(),
            name: Default::default(),
            comment: None,
            place_id: Default::default(),
            place: Default::default(),
            start_time,
            end_time: start_time,
            weight: Default::default(),
            energy: BigDecimal::from(2400),
            price: PgMoney::from(0),
            servings: 1,
        }
    }
}

fn serialize_optional_money<S>(money: &Option<PgMoney>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match money {
        Some(money) => serializer.serialize_str(&money.0.to_string()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_optional_money<'de, D>(deserializer: D) -> Result<Option<PgMoney>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let money: i64 = i64::from_str(&s).map_err(serde::de::Error::custom)?;
        let money = PgMoney(money);
        Ok(Some(money))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub event_id: i32,
    pub event_name: String,
    pub comment: Option<String>,
    #[serde(
        serialize_with = "crate::db::serialize_optional_money",
        deserialize_with = "crate::db::deserialize_optional_money"
    )]
    pub budget: Option<PgMoney>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Store {
    pub store_id: i32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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
pub struct RecipeIngrdient {
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
        serialize_with = "crate::db::serialize_interval",
        deserialize_with = "crate::db::deserialize_interval"
    )]
    pub fixed_duration: PgInterval,
    #[serde(
        serialize_with = "crate::db::serialize_interval",
        deserialize_with = "crate::db::deserialize_interval"
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

fn serialize_interval<S>(interval: &PgInterval, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration = interval.microseconds;
    serializer.serialize_str(&duration.to_string())
}

fn deserialize_interval<'de, D>(deserializer: D) -> Result<PgInterval, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let microseconds = s
        .parse()
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse interval: {}", e)))?;
    let interval = PgInterval {
        microseconds,
        days: 0,
        months: 0,
    };
    Ok(interval)
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
impl std::string::ToString for RecipeIngrdient {
    fn to_string(&self) -> String {
        self.ingredient.name().to_string()
    }
}

pub struct ShoppingListItem {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub price: PgMoney,
    pub weight: BigDecimal,
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
        }
        Err(_) => {
            log::error!("Failed to parse {description} as package_size");
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct FoodBase {
    pub(crate) pg_pool: Arc<PgPool>,
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

    pub async fn add_ingredient(
        &self,
        name: String,
        energy: BigDecimal,
        comment: Option<String>,
    ) -> eyre::Result<i32> {
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
        let records = sqlx::query_as!(
            Ingredient,
            r#" SELECT * FROM ingredients ORDER BY ingredient_id "#,
        )
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
        let ingredients = sqlx::query_as!(
            Ingredient,
            r#" SELECT * FROM ingredients ORDER BY ingredient_id "#,
        )
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

    pub async fn get_meta_ingredients(&self, recipe_id: i32) -> eyre::Result<Vec<RecipeIngrdient>> {
        let ingredients = self.get_recipe_ingredients(recipe_id).await?;
        let mut records = self.get_recipe_meta_ingredients(recipe_id).await?;
        records.extend(ingredients);
        Ok(records)
    }

    pub async fn get_recipe_ingredients(
        &self,
        recipe_id: i32,
    ) -> eyre::Result<Vec<RecipeIngrdient>> {
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
                 }| RecipeIngrdient {
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
    ) -> eyre::Result<Vec<RecipeIngrdient>> {
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
                 }| RecipeIngrdient {
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

    pub async fn fetch_subrecipes_export(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> Result<(), eyre::Error> {
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
        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();
        keys.dedup();

        let mut text = r#"
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
            "#
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

        use std::fmt::Write as FmtWrite;
        writeln!(text, "\\end{{document}}")?;

        #[cfg(feature = "tectonic")]
        {
            use std::io::Write;
            use std::path::Path;
            use tectonic::driver::ProcessingSessionBuilder;
            use tectonic::status;
            use tokio::task::spawn_blocking;

            let mut status = status::NoopStatusBackend::default();
            let name = subrecipes
                .first()
                .ok_or(eyre::eyre!("No recipe name found"))?
                .recipe
                .clone();

            let mut files = {
                spawn_blocking(move || {
                    let auto_create_config_file = false;
                    let config =
                        tectonic::config::PersistentConfig::open(auto_create_config_file).unwrap();

                    let only_cached = false;
                    let bundle = config.default_bundle(only_cached, &mut status).unwrap();

                    let format_cache_path = config.format_cache_path().unwrap();

                    let mut sb = ProcessingSessionBuilder::default();
                    sb.filesystem_root(Path::new("./recipes"))
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

            let Some(pdf) = files.remove("texput.pdf")  else {
                return Err(eyre::eyre!("LaTeX didn't report failure, but no PDF was created (??)"));
            };
            let pdf_data = pdf.data;
            println!("Output PDF size is {} bytes", pdf_data.len());

            let create_result = std::fs::create_dir("recipes/out");
            if let Err(e) = create_result {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(eyre::eyre!("failed to create output directory: {}", e));
                }
            }
            let mut file = std::fs::File::create(format!("recipes/out/{}.pdf", name))?;
            file.write_all(&pdf_data)?;
        }
        #[cfg(not(feature = "tectonic"))]
        {
            let mut file = std::fs::File::create(format!(
                "recipes/{}.tex",
                subrecipes.first().unwrap().recipe
            ))
            .unwrap();
            use std::io::prelude::Write as WF;
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
        entries: impl Iterator<Item = RecipeIngrdient>,
    ) -> eyre::Result<()> {
        let mut transaction = self.pg_pool.begin().await?;
        pub async fn insert_recipe_entry<'a>(
            executor: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
            recipe_id: i32,
            entry: RecipeIngrdient,
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
        .execute(&mut transaction)
        .await?;
        log::debug!("Deleted {} recipe_ingredients", count.rows_affected());

        let count = sqlx::query!(
            r#"
                DELETE FROM meta_recipes
                WHERE parent_id = $1
            "#,
            recipe_id,
        )
        .execute(&mut transaction)
        .await?;
        log::debug!("Deleted {} meta_recipes", count.rows_affected());

        for entry in entries {
            insert_recipe_entry(&mut transaction, recipe_id, entry).await?;
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
        .execute(&mut transaction)
        .await?;
        log::debug!("Deleted {} steps", count.rows_affected());

        for entry in entries {
            insert_recipe_step(&mut transaction, recipe_id, entry).await?;
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
                let duration = chrono::Duration::microseconds(duration.microseconds);
                duration.num_minutes().to_f64().unwrap_or_default()
                    + duration.num_seconds().to_f64().unwrap_or_default() / 60.
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

    pub async fn get_events(&self) -> eyre::Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#" SELECT event_id as "event_id!",
                    event_name as "event_name!",
                    events.comment as "comment",
                    budget as "budget"
                FROM events INNER JOIN event_meals USING (event_id)
                GROUP BY event_id, event_name, events.comment, budget
                ORDER BY MIN(start_time) DESC
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
        start_time: NaiveDateTime,
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
        // get urls of metro articles
        let urls: Vec<IngredientSorce> = self
            .get_metro_ingredient_sources(ingredient_id)
            .await?
            .into_iter()
            .filter(|is| is.url.is_some())
            .collect();

        // fetch article descriptions from the metro api
        let articles = metro_scrape::request::fetch_articles_from_urls(urls.iter().flat_map(|s| {
            s.url
                .is_some()
                .then(|| (s.ingredient_id, s.url.clone().unwrap()))
        }))
        .await?;
        print!("Fetched {} articles from metro api", articles.len());

        async fn update_ingredient_price(
            foodbase: &FoodBase,
            article: metro_scrape::article::Article,
            s: IngredientSorce,
        ) -> Result<(), eyre::ErrReport> {
            let variant = article
                .variants
                .values()
                .next()
                .ok_or(eyre::eyre!("Variant not found for id {}", s.ingredient_id))?;
            let bundle = variant
                .bundles
                .values()
                .next()
                .ok_or(eyre::eyre!("Bundle not found for id {}", s.ingredient_id))?;
            let price = bundle
                .stores
                .values()
                .next()
                .ok_or(eyre::eyre!("Store not found for id {}", s.ingredient_id))?
                .selling_price_info
                .gross_price;
            let weight = &bundle.gross_weight;
            println!(
                "#{}: {}€ {}kg {} {:?}",
                s.ingredient_id,
                price,
                weight,
                bundle.bundle_size,
                bundle.weight_per_piece.as_ref().map(|w| w.to_string())
            );
            println!("{}: {}", bundle.details.header.misc_name_webshop, price);
            let price = sqlx::postgres::types::PgMoney::from_bigdecimal(
                BigDecimal::from_f64(price)
                    .ok_or(eyre::eyre!("Failed to represent as BigDecimal"))?,
                2,
            )
            .map_err(|e| eyre::eyre!(e))?;
            foodbase
                .update_ingredient_source_price(
                    s.ingredient_id,
                    s.url,
                    price,
                    BigDecimal::from_str(weight)?,
                )
                .await?;
            Ok(())
        }
        for source in urls.iter() {
            if !articles.iter().any(|x| x.0 == source.ingredient_id) {
                log::error!("No article found for {}", source.ingredient_id);
            }
        }
        let source_articles = articles.into_iter().map(|(id, article)| {
            (
                urls.iter().find(|x| x.ingredient_id == id).clone().unwrap(),
                article,
            )
        });

        for (source, article) in source_articles {
            update_ingredient_price(self, article, source.clone())
                .await
                .unwrap_or_else(|e| log::error!("{e}"));
        }
        Ok(())
    }

    pub async fn get_metro_ingredient_sources(
        &self,
        ingredient_id: Option<i32>,
    ) -> eyre::Result<Vec<IngredientSorce>> {
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
        weight: BigDecimal,
    ) -> eyre::Result<u64> {
        sqlx::query!(
            r#"
                UPDATE ingredient_sources
                SET price = $3, package_size = $4, unit_id = 0
                WHERE ingredient_id = $1 AND url = $2
            "#,
            ingredient_id,
            url,
            price,
            weight,
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

    pub async fn update_single_meal(
        &self,
        old_meal: Option<Meal>,
        new_meal: Option<Meal>,
    ) -> eyre::Result<()> {
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

    pub async fn update_event_meals(
        &self,
        event_id: i32,
        meals: impl Iterator<Item = Meal>,
    ) -> eyre::Result<()> {
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
            event_id,
        )
        .execute(&mut transaction)
        .await?
        .rows_affected();
        log::debug!("Deleted {} event_meals", count);

        for meal in meals {
            insert_meal(&mut transaction, event_id, meal).await?;
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

    pub async fn get_event_cost(&self, event_id: i32) -> eyre::Result<PgMoney> {
        let records = sqlx::query!(
            r#"
                SELECT
                    SUM(price) as price
                FROM shopping_list
                WHERE event_id = $1
            "#,
            event_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(records.price.unwrap_or_else(|| PgMoney(0)))
    }
}

mod tests {
    #[test]
    fn test_unit_parsing() {
        use super::*;
        assert_eq!(
            Some((BigDecimal::new(1u32.into(), 0), 0)),
            parse_package_size("1kg")
        );
        assert_eq!(
            Some((BigDecimal::new(15u32.into(), 1), 0)),
            parse_package_size("1.5kg")
        );
        assert_eq!(
            Some((BigDecimal::new(1u32.into(), 0), 10)),
            parse_package_size("1Pkg")
        );
        assert_eq!(None, parse_package_size("1"));
    }
}
