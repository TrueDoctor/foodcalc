//! Status board use-cases: kitchen/serving board snapshot, reorders and
//! meal availability. Validation of the free-form status/urgency/availability
//! strings lives here so every transport shares the same rules.

use foodlib_new::ops::status_board::{CreateReorderInput, StatusBoardSnapshot};

use super::{Error, MyAppState, Result};

const URGENCIES: [&str; 3] = ["normal", "urgent", "critical"];
const REORDER_STATUSES: [&str; 4] = ["requested", "preparing", "en_route", "delivered"];
const AVAILABILITIES: [&str; 4] = ["available", "low", "replenishing", "sold_out"];

/// `GET /api/events/{id}/status-board` — combined kitchen + serving snapshot.
pub async fn get_board(state: &MyAppState, event_id: i32) -> Result<StatusBoardSnapshot> {
    Ok(state.status_board().get_board(event_id).await?)
}

/// `POST /api/events/{id}/reorders` — place a kitchen reorder. Returns the
/// new reorder id.
pub async fn create_reorder(
    state: &MyAppState,
    event_id: i32,
    input: CreateReorderInput,
) -> Result<i32> {
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
    let reorder_id = state.status_board().create_reorder(event_id, input).await?;
    Ok(reorder_id)
}

/// `PUT /api/events/{id}/reorders/{reorder_id}/status` — advance the reorder
/// through the kitchen workflow.
pub async fn update_reorder_status(
    state: &MyAppState,
    event_id: i32,
    reorder_id: i32,
    status: &str,
) -> Result<()> {
    if !REORDER_STATUSES.contains(&status) {
        return Err(Error::Validation {
            message: format!("invalid status: {status}"),
        });
    }
    Ok(state
        .status_board()
        .update_reorder_status(event_id, reorder_id, status)?)
}

/// `PUT /api/events/{id}/meals/{meal_id}/availability` — set the serving
/// availability (and optional ETA) for a meal on the serving board.
pub async fn set_meal_availability(
    state: &MyAppState,
    event_id: i32,
    meal_id: i32,
    availability: &str,
    eta_minutes: Option<i32>,
) -> Result<()> {
    if !AVAILABILITIES.contains(&availability) {
        return Err(Error::Validation {
            message: format!("invalid availability: {availability}"),
        });
    }
    Ok(state
        .status_board()
        .set_meal_availability(event_id, meal_id, availability, eta_minutes)?)
}
