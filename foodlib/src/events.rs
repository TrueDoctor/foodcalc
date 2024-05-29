use core::fmt::Display;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgMoney, types::chrono::NaiveDateTime};
use std::borrow::Cow;
use tabled::Tabled;

use crate::{recipes::EventRecipeIngredient, FoodBase};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub event_id: i32,
    pub event_name: String,
    pub comment: Option<String>,
    #[serde(
        serialize_with = "crate::util::serialize_optional_money",
        deserialize_with = "crate::util::deserialize_optional_money"
    )]
    pub budget: Option<PgMoney>,
}
impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.event_name.as_str())
    }
}

impl Tabled for Event {
    const LENGTH: usize = 4;
    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "ID".into(),
            "Name".into(),
            "Comment".into(),
            "Budget".into(),
        ]
    }

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.event_id.to_string().into(),
            self.event_name.clone().into(),
            self.comment.clone().unwrap_or_default().into(),
            self.budget
                .map(crate::util::format_pg_money)
                .unwrap_or_default()
                .into(),
        ]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct Place {
    pub place_id: i32,
    pub name: String,
    #[tabled(display_with = "crate::util::display_optional")]
    pub comment: Option<String>,
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FoodBase {
    //TODO Merge this with function below, as this currently excludes events without meals
    pub async fn get_events(&self) -> eyre::Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#" SELECT event_id as "event_id!",
                    event_name as "event_name!",
                    events.comment as "comment",
                    budget as "budget"
                FROM events LEFT JOIN event_meals USING (event_id)
                GROUP BY event_id, event_name, events.comment, budget
                ORDER BY MIN(start_time) DESC
            "#
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn get_all_events(&self) -> eyre::Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#" SELECT event_id as "event_id!",
                    event_name as "event_name!",
                    events.comment as "comment",
                    budget as "budget"
                FROM events
            "#
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn get_event_from_string_reference(&self, reference: String) -> Option<Event> {
        let event_id = reference.parse::<i32>().unwrap_or(-1);
        let records = sqlx::query_as!(
            Event,
            r#" SELECT event_id as "event_id!",
                    event_name as "event_name!",
                    events.comment as "comment",
                    budget as "budget"
                FROM events LEFT JOIN event_meals USING (event_id)
                WHERE event_id = $1 OR event_name = $2
            "#,
            event_id,
            reference
        )
        .fetch_one(&*self.pg_pool)
        .await;

        records.ok()
    }

    pub async fn get_event(&self, id: i32) -> Option<Event> {
        let records = sqlx::query_as!(
            Event,
            r#" SELECT event_id as "event_id!",
                    event_name as "event_name!",
                    events.comment as "comment",
                    budget as "budget"
                FROM events
                WHERE event_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pg_pool)
        .await;

        records.ok()
    }

    pub async fn get_event_recipe_ingredients(
        &self,
        meal_id: i32,
    ) -> eyre::Result<Vec<EventRecipeIngredient>> {
        let records = sqlx::query_as!(
            EventRecipeIngredient,
            r#" SELECT ingredient_id as "ingredient_id!",
                   ingredient as "name!",
                   round(sum(weight) / servings, 2) as "weight!",
                   round(sum(energy) /servings, 2) as "energy!",
                   sum(price) / servings as "price!"
                FROM event_ingredients
                WHERE meal_id = $1
                GROUP BY ingredient_id, ingredient, servings
                ORDER BY sum(weight) DESC"#,
            meal_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        Ok(records)
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

    pub async fn add_event(
        &self,
        name: String,
        budget: Option<PgMoney>,
        comment: Option<String>,
    ) -> eyre::Result<Event> {
        let event = sqlx::query_as!(
            Event,
            r#"
                INSERT INTO events (event_name, comment, budget)
                VALUES ($1, $3, $2)
                RETURNING *
            "#,
            name,
            budget,
            comment,
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

    pub async fn get_place_from_string_reference(&self, reference: String) -> eyre::Result<Place> {
        let place_id = reference.parse::<i32>()?;
        let records = sqlx::query_as!(
            Place,
            r#" SELECT *
                FROM places
                WHERE place_id = $1
                    OR name = $2
            "#,
            place_id,
            reference
        )
        .fetch_one(&*self.pg_pool)
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
        Ok(records.price.unwrap_or(PgMoney(0)))
    }
}
