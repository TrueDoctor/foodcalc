use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use foodlib_new::auth_context::AuthCtx;

use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

#[derive(Debug, Deserialize)]
pub struct MealAddInput {
    pub recipe_id: i32,
    pub place_id: Option<i32>,
    /// RFC 3339 timestamp, e.g. `2026-06-01T18:00:00Z`.
    pub start_time: String,
    pub end_time: Option<String>,
    /// Planned energy per portion in kJ — the scaling input that determines
    /// the total cooked weight via the recipe's energy density.
    pub energy_per_serving: String,
    pub servings: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MealUpdateInput {
    pub recipe_id: Option<i32>,
    pub place_id: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub energy_per_serving: Option<String>,
    pub servings: Option<i32>,
    /// `null` keeps the comment; an empty string clears it.
    pub comment: Option<String>,
}

/// `GET /api/events/{id}/meals` — planned meals with scaled weight, energy
/// density (kJ/g) and price, ordered by start time.
pub async fn list_meals(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.meals().get_event_meals_planner(id).await)
}

/// `POST /api/events/{id}/meals` — plan a recipe as a meal. Energy scaling:
/// the total weight is derived from `energy_per_serving × servings` against
/// the recipe's energy. Returns `{ "meal_id": … }`.
pub async fn add_meal(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<MealAddInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }

    let start_time = match parse_time(&payload.start_time) {
        Ok(time) => time,
        Err(response) => return response,
    };
    let end_time = match payload.end_time.as_deref() {
        Some(value) => match parse_time(value) {
            Ok(time) => time,
            Err(response) => return response,
        },
        None => start_time,
    };
    let energy = match parse_decimal(&payload.energy_per_serving, "energy_per_serving") {
        Ok(value) => value,
        Err(response) => return response,
    };
    let place_id = match payload.place_id {
        Some(place_id) => place_id,
        None => match default_place(&state).await {
            Ok(place_id) => place_id,
            Err(response) => return response,
        },
    };

    match state
        .meals()
        .add_meal(
            id,
            payload.recipe_id,
            place_id,
            start_time,
            end_time,
            energy,
            payload.servings,
            payload.comment,
        )
        .await
    {
        Ok(meal_id) => respond_created(Ok(json!({ "meal_id": meal_id }))),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `PUT /api/events/{id}/meals/{meal_id}` — update a planned meal. Provided
/// fields win; omitted fields keep the current values.
pub async fn update_meal(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
    Json(payload): Json<MealUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_meal(meal_id).await {
        return map_api_error(err).into_response();
    }
    let current = match state.meals().get_meal(meal_id).await {
        Ok(meal) => meal,
        Err(err) => return map_api_error(err).into_response(),
    };
    // Guard against moving a meal to another event via its path.
    if current.event_id != event_id {
        return bad_request("meal does not belong to this event".into());
    }

    let start_time = match payload.start_time.as_deref() {
        Some(value) => match parse_time(value) {
            Ok(time) => time,
            Err(response) => return response,
        },
        None => current.start_time,
    };
    let end_time = match payload.end_time.as_deref() {
        Some(value) => match parse_time(value) {
            Ok(time) => time,
            Err(response) => return response,
        },
        None => current.end_time,
    };
    let energy = match payload.energy_per_serving.as_deref() {
        Some(value) => match parse_decimal(value, "energy_per_serving") {
            Ok(parsed) => parsed,
            Err(response) => return response,
        },
        None => current.energy,
    };
    let comment = match payload.comment {
        Some(comment) if comment.is_empty() => None,
        Some(comment) => Some(comment),
        None => current.comment,
    };
    let place_id = payload.place_id.unwrap_or(current.place_id);
    let recipe_id = payload.recipe_id.unwrap_or(current.recipe_id);
    let servings = payload.servings.unwrap_or(current.servings);

    match state
        .meals()
        .update_meal(
            meal_id,
            recipe_id,
            place_id,
            start_time,
            end_time,
            energy,
            servings,
            comment,
        )
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({ "meal_id": meal_id, "event_id": event_id })),
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/events/{id}/meals/{meal_id}` — unplan the meal.
pub async fn remove_meal(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, meal_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_meal(meal_id).await {
        return map_api_error(err).into_response();
    }
    match state.meals().remove_meal(meal_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `POST /api/events/{id}/meals/{meal_id}/duplicate` — copy the meal within
/// the event. Returns `{ "meal_id": <new id> }`.
pub async fn duplicate_meal(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, meal_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_meal(meal_id).await {
        return map_api_error(err).into_response();
    }
    match state.meals().duplicate_meal(meal_id).await {
        Ok(new_id) => respond_created(Ok(json!({ "meal_id": new_id }))),
        Err(err) => map_api_error(err).into_response(),
    }
}

fn parse_time(value: &str) -> Result<OffsetDateTime, Response> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| bad_request(format!("invalid RFC 3339 timestamp: {value}")))
}

fn parse_decimal(value: &str, field: &'static str) -> Result<BigDecimal, Response> {
    BigDecimal::from_str(value)
        .map_err(|_| bad_request(format!("invalid {field}: {value}")))
}

/// Falls back to the first configured place when the client omits `place_id`.
async fn default_place(state: &MyAppState) -> Result<i32, Response> {
    let places = state
        .events()
        .get_places()
        .await
        .map_err(|err| map_api_error(err).into_response())?;
    places
        .first()
        .map(|place| place.id)
        .ok_or_else(|| bad_request("no places configured".into()))
}
