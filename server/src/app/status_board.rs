//! Status board use-cases: kitchen/serving board snapshot, reorders and
//! meal availability. Validation of the free-form status/urgency/availability
//! strings lives here so every transport shares the same rules.

use super::{AppCtx, Error, MyAppState, Result};
use foodlib_new::entities::types::ids::{EventId, MealId, ReorderId};
use foodlib_new::ops::status_board::{CreateReorderInput, StatusBoardSnapshot};
use foodlib_new::policy::EventPerm;
use serde::Deserialize;

const URGENCIES: [&str; 3] = ["normal", "urgent", "critical"];
const REORDER_STATUSES: [&str; 4] = ["requested", "preparing", "en_route", "delivered"];
const AVAILABILITIES: [&str; 4] = ["available", "low", "replenishing", "sold_out"];


pub async fn get_board(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<StatusBoardSnapshot> {
    actor.ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    state.status_board().get_board(event_id).await
}

pub async fn create_reorder(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    input: CreateReorderInput,
) -> Result<ReorderId> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ManageMeals)
        .await?;

    if input.amount_label.trim().is_empty() {
        return Err(Error::Validation {
            message: "amount_label must not be empty".into(),
        });
    }
    if !URGENCIES.contains(&input.urgency.as_str()) {
        return Err(Error::Validation {
            message: format!("invalid urgency: {}", input.urgency),
        });
    }
    state.status_board().create_reorder(event_id, input).await
}

pub async fn update_reorder_status(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    reorder_id: ReorderId,
    status: &str,
) -> Result<()> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ManageMeals)
        .await?;

    if !REORDER_STATUSES.contains(&status) {
        return Err(Error::Validation {
            message: format!("invalid status: {status}"),
        });
    }
    state
        .status_board()
        .update_reorder_status(event_id, reorder_id, status)
}

#[derive(Debug, Deserialize)]
pub struct SetAvailabilityInput {
    pub availability: String,
    pub eta_minutes: Option<i32>,
}

pub async fn set_meal_availability(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
    input: SetAvailabilityInput,
) -> Result<()> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ManageMeals)
        .await?;

    if !AVAILABILITIES.contains(&input.availability.as_str()) {
        return Err(Error::Validation {
            message: format!("invalid availability: {}", input.availability),
        });
    }

    state.status_board().set_meal_availability(
        event_id,
        meal_id,
        &input.availability,
        input.eta_minutes,
    )
}