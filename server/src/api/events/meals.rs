//! Meal endpoints for events — thin handlers over `crate::app::meals`.
//!
//! `ctx: AppCtx` works as an extractor because `api::events` implements
//! `FromRequestParts` for `AppCtx` (AuthCtx + `Channel::RestApi`).

use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use serde_json::json;

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::events::meals;
// Re-exported so the handlers share the input DTOs with the app layer.
pub use crate::app::events::meals::{MealAddInput, MealUpdateInput};
use crate::app::AppCtx;

/// `GET /api/events/{id}/meals` — planned meals with scaled weight, energy
/// density (kJ/g) and price, ordered by start time.
pub async fn list_meals(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(meals::list(&state, &ctx.ctx, id).await)
}

/// `POST /api/events/{id}/meals` — plan a recipe as a meal. Energy scaling:
/// the total weight is derived from `energy_per_serving × servings` against
/// the recipe's energy. Returns `{ "meal_id": … }`.
pub async fn add_meal(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<MealAddInput>,
) -> Response {
    respond_created(
        meals::add_meal_to_event(&state, &ctx, id, payload)
            .await
            .map(|meal_id| json!({ "meal_id": meal_id })),
    )
}

/// `PUT /api/events/{id}/meals/{meal_id}` — update a planned meal. Provided
/// fields win; omitted fields keep the current values.
pub async fn update_meal(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
    Json(payload): Json<MealUpdateInput>,
) -> Response {
    respond_no_content(meals::update_meal_of_event(&state, &ctx, event_id, meal_id, payload).await)
}

/// `DELETE /api/events/{id}/meals/{meal_id}` — unplan the meal.
pub async fn remove_meal(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> Response {
    respond_no_content(meals::remove_meal_from_event(&state, &ctx, event_id, meal_id).await)
}

/// `POST /api/events/{id}/meals/{meal_id}/duplicate` — copy the meal within
/// the event. Returns `{ "meal_id": <new id> }`.
pub async fn duplicate_meal(
    State(state): State<crate::MyAppState>,
    ctx: AppCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> Response {
    respond_created(
        meals::duplicate_meal_within_event(&state, &ctx, event_id, meal_id)
            .await
            .map(|new_id| json!({ "meal_id": new_id })),
    )
}
