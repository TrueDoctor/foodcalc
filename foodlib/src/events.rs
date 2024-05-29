use chrono::NaiveDateTime;
use core::fmt::Display;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgMoney;
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct FoodPrep {
    pub prep_id: i32,
    pub event_id: i32,
    pub recipe_id: i32,
    pub prep_date: NaiveDateTime,
    #[tabled(display_with = "crate::util::display_optional")]
    pub use_from: Option<NaiveDateTime>,
    pub use_until: NaiveDateTime,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct ShoppingTour {
    pub tour_id: i32,
    pub event_id: i32,
    pub tour_date: NaiveDateTime,
    pub store_id: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct SourceOverride {
    event_id: i32,
    ingredient_source_id: i32,
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

    pub async fn get_event_shopping_tours(&self, event_id: i32) -> eyre::Result<Vec<ShoppingTour>> {
        let records = sqlx::query_as!(
            ShoppingTour,
            "SELECT * FROM shopping_tours WHERE event_id=$1",
            event_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn add_event_shopping_tour(
        &self,
        event_id: i32,
        store_id: i32,
        date: NaiveDateTime,
    ) -> eyre::Result<ShoppingTour> {
        let tour = sqlx::query_as!(
            ShoppingTour,
            r#"
                INSERT INTO public.shopping_tours (tour_id, event_id, tour_date, store_id) 
                VALUES (DEFAULT, $1, $2, $3)
                RETURNING *
            "#,
            event_id,
            date,
            store_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(tour)
    }

    pub async fn update_event_shopping_tour_date(
        &self,
        tour_id: i32,
        date: NaiveDateTime,
    ) -> eyre::Result<ShoppingTour> {
        todo!()
    }

    pub async fn update_event_shopping_tour_store(
        &self,
        tour_id: i32,
        store_id: i32,
    ) -> eyre::Result<ShoppingTour> {
        todo!()
    }

    pub async fn get_event_food_prep(&self, event_id: i32) -> eyre::Result<Vec<FoodPrep>> {
        let records = sqlx::query_as!(
            FoodPrep,
            "SELECT * FROM food_prep WHERE event_id=$1",
            event_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn add_event_food_prep(
        &self,
        event_id: i32,
        recipe_id: i32,
        prep_date: NaiveDateTime,
        use_from: Option<NaiveDateTime>,
        use_til: NaiveDateTime,
    ) -> eyre::Result<FoodPrep> {
        let query = sqlx::query_as!(
            FoodPrep,
            r#"
                INSERT INTO public.food_prep (event_id, recipe_id, prep_date, use_from, use_until)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
            "#,
            event_id,
            recipe_id,
            prep_date,
            use_from,
            use_til
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(query)
    }

    pub async fn get_event_source_overrides(
        &self,
        event_id: i32,
    ) -> eyre::Result<Vec<SourceOverride>> {
        let records = sqlx::query_as!(
            SourceOverride,
            "SELECT * FROM event_source_overrides WHERE event_id=$1",
            event_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn add_event_source_override(
        &self,
        event_id: i32,
        source_id: i32,
    ) -> eyre::Result<SourceOverride> {
        let source_override = sqlx::query_as!(
            SourceOverride,
            r#"
                INSERT INTO public.event_source_overrides (event_id, ingredient_source_id) 
                VALUES ($1, $2)
                RETURNING *
            "#,
            event_id,
            source_id
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(source_override)
    }
}
