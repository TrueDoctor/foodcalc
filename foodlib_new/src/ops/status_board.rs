// foodlib_new/src/ops/status_board.rs
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

use crate::{
    ops::{events::EventOps, meals::MealOps},
    Error,
};

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
    pub urgency: String,
    pub status: String,
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
    pub status: String,
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
    pub availability: String,
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
    availability: HashMap<i32, (String, Option<i32>)>,
}

fn live_store() -> &'static RwLock<HashMap<i32, LiveEventState>> {
    static STORE: OnceLock<RwLock<HashMap<i32, LiveEventState>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

impl StatusBoardOps {
    pub async fn get_board(&self, event_id: i32) -> Result<StatusBoardSnapshot, Error> {
        let now = OffsetDateTime::now_utc();

        // Reuses EventOps::get & MealOps::get_event_meals_planner (returns clean 404 when missing)
        let event = EventOps::new(self.pool.clone()).get(event_id).await?;
        let meals = MealOps::new(self.pool.clone())
            .get_event_meals_planner(event_id)
            .await?;

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

        let mut current = Vec::new();
        let mut upcoming = Vec::new();
        let mut outgoing = Vec::new();

        for meal in meals {
            // Keep meals in [-2h, +8h] around now
            if meal.end_time < now - Duration::hours(2)
                || meal.start_time > now + Duration::hours(8)
            {
                continue;
            }

            let (availability, eta_minutes) = availability_map
                .get(&meal.meal_id)
                .cloned()
                .unwrap_or_else(|| ("available".to_string(), None));

            let dish = ServingDishItem {
                meal_id: meal.meal_id,
                dish_name: meal.name.clone(),
                dish_name_en: None,
                description: meal.comment.filter(|c| !c.trim().is_empty()),
                serving_start: meal.start_time,
                serving_end: Some(meal.end_time),
                availability: availability.clone(),
                eta_minutes,
                properties: Vec::new(),
            };

            let is_current = meal.start_time <= now + Duration::minutes(15)
                && meal.end_time >= now - Duration::minutes(15);

            if is_current {
                current.push(dish);
            } else if meal.start_time > now + Duration::minutes(15) {
                upcoming.push(dish);
            }

            if meal.start_time >= now - Duration::minutes(15)
                && meal.start_time <= now + Duration::hours(3)
            {
                outgoing.push(OutgoingBatchItem {
                    meal_id: meal.meal_id,
                    dish_name: meal.name,
                    serving_time: meal.start_time,
                    portions: meal.servings,
                    availability,
                    ready_for_dispatch: meal.start_time <= now + Duration::minutes(20),
                });
            }
        }

        Ok(StatusBoardSnapshot {
            event_name: event.name,
            server_time: now,
            kitchen: KitchenBoardSection {
                reorders,
                outgoing,
                prep_tasks: Vec::new(),
            },
            serving: ServingBoardSection { current, upcoming },
        })
    }

    pub async fn create_reorder(
        &self,
        event_id: i32,
        input: CreateReorderInput,
    ) -> Result<i32, Error> {
        let meals = MealOps::new(self.pool.clone())
            .get_event_meals_planner(event_id)
            .await?;
        let meal = meals
            .into_iter()
            .find(|m| m.meal_id == input.meal_id)
            .ok_or_else(|| Error::NotFound {
                id: input.meal_id.to_string(),
                entity: "meal",
            })?;

        let mut guard = live_store().write().expect("live_store lock poisoned");
        let state = guard.entry(event_id).or_default();
        state.next_reorder_id += 1;
        let id = state.next_reorder_id;

        state.reorders.push(KitchenReorderItem {
            id,
            meal_id: input.meal_id,
            dish_name: meal.name,
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