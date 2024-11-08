use bigdecimal::BigDecimal;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgMoney, types::chrono::NaiveDateTime};
use tabled::Tabled;

use crate::FoodBase;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct Meal {
    #[tabled(rename = "ID")]
    pub meal_id: i32,
    #[tabled(skip)]
    pub event_id: i32,
    #[tabled(skip)]
    pub recipe_id: i32,
    #[tabled(rename = "Recipe")]
    pub name: String,
    #[tabled(skip)]
    pub place_id: i32,
    pub place: String,
    #[tabled(rename = "Start")]
    pub start_time: NaiveDateTime,
    #[tabled(rename = "End")]
    pub end_time: NaiveDateTime,
    #[tabled(rename = "Weight")]
    pub weight: BigDecimal,
    #[tabled(rename = "Energy")]
    pub energy: BigDecimal,
    #[serde(
        serialize_with = "crate::util::serialize_money",
        deserialize_with = "crate::util::deserialize_money"
    )]
    #[tabled(rename = "Price", display_with = "crate::util::format_pg_money")]
    pub price: PgMoney,
    #[tabled(rename = "Servings")]
    pub servings: i32,
    #[tabled(rename = "Comment", display_with = "crate::util::display_optional")]
    pub comment: Option<String>,
}

impl Default for Meal {
    fn default() -> Self {
        let time = chrono::Local::now();
        let date = time.date_naive();
        let time = chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let start_time = NaiveDateTime::new(date, time);
        Self {
            meal_id: Default::default(),
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

impl FoodBase {
    pub async fn get_event_meals(&self, event_id: i32) -> eyre::Result<Vec<Meal>> {
        let records = sqlx::query_as!(
            Meal,
            r#" SELECT
            event_meals.event_id as "event_id!",
            event_meals.meal_id,
            event_meals.recipe_id as "recipe_id!",
             recipe as "name!",
             comment,
             event_meals.place_id as "place_id!",
             place as "place!",
             event_meals.start_time as "start_time!",
             event_meals.end_time as "end_time!",
             COALESCE(round(sum(weight),2),0) as "weight!",
             COALESCE((CASE WHEN event_meals.servings != 0 THEN round(sum(energy) / event_meals.servings,0) ELSE 0 END),0) as "energy!",
             COALESCE(sum(price),'0'::float8::numeric::money) as "price!",
             event_meals.servings as "servings!"

            FROM event_ingredients
            INNER JOIN event_meals
            ON event_ingredients.meal_id = event_meals.meal_id

            WHERE event_meals.event_id = $1
            GROUP BY recipe, place, event_meals.servings, event_meals.meal_id
            ORDER BY event_meals.start_time "#,
            event_id
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn get_event_meal(&self, meal_id: i32) -> eyre::Result<Meal> {
        let records = sqlx::query_as!(
            Meal,
            r#" SELECT
            event_meals.meal_id,
            event_meals.event_id as "event_id!",
            event_meals.recipe_id as "recipe_id!",
             recipe as "name!",
             comment,
             event_meals.place_id as "place_id!",
             place as "place!",
             event_meals.start_time as "start_time!",
             event_meals.end_time as "end_time!",
             COALESCE(round(sum(weight),2),0) as "weight!",
             COALESCE((CASE WHEN event_meals.servings != 0 THEN round(sum(energy) / event_meals.servings,0) ELSE 0 END),0) as "energy!",
             COALESCE(sum(price),'0'::float8::numeric::money) as "price!",
             event_meals.servings as "servings!"

            FROM event_ingredients
            INNER JOIN event_meals
            ON event_ingredients.meal_id=event_meals.meal_id

            WHERE event_meals.meal_id = $1
            GROUP BY recipe, place, event_meals.servings, event_meals.meal_id
            ORDER BY event_meals.start_time "#,
            meal_id,
        )
        .fetch_one(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn get_meals(&self) -> eyre::Result<Vec<Meal>> {
        let records = sqlx::query_as!(
            Meal,
            r#" SELECT
            event_meals.meal_id,
            event_meals.event_id as "event_id!",
            event_meals.recipe_id as "recipe_id!",
             recipe as "name!",
             comment,
             event_meals.place_id as "place_id!",
             place as "place!",
             event_meals.start_time as "start_time!",
             event_meals.end_time as "end_time!",
             round(sum(weight),2) as "weight!",
             (CASE WHEN event_meals.servings != 0 THEN round(sum(energy) / event_meals.servings,0) ELSE 0 END) as "energy!",
             sum(price) as "price!",
             event_meals.servings as "servings!"

            FROM event_ingredients
            INNER JOIN event_meals
            ON event_ingredients.meal_id=event_meals.meal_id

            GROUP BY event_meals.meal_id, recipe, place, event_meals.servings
            ORDER BY event_meals.start_time "#
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(records)
    }

    pub async fn update_single_meal(
        &self,
        meal_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
        energy: BigDecimal,
        servings: i32,
        comment: Option<String>,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
                    UPDATE event_meals
                    SET recipe_id = $1,
                        place_id = $2,
                        start_time = $3,
                        end_time = $4,
                        energy_per_serving = $5,
                        servings = $6,
                        comment = $7
                    WHERE
                        meal_id = $8
                    "#,
            recipe_id,
            place_id,
            start_time,
            end_time,
            energy,
            servings,
            comment,
            meal_id
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        assert_eq!(count, 1);
        Ok(())
    }

    pub async fn add_meal(
        &self,
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
        energy: BigDecimal,
        servings: i32,
        comment: Option<String>,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
            INSERT INTO event_meals (event_id, recipe_id, place_id, start_time, end_time, energy_per_serving, servings, comment)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            event_id,
            recipe_id,
            place_id,
            start_time,
            end_time,
            energy,
            servings,
            comment,
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        assert_eq!(count, 1);
        Ok(())
    }

    pub async fn remove_meal(&self, meal_id: i32) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
            DELETE FROM event_meals
            WHERE
                meal_id = $1
            "#,
            meal_id
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        assert_eq!(count, 1);
        Ok(())
    }

    pub async fn remove_meal_by_reference(
        &self,
        event_id: i32,
        recipe_id: i32,
        place_id: i32,
        start_time: NaiveDateTime,
    ) -> eyre::Result<()> {
        let count = sqlx::query!(
            r#"
            DELETE FROM event_meals
            WHERE
                event_id = $1 AND
                recipe_id = $2 AND
                place_id = $3 AND
                start_time = $4
            "#,
            event_id,
            recipe_id,
            place_id,
            start_time,
        )
        .execute(&*self.pg_pool)
        .await?
        .rows_affected();

        assert_eq!(count, 1);
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
        .execute(&mut *transaction)
        .await?
        .rows_affected();
        log::debug!("Deleted {} event_meals", count);

        for meal in meals {
            insert_meal(&mut *transaction, event_id, meal).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
