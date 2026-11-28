use crate::entities::event::*;
use crate::error::{Error, Result};
use crate::types::ids::{
    EventId, FoodPrepId, GroupId, IngredientId, IngredientSourceId, InventoryId, RecipeId,
    ShoppingTourId, StoreId, UserId,
};

use sqlx::PgPool;
use std::sync::Arc;
use bigdecimal::BigDecimal;
use time::OffsetDateTime;

/// Operations for managing events and their associated data
pub struct EventOps {
    pool: Arc<PgPool>,
}

impl EventOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Creates a new event
    pub async fn create(&self, event: Event) -> Result<Event> {
        let record = sqlx::query_as!(
            Event,
            r#"
            INSERT INTO events (event_name, comment, budget, group_id)
            VALUES ($1, $2, $3, $4)
            RETURNING event_id as "id: EventId", event_name as name, comment, budget , group_id as "group_id: GroupId"
            "#,
            event.name,
            event.comment,
            event.budget,
            event.group_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Retrieves an event by ID
    pub async fn get(&self, id: EventId) -> Result<Event> {
        let record = sqlx::query_as!(
            Event,
            r#"
            SELECT 
                event_id as "id: EventId",
                event_name as name,
                comment,
                group_id as "group_id: GroupId",
                budget
            FROM events 
            WHERE event_id = $1
            "#,
            id.get()
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Event",
            id: id.to_string(),
        })?;

        Ok(record)
    }

    /// Updates an existing event (group_id is preserved from DB)
    pub async fn update(&self, event: Event) -> Result<Event> {
        let record = sqlx::query_as!(
            Event,
            r#"
            UPDATE events
            SET event_name = $1, comment = $2, budget = $3
            WHERE event_id = $4
            RETURNING event_id as "id: EventId", event_name as name, comment, budget , group_id as "group_id: GroupId"
            "#,
            event.name,
            event.comment,
            event.budget,
            event.id.get()
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Event",
            id: event.id.to_string(),
        })?;

        Ok(record)
    }

    /// Moves an event to a different group
    pub async fn set_group(&self, event_id: EventId, group_id: GroupId) -> Result<()> {
        sqlx::query!(
            r#"UPDATE events SET group_id = $1 WHERE event_id = $2"#,
            group_id.get(),
            event_id.get()
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    /// Deletes an event and all associated data
    pub async fn delete(&self, id: EventId) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete associated data first
        sqlx::query!("DELETE FROM event_meals WHERE event_id = $1", id.get())
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM event_source_overrides WHERE event_id = $1", id.get())
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM shopping_tours WHERE event_id = $1", id.get())
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM food_prep WHERE event_id = $1", id.get())
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM event_inventories WHERE event_id = $1", id.get())
            .execute(&mut *tx)
            .await?;

        // Finally delete the event
        let result = sqlx::query!("DELETE FROM events WHERE event_id = $1", id.get())
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "Event",
                id: id.to_string(),
            });
        }

        tx.commit().await?;
        Ok(())
    }

    /// Duplicates an event, including all associated data
    pub async fn duplicate(&self, id: EventId, new_group_id: GroupId) -> Result<EventId> {
        let mut tx = self.pool.begin().await?;

        // Get the event details
        let event = sqlx::query_as!(
            Event,
            r#"
            SELECT 
                event_id as id,
                event_name as name,
                comment,
                group_id as "group_id: GroupId",
                budget
            FROM events 
            WHERE event_id = $1
            "#,
            id.get()
        )
        .fetch_one(&mut *tx)
        .await?;

        let new_name = format!("{} (Copy)", event.name);

        // Create a new event
        let new_event = sqlx::query_as!(
            Event,
            r#"
            INSERT INTO events (event_name, comment, budget, group_id)
            VALUES ($1, $2, $3, $4)
            RETURNING event_id as "id: EventId", event_name as name, comment, budget, group_id as "group_id: GroupId"
            "#,
            new_name,
            event.comment,
            event.budget,
            new_group_id.get(),
        )
        .fetch_one(&mut *tx)
        .await?;

        // Duplicate meal plans
        let meals = crate::ops::meals::MealOps::new(self.pool.clone())
            .get_event_meals(id)
            .await?;

        for meal in meals {
            sqlx::query!(
                    r#"
                        INSERT INTO event_meals (event_id, recipe_id, place_id, start_time, end_time, energy_per_serving, servings, comment)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    "#,
                    new_event.id.get(),
                    meal.recipe_id.get(),
                    meal.place_id.get(),
                    meal.start_time,
                    meal.end_time,
                    meal.energy,
                    meal.servings,
                    meal.comment,
                )
                .execute(&mut *tx)
                .await?;
        }

        // Duplicate event source overrides
        let overrides = sqlx::query_as!(
            SourceOverride,
            r#"
            SELECT event_id as "event_id: EventId", ingredient_source_id as "source_id: IngredientSourceId"
            FROM event_source_overrides
            WHERE event_id = $1
            "#,
            id.get()
        )
        .fetch_all(&mut *tx)
        .await?;

        for source_override in overrides {
            sqlx::query!(
                r#"
                INSERT INTO event_source_overrides (event_id, ingredient_source_id)
                VALUES ($1, $2)
                "#,
                new_event.id.get(),
                source_override.source_id.get(),
            )
            .execute(&mut *tx)
            .await?;
        }

        // TODO Duplicate shopping tours
        let tours = sqlx::query_as!(
            ShoppingTour,
            r#"
            SELECT tour_id as "id: ShoppingTourId", event_id as "event_id: EventId", tour_date, store_id as "store_id: StoreId", name as store_name
            FROM shopping_tours JOIN stores USING(store_id)
            WHERE event_id = $1
            "#,
            id.get()
        )
        .fetch_all(&mut *tx)
        .await?;

        for tour in tours {
            sqlx::query!(
                r#"
                INSERT INTO shopping_tours (event_id, tour_date, store_id)
                VALUES ($1, $2, $3)
                "#,
                new_event.id.get(),
                tour.tour_date,
                tour.store_id.get(),
            )
            .execute(&mut *tx)
            .await?;
        }

        // Duplicate food prep tasks
        let preps = sqlx::query_as!(
            FoodPrep,
            r#"
            SELECT prep_id as "id: FoodPrepId", event_id as "event_id: EventId", recipe_id as "recipe_id: RecipeId", prep_date, use_from, use_until
            FROM food_prep
            WHERE event_id = $1
            "#,
            id.get()
        )
        .fetch_all(&mut *tx)
        .await?;

        for prep in preps {
            sqlx::query!(
                r#"
                INSERT INTO food_prep (event_id, recipe_id, prep_date, use_from, use_until)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                new_event.id.get(),
                prep.recipe_id.get(),
                prep.prep_date,
                prep.use_from,
                prep.use_until,
            )
            .execute(&mut *tx)
            .await?;
        }

        // Duplicate inventory associations
        let inventories = sqlx::query_as!(
            EventInventory,
            r#"
            SELECT event_id as "event_id: EventId", inventory_id as "inventory_id: InventoryId"
            FROM event_inventories
            WHERE event_id = $1
            "#,
            id.get()
        )
        .fetch_all(&mut *tx)
        .await?;

        for inventory in inventories {
            sqlx::query!(
                r#"
                INSERT INTO event_inventories (event_id, inventory_id)
                VALUES ($1, $2)
                "#,
                new_event.id.get(),
                inventory.inventory_id.get(),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(new_event.id)
    }

    /// Lists events visible to a user (those whose group the user belongs to)
    pub async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#"
            SELECT
                event_id as "id: EventId",
                event_name as name,
                events.comment,
                group_id as "group_id: GroupId",
                budget
            FROM events
            LEFT JOIN event_meals USING (event_id)
            WHERE group_id IN (
                SELECT group_id FROM user_groups WHERE user_id = $1
            )
            GROUP BY event_id, event_name, events.comment, group_id, budget
            ORDER BY MIN(start_time) DESC
            "#,
            user_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Lists all events (admin only)
    pub async fn list(&self) -> Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#"
            SELECT
                event_id as "id: EventId",
                event_name as name,
                events.comment,
                group_id as "group_id: GroupId",
                budget
            FROM events LEFT JOIN event_meals USING (event_id)
            GROUP BY event_id, event_name, events.comment, group_id, budget
            ORDER BY MIN(start_time) DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Adds a shopping tour to an event
    pub async fn add_shopping_tour(&self, tour: ShoppingTour) -> Result<ShoppingTour> {
        let record = sqlx::query_as!(
            ShoppingTour,
            r#"
            INSERT INTO shopping_tours (event_id, tour_date, store_id)
            VALUES ($1, $2, $3)
            RETURNING tour_id as "id: ShoppingTourId", event_id as "event_id: EventId", tour_date, store_id as "store_id: StoreId", (SELECT name FROM stores WHERE store_id = $3) as store_name
            "#,
            tour.event_id.get(),
            tour.tour_date,
            tour.store_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Updates a shopping tour's details
    pub async fn update_shopping_tour(&self, tour: ShoppingTour) -> Result<ShoppingTour> {
        let record = sqlx::query_as!(
            ShoppingTour,
            r#"
            UPDATE shopping_tours
            SET tour_date = $2, store_id = $3
            WHERE tour_id = $1
            RETURNING tour_id as "id: ShoppingTourId", event_id as "event_id: EventId", tour_date, store_id as "store_id: StoreId", $4 as store_name
            "#,
            tour.id.get(),
            tour.tour_date,
            tour.store_id.get(),
            tour.store_name,
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "ShoppingTour",
            id: tour.id.to_string(),
        })?;

        Ok(record)
    }

    /// Deletes a shopping tour
    pub async fn delete_shopping_tour(&self, id: ShoppingTourId) -> Result<()> {
        let result = sqlx::query!("DELETE FROM shopping_tours WHERE tour_id = $1", id.get())
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "ShoppingTour",
                id: id.to_string(),
            });
        }

        Ok(())
    }

    /// Gets all shopping tours for an event
    pub async fn get_shopping_tours(&self, event_id: EventId) -> Result<Vec<ShoppingTour>> {
        let records = sqlx::query_as!(
            ShoppingTour,
            r#"
            SELECT tour_id as "id: ShoppingTourId", event_id as "event_id: EventId", tour_date, store_id as "store_id: StoreId", name as store_name
            FROM shopping_tours JOIN stores USING(store_id)
            WHERE event_id = $1
            ORDER BY tour_date
            "#,
            event_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Gets shopping tour by id
    pub async fn get_shopping_tour(&self, tour_id: ShoppingTourId) -> Result<ShoppingTour> {
        let records = sqlx::query_as!(
            ShoppingTour,
            r#"
            SELECT tour_id as "id: ShoppingTourId", event_id as "event_id: EventId", tour_date, store_id as "store_id: StoreId", name as store_name
            FROM shopping_tours JOIN stores USING(store_id)
            WHERE tour_id = $1
            ORDER BY tour_date
            "#,
            tour_id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Metro article numbers for the ingredients on a tour, read from the cache
    /// populated during metro sync (`metro_categories.article_number`). Returns
    /// `ingredient_id -> article_number`. Ingredients whose source has no cached
    /// article number (never synced, or sync found none) are simply absent from
    /// the map; the caller decides how to render those. This is a pure DB read —
    /// run a metro sync to refresh the cache.
    pub async fn resolve_metro_article_numbers(
        &self,
        tour_id: ShoppingTourId,
    ) -> Result<std::collections::HashMap<IngredientId, String>> {
        use crate::ops::metro_sync::METRO_STORE_ID;

        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT s.ingredient_id as "ingredient_id: IngredientId", mc.article_number AS "article_number!"
            FROM shopping_list sl
            JOIN ingredient_sources s ON s.ingredient_id = sl.ingredient_id
            JOIN metro_categories mc ON mc.ingredient_source_id = s.ingredient_source_id
            WHERE sl.tour_id = $1
              AND s.store_id = $2
              AND mc.article_number IS NOT NULL
            "#,
            tour_id.get(),
            METRO_STORE_ID.get(),
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.ingredient_id, r.article_number))
            .collect())
    }

    /// Tour shopping list **with** the per-recipe usage breakdown, from a single
    /// query over `shopping_tour_ingredients` (recipe grain) aggregated to
    /// ingredient totals in Rust. Replaces a separate shopping-list + usage
    /// query pair for the exports — both are aggregations of the same expensive
    /// base view, so we scan it once.
    ///
    /// Price/category are derived exactly as the `shopping_list` view does (the
    /// `best_event_ingredient_sources` and `metro_categories` joins); price is
    /// `per-unit × total weight`. Each ingredient's `recipe_usage` is scoped to
    /// what this tour covers (food-prep splits an ingredient across tours by
    /// `buy_by` time), ordered by descending amount. Items are returned ordered
    /// by category then name, matching the shopping-list ordering.
    pub async fn tour_shopping_list_with_usage(
        &self,
        tour_id: ShoppingTourId,
    ) -> Result<Vec<ShoppingListItemWithUsage>> {
        // One row per (ingredient, recipe). price_per_unit and category repeat
        // across an ingredient's recipe rows (they're ingredient-level); weight
        // is the recipe's share.
        let rows = sqlx::query!(
            r#"
            SELECT sti.ingredient_id AS "ingredient_id!: IngredientId",
                   sti.ingredient AS "ingredient_name!",
                   sti.recipe AS "recipe!",
                   SUM(sti.weight) AS "weight!",
                   beis.price AS price_per_unit,
                   mc.category AS "category?"
            FROM shopping_tour_ingredients sti
            LEFT JOIN best_event_ingredient_sources beis
                USING (ingredient_id, event_id)
            LEFT JOIN metro_categories mc USING (ingredient_source_id)
            WHERE sti.tour_id = $1
            GROUP BY sti.ingredient_id, sti.ingredient, sti.recipe, beis.price, mc.category
            ORDER BY COALESCE(mc.category, ''), sti.ingredient,
                     SUM(sti.weight) DESC, sti.recipe
            "#,
            tour_id.get(),
        )
        .fetch_all(&*self.pool)
        .await?;

        // Aggregate recipe-grain rows into per-ingredient items, preserving the
        // SQL ordering (category, name) for items and (desc weight) for recipes.
        let mut items: Vec<ShoppingListItemWithUsage> = Vec::new();
        for row in rows {
            match items.last_mut() {
                Some(item) if item.ingredient_id == row.ingredient_id => {
                    item.weight += &row.weight;
                    item.recipe_usage.push((row.recipe, row.weight));
                }
                _ => items.push(ShoppingListItemWithUsage {
                    ingredient_id: row.ingredient_id,
                    ingredient_name: row.ingredient_name,
                    weight: row.weight.clone(),
                    // Price is per-unit × total weight; multiply once weight is
                    // fully summed below. Stash per-unit here, finalize after.
                    price: row.price_per_unit.clone(),
                    category: row.category,
                    recipe_usage: vec![(row.recipe, row.weight)],
                }),
            }
        }
        // Finalize price: per-unit × summed weight (matches the shopping_list
        // view's `SUM(weight) * price`). A negative sentinel (-1) means "no
        // source priced", which we surface as None.
        for item in &mut items {
            item.price = item.price.as_ref().and_then(|per_unit| {
                (per_unit >= &BigDecimal::from(0)).then(|| per_unit * &item.weight)
            });
        }
        Ok(items)
    }

    /// Gets the shopping list for a specific tour
    pub async fn get_shopping_list(&self, tour_id: ShoppingTourId) -> Result<Vec<ShoppingListItem>> {
        let records = sqlx::query_as!(
            ShoppingListItem,
            r#"
            SELECT
                event_id as "event_id!: EventId",
                event_name as "event_name!",
                ingredient_id as "ingredient_id!: IngredientId",
                ingredient as "ingredient_name!",
                weight as "weight!",
                energy as "energy!",
                price,
                tour_id as "tour_id: ShoppingTourId",
                category
            FROM shopping_list
            WHERE tour_id = $1
            ORDER BY COALESCE(category, ''), ingredient
            "#,
            tour_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Gets all shopping list items for an event
    pub async fn get_event_shopping_list(&self, event_id: EventId) -> Result<Vec<ShoppingListItem>> {
        let records = sqlx::query_as!(
            ShoppingListItem,
            r#"
            SELECT
                event_id as "event_id!: EventId",
                event_name as "event_name!",
                ingredient_id as "ingredient_id!: IngredientId",
                ingredient as "ingredient_name!",
                weight as "weight!",
                energy as "energy!",
                price,
                tour_id as "tour_id: ShoppingTourId",
                category
            FROM shopping_list
            WHERE event_id = $1
            ORDER BY COALESCE(category, ''), ingredient
            "#,
            event_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Gets all ingredients for an event that don't have a shopping tour
    pub async fn get_ingredients_without_tour(
        &self,
        event_id: EventId,
    ) -> Result<Vec<IngredientWithoutTour>> {
        let items = sqlx::query_as!(
            IngredientWithoutTour,
            r#"
        SELECT
            ingredient_id as "ingredient_id!: IngredientId",
            ingredient as "ingredient_name!",
            sum(weight) as "weight",
            coalesce(sn.name, 'No Store') as "store_name!"
        FROM shopping_tour_ingredients_without_tour
        LEFT JOIN stores sn USING (store_id)
        WHERE event_id = $1
        GROUP BY event_id, ingredient_id, ingredient, store_id, sn.name
        ORDER BY ingredient
        "#,
            event_id.get()
        )
        .fetch_all(&*self.pool)
        .await
        .unwrap_or_default();

        Ok(items)
    }

    /// Adds a food preparation task
    pub async fn add_food_prep(&self, prep: FoodPrep) -> Result<FoodPrep> {
        let record = sqlx::query_as!(
            FoodPrep,
            r#"
            INSERT INTO food_prep (
                event_id, recipe_id, prep_date, use_from, use_until
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                prep_id as "id: FoodPrepId",
                event_id as "event_id: EventId",
                recipe_id as "recipe_id: RecipeId",
                prep_date,
                use_from,
                use_until
            "#,
            prep.event_id.get(),
            prep.recipe_id.get(),
            prep.prep_date,
            prep.use_from,
            prep.use_until,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Updates a food preparation task
    pub async fn update_food_prep(&self, prep: FoodPrep) -> Result<FoodPrep> {
        let record = sqlx::query_as!(
            FoodPrep,
            r#"
            UPDATE food_prep
            SET
                recipe_id = $2,
                prep_date = $3,
                use_from = $4,
                use_until = $5
            WHERE prep_id = $1
            RETURNING
                prep_id as "id: FoodPrepId",
                event_id as "event_id: EventId",
                recipe_id as "recipe_id: RecipeId",
                prep_date,
                use_from,
                use_until
            "#,
            prep.id.get(),
            prep.recipe_id.get(),
            prep.prep_date,
            prep.use_from,
            prep.use_until,
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "FoodPrep",
            id: prep.id.to_string(),
        })?;

        Ok(record)
    }
    pub async fn get_food_prep(&self, prep_id: FoodPrepId) -> Result<FoodPrep> {
        let record = sqlx::query_as!(
            FoodPrep,
            r#"
            SELECT
                prep_id as "id: FoodPrepId",
                event_id as "event_id: EventId",
                recipe_id as "recipe_id: RecipeId",
                prep_date,
                use_from,
                use_until
            FROM food_prep
            WHERE prep_id = $1
            "#,
            prep_id.get(),
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "FoodPrep",
            id: prep_id.to_string(),
        })?;

        Ok(record)
    }

    /// Shifts every dated row attached to `event_id` by the delta that takes
    /// the earliest meal's start_time to `new_first_meal_start`. Returns the
    /// applied delta. No-op (returns `Duration::ZERO`) when the event has no
    /// meals — without a meal there is no anchor to shift from.
    pub async fn shift_event_times(
        &self,
        event_id: EventId,
        new_first_meal_start: OffsetDateTime,
    ) -> Result<time::Duration> {
        let mut tx = self.pool.begin().await?;

        let earliest: Option<OffsetDateTime> = sqlx::query_scalar!(
            "SELECT MIN(start_time) FROM event_meals WHERE event_id = $1",
            event_id.get(),
        )
        .fetch_one(&mut *tx)
        .await?;

        let Some(earliest) = earliest else {
            return Ok(time::Duration::ZERO);
        };
        let delta = new_first_meal_start - earliest;
        if delta.is_zero() {
            return Ok(delta);
        }

        // Postgres `interval` round-trips from time::Duration via sqlx, and
        // `NULL + interval` is `NULL`, so nullable columns (`use_from`) need
        // no special-casing.
        sqlx::query!(
            "UPDATE event_meals SET start_time = start_time + $2, end_time = end_time + $2 WHERE event_id = $1",
            event_id.get(),
            delta as _,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE food_prep SET prep_date = prep_date + $2, use_from = use_from + $2, use_until = use_until + $2 WHERE event_id = $1",
            event_id.get(),
            delta as _,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE shopping_tours SET tour_date = tour_date + $2 WHERE event_id = $1",
            event_id.get(),
            delta as _,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(delta)
    }

    /// Deletes a food preparation task
    pub async fn delete_food_prep(&self, id: FoodPrepId) -> Result<()> {
        let result = sqlx::query!("DELETE FROM food_prep WHERE prep_id = $1", id.get())
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "FoodPrep",
                id: id.to_string(),
            });
        }

        Ok(())
    }

    /// Gets all food preparation tasks for an event
    pub async fn get_food_prep_tasks(&self, event_id: EventId) -> Result<Vec<FoodPrep>> {
        let records = sqlx::query_as!(
            FoodPrep,
            r#"
            SELECT
                prep_id as "id: FoodPrepId",
                event_id as "event_id: EventId",
                recipe_id as "recipe_id: RecipeId",
                prep_date,
                use_from,
                use_until
            FROM food_prep
            WHERE event_id = $1
            ORDER BY prep_date
            "#,
            event_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Manages source overrides for an event
    pub async fn set_source_override(
        &self,
        event_id: EventId,
        source_id: IngredientSourceId,
    ) -> Result<SourceOverride> {
        let record = sqlx::query_as!(
            SourceOverride,
            r#"
            INSERT INTO event_source_overrides (event_id, ingredient_source_id)
            VALUES ($1, $2)
            ON CONFLICT (event_id, ingredient_source_id) DO UPDATE
            SET event_id = $1
            RETURNING event_id as "event_id: EventId", ingredient_source_id as "source_id: IngredientSourceId"
            "#,
            event_id.get(),
            source_id.get(),
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Removes a source override
    pub async fn delete_source_override(&self, event_id: EventId, source_id: IngredientSourceId) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM event_source_overrides
            WHERE event_id = $1 AND ingredient_source_id = $2
            "#,
            event_id.get(),
            source_id.get(),
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "SourceOverride",
                id: format!("event:{} source:{}", event_id, source_id),
            });
        }

        Ok(())
    }

    /// Gets all source overrides for an event with detailed information
    pub async fn get_source_overrides(&self, event_id: EventId) -> Result<Vec<SourceOverrideView>> {
        let records = sqlx::query_as!(
            SourceOverrideView,
            r#"
            SELECT
                event_id as "event_id: EventId",
                ingredient_id as "ingredient_id: IngredientId",
                ingredient_sources.ingredient_source_id as "source_id: IngredientSourceId",
                ingredients.name as ingredient_name,
                stores.store_id as "store_id: StoreId",
                stores.name as store_name
            FROM event_source_overrides
            INNER JOIN ingredient_sources USING (ingredient_source_id)
            INNER JOIN ingredients USING (ingredient_id)
            INNER JOIN stores USING (store_id)
            WHERE event_id = $1
            ORDER BY ingredients.name
            "#,
            event_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Associates an inventory with an event
    pub async fn add_inventory(&self, event_id: EventId, inventory_id: InventoryId) -> Result<EventInventory> {
        let record = sqlx::query_as!(
            EventInventory,
            r#"
            INSERT INTO event_inventories (event_id, inventory_id)
            VALUES ($1, $2)
            RETURNING event_id as "event_id: EventId", inventory_id as "inventory_id: InventoryId"
            "#,
            event_id.get(),
            inventory_id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Removes an inventory association from an event
    pub async fn remove_inventory(&self, event_id: EventId, inventory_id: InventoryId) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM event_inventories
            WHERE event_id = $1 AND inventory_id = $2
            "#,
            event_id.get(),
            inventory_id.get()
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound {
                entity: "EventInventory",
                id: format!("event:{} inventory:{}", event_id, inventory_id),
            });
        }

        Ok(())
    }

    /// Gets all inventories associated with an event
    pub async fn get_inventories(&self, event_id: EventId) -> Result<Vec<EventInventory>> {
        let records = sqlx::query_as!(
            EventInventory,
            r#"
            SELECT event_id as "event_id: EventId", inventory_id as "inventory_id: InventoryId"
            FROM event_inventories
            WHERE event_id = $1
            ORDER BY inventory_id
            "#,
            event_id.get()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Calculates the total cost for an event
    pub async fn get_total_cost(&self, event_id: EventId) -> Result<BigDecimal> {
        let record = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(price), 0) as "total!"
            FROM event_ingredients
            WHERE event_id = $1
            "#,
            event_id.get()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record.total)
    }

    /// Picks the event to show by default at the given time: an ongoing event
    /// (window contains `at`) ending soonest, else the soonest upcoming event.
    /// Fully-past events are never returned; `None` means nothing is scheduled.
    pub async fn get_current_or_upcoming_event(&self, at: OffsetDateTime) -> Result<Option<Event>> {
        let record = sqlx::query_as!(
            Event,
            r#"
            SELECT
                e.event_id as "id: EventId",
                e.event_name as name,
                e.comment,
                e.group_id as "group_id: GroupId",
                e.budget 
            FROM events e
            LEFT JOIN event_meals em ON e.event_id = em.event_id
            LEFT JOIN food_prep fp ON e.event_id = fp.event_id
            GROUP BY (e.event_id, e.event_name, e.comment, e.group_id, e.budget)
            HAVING MAX(GREATEST(em.end_time, fp.use_until)) >= $1
            ORDER BY
                (MIN(LEAST(em.start_time, fp.prep_date)) > $1),
                CASE
                    WHEN MIN(LEAST(em.start_time, fp.prep_date)) <= $1
                        THEN MAX(GREATEST(em.end_time, fp.use_until))
                    ELSE MIN(LEAST(em.start_time, fp.prep_date))
                END
            LIMIT 1
            "#,
            at
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Gets past events (those with no future meals or prep tasks after the given date)
    pub async fn get_past_events(&self, before: OffsetDateTime) -> Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#"
            SELECT DISTINCT
                e.event_id as id,
                e.event_name as name,
                e.comment,
                e.group_id,
                e.budget
            FROM events e
            LEFT JOIN event_meals em ON e.event_id = em.event_id
            LEFT JOIN food_prep fp ON e.event_id = fp.event_id
            GROUP BY e.event_id, e.event_name, e.comment, e.group_id, e.budget
            HAVING MAX(GREATEST(em.end_time, fp.use_until)) < $1
            ORDER BY e.event_name
            "#,
            before
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Searches for events by name
    pub async fn search_by_name(&self, query: &str) -> Result<Vec<Event>> {
        let query = format!("%{}%", query.to_lowercase());
        let records = sqlx::query_as!(
            Event,
            r#"
            SELECT
                event_id as id,
                event_name as name,
                comment,
                group_id,
                budget
            FROM events
            WHERE LOWER(event_name) LIKE $1
            ORDER BY event_name
            "#,
            query
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_places(&self) -> Result<Vec<crate::entities::event::Place>> {
        let records = sqlx::query_as!(
            crate::entities::event::Place,
            r#"SELECT place_id as id, name, comment FROM places ORDER BY name"#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(records)
    }
}
