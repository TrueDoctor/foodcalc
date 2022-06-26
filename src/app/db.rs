use std::sync::Arc;

use iced::{
    button, Alignment, Application, Button, Checkbox, Column, Command, Container, Element, Font, Length, Row, Settings,
    Text,
};
use sqlx::postgres::types::PgMoney;
use sqlx::postgres::PgPool;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::types::BigDecimal;

pub const METRO: i32 = 0;

#[derive(Clone, Debug)]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}
impl Ingredient {
    pub fn view(&mut self) -> Element<TaskMessage> {
        let checkbox = Checkbox::new(true, &self.name, TaskMessage::Completed).width(Length::Fill);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(checkbox)
            /*.push(
                Button::new(&mut button::State::new(), edit_icon())
                    .on_press(TaskMessage::Edit)
                    .padding(10)
                    .style(style::Button::Icon),
            )*/
            .into()
    }
}

fn icon(unicode: char) -> Text {
    Text::new(unicode.to_string())
        //.font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(iced::alignment::Horizontal::Center)
        .size(20)
}

fn edit_icon() -> Text {
    icon('\u{F303}')
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}
mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        FilterActive,
        FilterSelected,
        Icon,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            match self {
                Button::FilterActive => button::Style::default(),
                Button::FilterSelected => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.7))),
                    border_radius: 10.0,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                },
                Button::Icon => button::Style {
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..button::Style::default()
                },
                Button::Destructive => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                    border_radius: 5.0,
                    text_color: Color::WHITE,
                    shadow_offset: Vector::new(1.0, 1.0),
                    ..button::Style::default()
                },
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            button::Style {
                text_color: match self {
                    Button::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                    Button::FilterActive => Color::from_rgb(0.2, 0.2, 0.7),
                    _ => active.text_color,
                },
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RecipeIngredient {
    pub ingredient_id: i32,
    pub name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: PgMoney,
}

#[derive(Clone, Debug)]
pub struct SubRecipe {
    pub subrecipe_id: i32,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: BigDecimal,
    pub is_subrecipe: bool,
}

#[derive(Clone, Debug)]
pub struct Meal {
    pub event_id: i32,
    pub recipe_id: i32,
    pub name: String,
    pub place_id: i32,
    pub place: String,
    pub start_time: PrimitiveDateTime,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: PgMoney,
    pub servings: i32,
}

use std::fmt::Display;
impl Display for Ingredient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct Store {
    pub store_id: i32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub unit_id: i32,
    pub name: String,
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

    pub async fn add_ingredient(&self, name: String, energy: BigDecimal, comment: Option<String>) -> eyre::Result<i32> {
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

    pub async fn get_ingredients_option(&self) -> Option<Vec<Ingredient>> {
        self.get_ingredients().await.ok()
    }

    pub async fn get_ingredients(&self) -> eyre::Result<Vec<Ingredient>> {
        let records = sqlx::query_as!(Ingredient, r#" SELECT * FROM ingredients ORDER BY ingredient_id "#,)
            .fetch_all(&*self.pg_pool)
            .await?;

        Ok(records)
    }

    pub async fn fetch_subrecipes_export(&self, recipe_id: i32, weight: BigDecimal) {
        let mut subrecipes = sqlx::query_as!(
            SubRecipe,
            r#"
                SELECT recipe as "recipe!", ingredient as "ingredient!", round(weight, 4) as "weight!", subrecipe as "subrecipe!", is_subrecipe as "is_subrecipe!", subrecipe_id as "subrecipe_id!" FROM (
            SELECT
                rr.recipe_id,
                rr.recipe,
                rr.ingredient,
                sum(rr.weight / recipe_weight.weight * $2) as weight,
                rr.subrecipe_id,
                recipes.name as subrecipe,
                false as is_subrecipe
                FROM resolved_recipes as rr
                JOIN recipe_weight using(recipe_id)
                JOIN recipes ON(subrecipe_id = recipes.recipe_id)
                where rr.recipe_id = $1 group by rr.recipe_id, rr.subrecipe_id, recipe, ingredient_id, ingredient, subrecipe
            UNION
            SELECT parent_id as recipe_id, parent as recipe, child as ingredient,
            meta_with_names.weight / recipe_weight.weight * $2 as weight,
             parent_id as subrecipe_id, parent as subrecipe, true as is_subrecipe
                FROM meta_with_names
                JOIN recipe_weight on(recipe_id = $1)
                where parent_id IN (SELECT subrecipe FROM resolved_meta where recipe_id = $1) or parent_id = $1
                ) as bar JOIN recipes USING(recipe_id) ORDER BY recipe, subrecipe_id, is_subrecipe DESC
            "#,
            recipe_id,
            weight,
        ).fetch_all(&*self.pg_pool).await.unwrap();
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

        let mut file = std::fs::File::create(format!("{}.tex", subrecipes.first().unwrap().recipe)).unwrap();
        use std::io::prelude::Write as WF;
        file.write_all(text.as_bytes()).unwrap();
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
                ingredient.weight
            )
            .unwrap();
        }
        for ingredient in ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight
            )
            .unwrap();
        }
        writeln!(text, "\\printrecipe{{{title}}}").unwrap();
    }

    pub async fn get_recipe_ingredients(
        &self,
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: PrimitiveDateTime,
    ) -> eyre::Result<Vec<RecipeIngredient>> {
        let records = sqlx::query_as!(
            RecipeIngredient,
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
            event_id as "event_id!",
             recipe_id as "recipe_id!",
             recipe as "name!",
             place_id as "place_id!",
             place as "place!",
             start_time as "start_time!",
             round(sum(weight),2) as "weight!",
             round(sum(energy),0) as "energy!",
             sum(price) as "price!",
             servings as "servings!"

            FROM event_ingredients
            WHERE event_id = $1
            GROUP BY event_id, recipe_id, recipe, place_id, place, start_time, servings
            ORDER BY start_time "#,
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
