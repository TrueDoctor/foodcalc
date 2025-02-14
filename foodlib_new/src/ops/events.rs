use crate::entities::event::*;
use crate::error::{Error, Result};
use crate::meal::Meal;
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::sync::Arc;
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
            INSERT INTO events (event_name, comment, budget)
            VALUES ($1, $2, $3)
            RETURNING event_id as id, event_name as name, comment, budget
            "#,
            event.name,
            event.comment,
            event.budget,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Retrieves an event by ID
    pub async fn get(&self, id: i32) -> Result<Event> {
        let record = sqlx::query_as!(
            Event,
            r#"
            SELECT 
                event_id as id,
                event_name as name,
                comment,
                budget
            FROM events 
            WHERE event_id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Event",
            id: id.to_string(),
        })?;

        Ok(record)
    }

    /// Updates an existing event
    pub async fn update(&self, event: Event) -> Result<Event> {
        let record = sqlx::query_as!(
            Event,
            r#"
            UPDATE events 
            SET event_name = $1, comment = $2, budget = $3
            WHERE event_id = $4
            RETURNING event_id as id, event_name as name, comment, budget
            "#,
            event.name,
            event.comment,
            event.budget,
            event.id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Event",
            id: event.id.to_string(),
        })?;

        Ok(record)
    }

    /// Deletes an event and all associated data
    pub async fn delete(&self, id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete associated data first
        sqlx::query!("DELETE FROM event_meals WHERE event_id = $1", id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM event_source_overrides WHERE event_id = $1", id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM shopping_tours WHERE event_id = $1", id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM food_prep WHERE event_id = $1", id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM event_inventories WHERE event_id = $1", id)
            .execute(&mut *tx)
            .await?;

        // Finally delete the event
        let result = sqlx::query!("DELETE FROM events WHERE event_id = $1", id)
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
    pub async fn duplicate(&self, id: i32) -> Result<i32> {
        let mut tx = self.pool.begin().await?;

        // Get the event details
        let event = sqlx::query_as!(
            Event,
            r#"
            SELECT 
                event_id as id,
                event_name as name,
                comment,
                budget
            FROM events 
            WHERE event_id = $1
            "#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        let new_name = format!("{} (Copy)", event.name);

        // Create a new event
        let new_event = sqlx::query_as!(
            Event,
            r#"
            INSERT INTO events (event_name, comment, budget)
            VALUES ($1, $2, $3)
            RETURNING event_id as id, event_name as name, comment, budget
            "#,
            new_name,
            event.comment,
            event.budget,
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
                    new_event.id,
                    meal.recipe_id,
                    meal.place_id,
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
            SELECT event_id, ingredient_source_id as source_id
            FROM event_source_overrides
            WHERE event_id = $1
            "#,
            id
        )
        .fetch_all(&mut *tx)
        .await?;

        for source_override in overrides {
            sqlx::query!(
                r#"
                INSERT INTO event_source_overrides (event_id, ingredient_source_id)
                VALUES ($1, $2)
                "#,
                new_event.id,
                source_override.source_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        // TODO Duplicate shopping tours
        let tours = sqlx::query_as!(
            ShoppingTour,
            r#"
            SELECT tour_id as id, event_id, tour_date, store_id
            FROM shopping_tours
            WHERE event_id = $1
            "#,
            id
        )
        .fetch_all(&mut *tx)
        .await?;

        for tour in tours {
            sqlx::query!(
                r#"
                INSERT INTO shopping_tours (event_id, tour_date, store_id)
                VALUES ($1, $2, $3)
                "#,
                new_event.id,
                tour.tour_date,
                tour.store_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        // Duplicate food prep tasks
        let preps = sqlx::query_as!(
            FoodPrep,
            r#"
            SELECT prep_id as id, event_id, recipe_id, prep_date, use_from, use_until
            FROM food_prep
            WHERE event_id = $1
            "#,
            id
        )
        .fetch_all(&mut *tx)
        .await?;

        for prep in preps {
            sqlx::query!(
                r#"
                INSERT INTO food_prep (event_id, recipe_id, prep_date, use_from, use_until)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                new_event.id,
                prep.recipe_id,
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
            SELECT event_id, inventory_id
            FROM event_inventories
            WHERE event_id = $1
            "#,
            id
        )
        .fetch_all(&mut *tx)
        .await?;

        for inventory in inventories {
            sqlx::query!(
                r#"
                INSERT INTO event_inventories (event_id, inventory_id)
                VALUES ($1, $2)
                "#,
                new_event.id,
                inventory.inventory_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(new_event.id)
    }

    /// Lists all events
    pub async fn list(&self) -> Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#"
            SELECT 
                event_id as id,
                event_name as name,
                comment,
                budget
            FROM events 
            ORDER BY event_name
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
            RETURNING tour_id as id, event_id, tour_date, store_id
            "#,
            tour.event_id,
            tour.tour_date,
            tour.store_id,
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
            RETURNING tour_id as id, event_id, tour_date, store_id
            "#,
            tour.id,
            tour.tour_date,
            tour.store_id,
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
    pub async fn delete_shopping_tour(&self, id: i32) -> Result<()> {
        let result = sqlx::query!("DELETE FROM shopping_tours WHERE tour_id = $1", id)
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
    pub async fn get_shopping_tours(&self, event_id: i32) -> Result<Vec<ShoppingTour>> {
        let records = sqlx::query_as!(
            ShoppingTour,
            r#"
            SELECT tour_id as id, event_id, tour_date, store_id
            FROM shopping_tours
            WHERE event_id = $1
            ORDER BY tour_date
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Gets the shopping list for a specific tour
    pub async fn get_shopping_list(&self, tour_id: i32) -> Result<Vec<ShoppingListItem>> {
        let records = sqlx::query_as!(
            ShoppingListItem,
            r#"
            SELECT 
                event_id as "event_id!",
                event_name as "event_name!",
                ingredient_id as "ingredient_id!",
                ingredient as "ingredient_name!",
                weight as "weight!",
                energy as "energy!",
                price,
                tour_id,
                category
            FROM shopping_list
            WHERE tour_id = $1
            ORDER BY COALESCE(category, ''), ingredient
            "#,
            tour_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Gets all shopping list items for an event
    pub async fn get_event_shopping_list(&self, event_id: i32) -> Result<Vec<ShoppingListItem>> {
        let records = sqlx::query_as!(
            ShoppingListItem,
            r#"
            SELECT 
                event_id as "event_id!",
                event_name as "event_name!",
                ingredient_id as "ingredient_id!",
                ingredient as "ingredient_name!",
                weight as "weight!",
                energy as "energy!",
                price,
                tour_id,
                category
            FROM shopping_list
            WHERE event_id = $1
            ORDER BY COALESCE(category, ''), ingredient
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
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
                prep_id as id,
                event_id,
                recipe_id,
                prep_date,
                use_from,
                use_until
            "#,
            prep.event_id,
            prep.recipe_id,
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
                prep_id as id,
                event_id,
                recipe_id,
                prep_date,
                use_from,
                use_until
            "#,
            prep.id,
            prep.recipe_id,
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

    /// Deletes a food preparation task
    pub async fn delete_food_prep(&self, id: i32) -> Result<()> {
        let result = sqlx::query!("DELETE FROM food_prep WHERE prep_id = $1", id)
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
    pub async fn get_food_prep_tasks(&self, event_id: i32) -> Result<Vec<FoodPrep>> {
        let records = sqlx::query_as!(
            FoodPrep,
            r#"
            SELECT 
                prep_id as id,
                event_id,
                recipe_id,
                prep_date,
                use_from,
                use_until
            FROM food_prep
            WHERE event_id = $1
            ORDER BY prep_date
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Manages source overrides for an event
    pub async fn set_source_override(
        &self,
        event_id: i32,
        source_id: i32,
    ) -> Result<SourceOverride> {
        let record = sqlx::query_as!(
            SourceOverride,
            r#"
            INSERT INTO event_source_overrides (event_id, ingredient_source_id)
            VALUES ($1, $2)
            ON CONFLICT (event_id, ingredient_source_id) DO UPDATE
            SET event_id = $1
            RETURNING event_id, ingredient_source_id as source_id
            "#,
            event_id,
            source_id,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Removes a source override
    pub async fn delete_source_override(&self, event_id: i32, source_id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM event_source_overrides 
            WHERE event_id = $1 AND ingredient_source_id = $2
            "#,
            event_id,
            source_id,
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
    pub async fn get_source_overrides(&self, event_id: i32) -> Result<Vec<SourceOverrideView>> {
        let records = sqlx::query_as!(
            SourceOverrideView,
            r#"
            SELECT 
                event_id,
                ingredient_id,
                ingredient_sources.ingredient_source_id as source_id,
                ingredients.name as ingredient_name,
                stores.store_id,
                stores.name as store_name
            FROM event_source_overrides
            INNER JOIN ingredient_sources USING (ingredient_source_id)
            INNER JOIN ingredients USING (ingredient_id)
            INNER JOIN stores USING (store_id)
            WHERE event_id = $1
            ORDER BY ingredients.name
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Associates an inventory with an event
    pub async fn add_inventory(&self, event_id: i32, inventory_id: i32) -> Result<EventInventory> {
        let record = sqlx::query_as!(
            EventInventory,
            r#"
            INSERT INTO event_inventories (event_id, inventory_id)
            VALUES ($1, $2)
            RETURNING event_id, inventory_id
            "#,
            event_id,
            inventory_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record)
    }

    /// Removes an inventory association from an event
    pub async fn remove_inventory(&self, event_id: i32, inventory_id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM event_inventories 
            WHERE event_id = $1 AND inventory_id = $2
            "#,
            event_id,
            inventory_id
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
    pub async fn get_inventories(&self, event_id: i32) -> Result<Vec<EventInventory>> {
        let records = sqlx::query_as!(
            EventInventory,
            r#"
            SELECT event_id, inventory_id
            FROM event_inventories
            WHERE event_id = $1
            "#,
            event_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
    }

    /// Calculates the total cost for an event
    pub async fn get_total_cost(&self, event_id: i32) -> Result<BigDecimal> {
        let record = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(price), 0) as "total!"
            FROM event_ingredients
            WHERE event_id = $1
            "#,
            event_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(record.total)
    }

    /// Gets upcoming events (those with meals or prep tasks after the given date)
    pub async fn get_upcoming_events(&self, after: OffsetDateTime) -> Result<Vec<Event>> {
        let records = sqlx::query_as!(
            Event,
            r#"
            SELECT DISTINCT
                e.event_id as id,
                e.event_name as name,
                e.comment,
                e.budget
            FROM events e
            LEFT JOIN event_meals em ON e.event_id = em.event_id
            LEFT JOIN food_prep fp ON e.event_id = fp.event_id
            WHERE em.start_time >= $1 OR fp.prep_date >= $1
            ORDER BY e.event_name
            "#,
            after
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(records)
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
                e.budget
            FROM events e
            LEFT JOIN event_meals em ON e.event_id = em.event_id
            LEFT JOIN food_prep fp ON e.event_id = fp.event_id
            GROUP BY e.event_id, e.event_name, e.comment, e.budget
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
}
