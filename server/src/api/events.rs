use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::json;

use foodlib_new::{auth_context::AuthCtx, entities::event::Event};

use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

pub mod exports;
pub mod meals;
pub mod shopping;

pub fn events_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_events).post(create_event))
        .route(
            "/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route("/{id}/clone", post(clone_event))
        .route("/{id}/meals", get(meals::list_meals).post(meals::add_meal))
        .route(
            "/{id}/meals/{meal_id}",
            put(meals::update_meal).delete(meals::remove_meal),
        )
        .route(
            "/{id}/meals/{meal_id}/duplicate",
            post(meals::duplicate_meal),
        )
        .route(
            "/{id}/tours",
            get(shopping::list_tours).post(shopping::add_tour),
        )
        .route(
            "/{id}/tours/{tour_id}",
            put(shopping::update_tour).delete(shopping::delete_tour),
        )
        .route("/{id}/shopping-list", get(shopping::shopping_list))
        .route(
            "/{id}/source-overrides",
            get(shopping::list_overrides).post(shopping::add_override),
        )
        .route(
            "/{id}/source-overrides/{source_id}",
            delete(shopping::delete_override),
        )
        .route("/{id}/exports/recipes", get(exports::export_recipes))
        .route("/{id}/exports/meal/{meal_id}", get(exports::export_meal))
        .route("/{id}/exports/allergens", get(exports::export_allergens))
        .route("/{id}/exports/labels", get(exports::export_labels))
        .route("/{id}/exports/tour/{tour_id}", get(exports::export_tour))
}

/// `GET /api/events` — all events of the user's groups with planner totals
/// (meal count, ingredient cost, time span), newest activity first.
async fn list_events(State(state): State<MyAppState>, ctx: AuthCtx) -> Response {
    respond(state.events().list_summaries_for_user(ctx.user.id).await)
}

#[derive(Debug, Deserialize)]
pub struct EventCreateInput {
    pub name: String,
    pub comment: Option<String>,
    /// Budget as a decimal string (BigDecimal over the wire).
    pub budget: Option<String>,
    pub group_id: i32,
}

/// `POST /api/events` — create an event owned by one of the user's groups.
async fn create_event(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Json(payload): Json<EventCreateInput>,
) -> Response {
    if let Err(err) = ctx.assert_group(payload.group_id) {
        return map_api_error(err).into_response();
    }
    let budget = match parse_budget(payload.budget) {
        Ok(budget) => budget,
        Err(response) => return response,
    };

    let event = Event {
        id: 0,
        name: payload.name,
        comment: payload.comment,
        budget,
        group_id: payload.group_id,
    };
    respond_created(state.events().create(event).await)
}

#[derive(Debug, Deserialize)]
pub struct EventUpdateInput {
    pub name: Option<String>,
    /// `null` keeps the current comment; an empty string clears it.
    pub comment: Option<String>,
    pub budget: Option<String>,
}

/// `PUT /api/events/{id}` — update the event's frame data.
async fn update_event(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<EventUpdateInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let current = match state.events().get(id).await {
        Ok(event) => event,
        Err(err) => return map_api_error(err).into_response(),
    };
    let budget = match parse_budget(payload.budget) {
        Ok(budget) => budget.or(current.budget),
        Err(response) => return response,
    };
    let comment = match payload.comment {
        Some(comment) if comment.is_empty() => None,
        Some(comment) => Some(comment),
        None => current.comment,
    };

    let updated = Event {
        id: current.id,
        name: payload.name.unwrap_or(current.name),
        comment,
        budget,
        group_id: current.group_id,
    };
    respond(state.events().update(updated).await)
}

/// `DELETE /api/events/{id}` — delete the event (cascades to meals/tours).
async fn delete_event(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    match state.events().delete(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `POST /api/events/{id}/clone` — duplicate the event incl. meals and source
/// overrides. Returns `{ "id": <new event id> }`.
async fn clone_event(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let event = match state.events().get(id).await {
        Ok(event) => event,
        Err(err) => return map_api_error(err).into_response(),
    };
    match state.events().duplicate(id, event.group_id).await {
        Ok(new_id) => respond_created(Ok(json!({ "id": new_id }))),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/events/{id}` — event frame + planned meals (with scaled weight,
/// energy density and price) + the event's total ingredient cost.
async fn get_event(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let event = match state.events().get(id).await {
        Ok(event) => event,
        Err(err) => return map_api_error(err).into_response(),
    };
    let meals = match state.meals().get_event_meals_planner(id).await {
        Ok(meals) => meals,
        Err(err) => return map_api_error(err).into_response(),
    };
    let total_cost = match state.events().get_total_cost(id).await {
        Ok(cost) => cost,
        Err(err) => return map_api_error(err).into_response(),
    };

    (
        StatusCode::OK,
        Json(json!({
            "event": event,
            "meals": meals,
            "total_cost": total_cost,
        })),
    )
        .into_response()
}

/// Parses a budget decimal string; an empty string clears the budget.
fn parse_budget(budget: Option<String>) -> Result<Option<BigDecimal>, Response> {
    match budget.as_deref() {
        None | Some("") => Ok(None),
        Some(value) => match BigDecimal::from_str(value) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Err(bad_request(format!("invalid budget: {value}"))),
        },
    }
}
