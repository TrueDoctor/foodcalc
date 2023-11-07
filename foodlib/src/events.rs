use bigdecimal::BigDecimal;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgMoney, types::chrono::NaiveDateTime};

use crate::{recipes::EventRecipeIngredient, FoodBase};

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
        serialize_with = "crate::util::serialize_money",
        deserialize_with = "crate::util::deserialize_money"
    )]
    pub price: PgMoney,
    pub servings: i32,
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

impl FoodBase {
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
        Ok(records.price.unwrap_or(PgMoney(0)))
    }
}
