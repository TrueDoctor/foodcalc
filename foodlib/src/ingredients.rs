use num::FromPrimitive;
use num::Num;
use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;
use tabled::Tabled;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgMoney, types::BigDecimal};

use crate::{
    recipes::{Recipe, RecipeIngredient, RecipeMetaIngredient},
    FoodBase,
};

pub const METRO: i32 = 0;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub name: String,
    pub energy: BigDecimal,
    #[tabled(display_with = "crate::util::display_optional")]
    pub comment: Option<String>,
}

impl Display for Ingredient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
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

    pub fn change_name(&self, name: String) -> Self {
        Self {
            name,
            ..self.clone()
        }
    }

    pub fn change_energy(&self, energy: BigDecimal) -> Self {
        Self {
            energy,
            ..self.clone()
        }
    }

    pub fn change_comment(&self, comment: Option<String>) -> Self {
        Self {
            comment,
            ..self.clone()
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

pub struct ShoppingListItem {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub price: PgMoney,
    pub weight: BigDecimal,
}

#[derive(Clone)]
pub struct IngredientSource {
    pub ingredient_source_id: i32,
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

impl FoodBase {
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

    pub async fn update_ingredient(&self, ingredient: &Ingredient) -> eyre::Result<i32> {
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

    pub async fn get_ingredient(&self, ingredient_id: i32) -> eyre::Result<Ingredient> {
        let record = sqlx::query_as!(
            Ingredient,
            r#" SELECT * FROM ingredients WHERE ingredient_id = $1 "#,
            ingredient_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(record)
    }
    pub async fn ingredient_usages(&self, ingredient_id: i32) -> eyre::Result<Vec<Recipe>> {
        let record = sqlx::query_as!(
            Recipe,
            r#" SELECT name, recipes.recipe_id, comment FROM recipe_ingredients
                INNER JOIN recipes USING(recipe_id)
                WHERE ingredient_id = $1
                GROUP BY recipes.name, recipes.recipe_id, recipes.comment
            "#,
            ingredient_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        Ok(record)
    }

    pub async fn delete_ingredient(&self, ingredient_id: i32) -> eyre::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM recipe_ingredients WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .execute(&*self.pg_pool)
        .await?;
        sqlx::query!(
            r#"
                DELETE FROM ingredients WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .execute(&*self.pg_pool)
        .await?;
        Ok(())
    }

    pub async fn get_ingredient_from_string_reference(
        &self,
        reference: String,
    ) -> Option<Ingredient> {
        let ingredient_id = reference.parse::<i32>().unwrap_or(-1);
        let records = sqlx::query_as!(
            Ingredient,
            r#" 
                SELECT * FROM ingredients 
                WHERE name = $1 OR ingredient_id = $2
                ORDER BY ingredient_id
            "#,
            reference,
            ingredient_id
        )
        .fetch_one(&*self.pg_pool)
        .await;

        if let Ok(record) = records {
            Some(record)
        } else {
            None
        }
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

    pub async fn get_unit(&self, unit_id: i32) -> eyre::Result<Unit> {
        let record = sqlx::query_as!(Unit, r#" SELECT * FROM units WHERE unit_id = $1 "#, unit_id)
            .fetch_one(&*self.pg_pool)
            .await?;

        Ok(record)
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

    pub async fn get_meta_ingredients(
        &self,
        recipe_id: i32,
    ) -> eyre::Result<Vec<RecipeIngredient>> {
        let ingredients = self.get_recipe_ingredients(recipe_id).await?;
        let mut records = self.get_recipe_meta_ingredients(recipe_id).await?;
        records.extend(ingredients);
        Ok(records)
    }

    pub async fn fetch_metro_prices(&self, ingredient_id: Option<i32>) -> eyre::Result<()> {
        // get urls of metro articles
        let urls: Vec<IngredientSource> = self
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
            s: IngredientSource,
        ) -> Result<(), eyre::ErrReport> {
            let variant = article
                .variants
                .values()
                .next()
                .ok_or(eyre::eyre!("Variant not found for id {}", s.ingredient_id))?;
            let bundle = variant
                .bundles
                .values()
                .min_by_key(|b| (f64::from_str(&b.gross_weight).unwrap_or_default() * 1000.) as u64)
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
                urls.iter().find(|x| x.ingredient_id == id).unwrap(),
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
    ) -> eyre::Result<Vec<IngredientSource>> {
        let records = match ingredient_id {
            Some(id) => sqlx::query_as!(
                IngredientSource,
                r#" SELECT * FROM ingredient_sources WHERE store_id = $1 AND ingredient_id = $2 ORDER BY ingredient_id "#,
                METRO,
                id
            )
            .fetch_all(&*self.pg_pool)
            .await?,
            None => sqlx::query_as!(
                IngredientSource,
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
