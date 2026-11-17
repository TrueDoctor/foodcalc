use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use foodlib_new::Error;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};

pub struct StatusBoardOps {
    pool: Arc<PgPool>,
}

impl StatusBoardOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenReorderItem {
    pub id: i32,
    pub meal_id: i32,
    pub dish_name: String,
    pub amount_label: String,
    pub station: String,
    pub urgency: String, // "normal" | "urgent" | "critical"
    pub status: String,  // "requested" | "preparing" | "en_route" | "delivered"
    pub note: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub requested_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingBatchItem {
    pub meal_id: i32,
    pub dish_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub serving_time: OffsetDateTime,
    pub portions: i32,
    pub availability: String,
    pub ready_for_dispatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenPrepTask {
    pub id: String,
    pub title: String,
    pub dish_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub target_time: OffsetDateTime,
    pub quantity_label: Option<String>,
    pub assignee: Option<String>,
    pub status: String, // "todo" | "in_progress"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DishPropertyTag {
    pub id: i32,
    pub name: String,
    pub code: Option<String>,
    pub is_allergen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingDishItem {
    pub meal_id: i32,
    pub dish_name: String,
    pub dish_name_en: Option<String>,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub serving_start: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub serving_end: Option<OffsetDateTime>,
    pub availability: String, // "available" | "low" | "replenishing" | "sold_out"
    pub eta_minutes: Option<i32>,
    pub properties: Vec<DishPropertyTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitchenBoardSection {
    pub reorders: Vec<KitchenReorderItem>,
    pub outgoing: Vec<OutgoingBatchItem>,
    pub prep_tasks: Vec<KitchenPrepTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingBoardSection {
    pub current: Vec<ServingDishItem>,
    pub upcoming: Vec<ServingDishItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBoardSnapshot {
    pub event_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub server_time: OffsetDateTime,
    pub kitchen: KitchenBoardSection,
    pub serving: ServingBoardSection,
}

#[derive(Debug, Deserialize)]
pub struct CreateReorderInput {
    pub meal_id: i32,
    pub amount_label: String,
    #[serde(default = "default_station")]
    pub station: String,
    #[serde(default = "default_urgency")]
    pub urgency: String,
    pub note: Option<String>,
}

fn default_station() -> String {
    "Ausgabe".to_string()
}

fn default_urgency() -> String {
    "normal".to_string()
}

#[derive(Debug, Default)]
struct LiveEventState {
    next_reorder_id: i32,
    reorders: Vec<KitchenReorderItem>,
    /// meal_id -> (availability, eta_minutes)
    availability: HashMap<i32, (String, Option<i32>)>,
}

fn live_store() -> &'static RwLock<HashMap<i32, LiveEventState>> {
    static STORE: OnceLock<RwLock<HashMap<i32, LiveEventState>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

impl StatusBoardOps {
    pub async fn get_board(&self, event_id: i32) -> Result<StatusBoardSnapshot, Error> {
        let now = OffsetDateTime::now_utc();
        let pool = self.pool.as_ref();

        let event_row = sqlx::query("SELECT name FROM events WHERE id = $1")
            .bind(event_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                id: event_id.to_string(),
                entity: "event",
            })?;
        let event_name: String = event_row.get("name");

        // Snapshot ephemeral reorders & availability overrides from memory
        let (reorders, availability_map) = {
            let guard = live_store().read().expect("live_store lock poisoned");
            if let Some(live) = guard.get(&event_id) {
                let mut active: Vec<KitchenReorderItem> = live
                    .reorders
                    .iter()
                    .filter(|r| r.status != "delivered")
                    .cloned()
                    .collect();
                active.sort_by_key(|r| match r.urgency.as_str() {
                    "critical" => 0,
                    "urgent" => 1,
                    _ => 2,
                });
                (active, live.availability.clone())
            } else {
                (Vec::new(), HashMap::new())
            }
        };

        // Scheduled meals around `now` (existing `meals` + `recipes` tables only)
        let window_start = now - Duration::hours(2);
        let window_end = now + Duration::hours(8);

        let meal_rows = sqlx::query(
            r#"
            SELECT m.id AS meal_id,
                   m.recipe_id,
                   rec.name AS dish_name,
                   NULLIF(rec.comment, '') AS description,
                   m.start_time,
                   m.end_time,
                   m.servings
            FROM meals m
            JOIN recipes rec ON rec.id = m.recipe_id
            WHERE m.event_id = $1
              AND m.end_time >= $2
              AND m.start_time <= $3
            ORDER BY m.start_time ASC, m.id ASC
            "#,
        )
            .bind(event_id)
            .bind(window_start)
            .bind(window_end)
            .fetch_all(pool)
            .await?;

        let mut current = Vec::new();
        let mut upcoming = Vec::new();
        let mut outgoing = Vec::new();

        for row in meal_rows {
            let meal_id: i32 = row.get("meal_id");
            let recipe_id: i32 = row.get("recipe_id");
            let dish_name: String = row.get("dish_name");
            let description: Option<String> = row.get("description");
            let start_time: OffsetDateTime = row.get("start_time");
            let end_time: OffsetDateTime = row.get("end_time");
            let servings: i32 = row.get("servings");

            let (availability, eta_minutes) = availability_map
                .get(&meal_id)
                .cloned()
                .unwrap_or_else(|| ("available".to_string(), None));

            let properties = load_recipe_tags(pool, recipe_id).await?;

            let dish = ServingDishItem {
                meal_id,
                dish_name: dish_name.clone(),
                dish_name_en: None,
                description,
                serving_start: start_time,
                serving_end: Some(end_time),
                availability: availability.clone(),
                eta_minutes,
                properties,
            };

            let is_current = start_time <= now + Duration::minutes(15)
                && end_time >= now - Duration::minutes(15);

            if is_current {
                current.push(dish);
            } else if start_time > now + Duration::minutes(15) {
                upcoming.push(dish);
            }

            if start_time >= now - Duration::minutes(15)
                && start_time <= now + Duration::hours(3)
            {
                outgoing.push(OutgoingBatchItem {
                    meal_id,
                    dish_name,
                    serving_time: start_time,
                    portions: servings,
                    availability,
                    ready_for_dispatch: start_time <= now + Duration::minutes(20),
                });
            }
        }

        // Prep tasks from existing `food_preps` table
        let prep_rows = sqlx::query(
            r#"
            SELECT fp.id, rec.name AS recipe_name, fp.prep_date
            FROM food_preps fp
            JOIN recipes rec ON rec.id = fp.recipe_id
            WHERE fp.event_id = $1
              AND fp.use_until >= $2
              AND fp.prep_date <= $3
            ORDER BY fp.prep_date ASC
            "#,
        )
            .bind(event_id)
            .bind(now - Duration::hours(1))
            .bind(now + Duration::hours(6))
            .fetch_all(pool)
            .await?;

        let prep_tasks: Vec<KitchenPrepTask> = prep_rows
            .into_iter()
            .map(|row| {
                let id: i32 = row.get("id");
                let recipe_name: String = row.get("recipe_name");
                let prep_date: OffsetDateTime = row.get("prep_date");
                KitchenPrepTask {
                    id: format!("prep-{id}"),
                    title: recipe_name.clone(),
                    dish_name: recipe_name,
                    target_time: prep_date,
                    quantity_label: None,
                    assignee: None,
                    status: if prep_date <= now {
                        "in_progress".to_string()
                    } else {
                        "todo".to_string()
                    },
                }
            })
            .collect();

        Ok(StatusBoardSnapshot {
            event_name,
            server_time: now,
            kitchen: KitchenBoardSection {
                reorders,
                outgoing,
                prep_tasks,
            },
            serving: ServingBoardSection { current, upcoming },
        })
    }

    pub async fn create_reorder(
        &self,
        event_id: i32,
        input: CreateReorderInput,
    ) -> Result<i32, Error> {
        let row = sqlx::query(
            r#"
            SELECT rec.name AS dish_name
            FROM meals m
            JOIN recipes rec ON rec.id = m.recipe_id
            WHERE m.id = $1 AND m.event_id = $2
            "#,
        )
            .bind(input.meal_id)
            .bind(event_id)
            .fetch_optional(self.pool.as_ref())
            .await?
            .ok_or_else(|| Error::NotFound {
                id: input.meal_id.to_string(),
                entity: "meal",
            })?;
        let dish_name: String = row.get("dish_name");

        let mut guard = live_store().write().expect("live_store lock poisoned");
        let state = guard.entry(event_id).or_default();
        state.next_reorder_id += 1;
        let id = state.next_reorder_id;

        state.reorders.push(KitchenReorderItem {
            id,
            meal_id: input.meal_id,
            dish_name,
            amount_label: input.amount_label.trim().to_string(),
            station: input.station.trim().to_string(),
            urgency: input.urgency,
            status: "requested".to_string(),
            note: input.note,
            requested_at: OffsetDateTime::now_utc(),
        });

        Ok(id)
    }

    pub fn update_reorder_status(
        &self,
        event_id: i32,
        reorder_id: i32,
        status: &str,
    ) -> Result<(), Error> {
        let mut guard = live_store().write().expect("live_store lock poisoned");
        let state = guard.get_mut(&event_id).ok_or_else(|| Error::NotFound {
            id: reorder_id.to_string(),
            entity: "reorder",
        })?;

        let reorder = state
            .reorders
            .iter_mut()
            .find(|r| r.id == reorder_id)
            .ok_or_else(|| Error::NotFound {
                id: reorder_id.to_string(),
                entity: "reorder",
            })?;

        reorder.status = status.to_string();
        let meal_id = reorder.meal_id;

        if status == "en_route" {
            state
                .availability
                .insert(meal_id, ("replenishing".to_string(), Some(5)));
        } else if status == "delivered" {
            state
                .availability
                .insert(meal_id, ("available".to_string(), None));
        }

        Ok(())
    }

    pub fn set_meal_availability(
        &self,
        event_id: i32,
        meal_id: i32,
        availability: &str,
        eta_minutes: Option<i32>,
    ) -> Result<(), Error> {
        let mut guard = live_store().write().expect("live_store lock poisoned");
        let state = guard.entry(event_id).or_default();
        state
            .availability
            .insert(meal_id, (availability.to_string(), eta_minutes));
        Ok(())
    }
}

async fn load_recipe_tags(pool: &PgPool, recipe_id: i32) -> Result<Vec<DishPropertyTag>, Error> {
    let rows = sqlx::query(
        r#"
        WITH RECURSIVE tree(rid) AS (
            SELECT $1
            UNION
            SELECT sr.child_recipe_id
            FROM recipe_sub_recipes sr
            JOIN tree t ON t.rid = sr.parent_recipe_id
        )
        SELECT DISTINCT p.id, p.name
        FROM tree t
        JOIN recipe_ingredients ri ON ri.recipe_id = t.rid
        JOIN ingredient_properties ip ON ip.ingredient_id = ri.ingredient_id
        JOIN properties p ON p.id = ip.property_id
        ORDER BY p.name ASC
        "#,
    )
        .bind(recipe_id)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("name");
            let is_allergen = name.starts_with("Allergen:") || name.contains("Gluten");
            DishPropertyTag {
                id: row.get("id"),
                name,
                code: None,
                is_allergen,
            }
        })
        .collect())
}