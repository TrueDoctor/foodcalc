//! Event endpoints. Handlers are thin: they extract inputs, delegate to the
//! application layer (`crate::app::events`) and map errors to HTTP. All
//! permission checks, orchestration and audit logging live in the app layer.

use axum::http::request::Parts;
use axum::{
    extract::{FromRequestParts, Path, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::api::{map_api_error, respond, respond_created, respond_no_content};
use crate::app::{events, AppCtx, Channel};
use crate::MyAppState;

pub mod exports;
pub mod logistics;
pub mod meals;
pub mod members;
pub mod shopping;

// Re-exported so handlers share the input DTOs with the app layer.
pub use crate::app::events::{EventCreateInput, EventUpdateInput};


#[derive(Debug, Deserialize)]
pub struct OwnerTransferInput {
    pub group_id: i32,
}

pub fn events_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_events).post(create_event))
        .route(
            "/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route("/{id}/clone", post(clone_event))
        .route("/{id}/owner", put(transfer_owner))
        .nest("/{id}/members", members::members_router())
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
        .route(
            "/{id}/tours/{tour_id}/items",
            post(shopping::add_tour_item),
        )
        .route(
            "/{id}/tours/{tour_id}/items/{ingredient_id}",
            delete(shopping::remove_tour_item),
        )
        .route(
            "/{id}/tours/{tour_id}/completed",
            put(shopping::set_tour_completed),
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
        .route("/{id}/exports/overview", get(exports::export_overview))
        .route("/{id}/exports/recipes", get(exports::export_recipes))
        .route("/{id}/exports/meal/{meal_id}", get(exports::export_meal))
        .route("/{id}/exports/allergens", get(exports::export_allergens))
        .route("/{id}/exports/labels", get(exports::export_labels))
        .route("/{id}/exports/tour/{tour_id}", get(exports::export_tour))
        .merge(logistics::logistics_router())
        .merge(crate::api::status_board::status_board_router())
}

/// App-layer context for REST handlers: the resolved `AuthCtx` plus the
/// `Channel::RestApi` marker. Implemented as an extractor so thin handlers
/// can take `ctx: AppCtx` directly.
impl FromRequestParts<MyAppState> for AppCtx {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &MyAppState,
    ) -> Result<Self, Self::Rejection> {
        let ctx = foodlib_new::auth_context::AuthCtx::from_request_parts(parts, state)
            .await
            .map_err(|err| map_api_error(err).into_response())?;
        Ok(AppCtx::new(ctx, Channel::RestApi))
    }
}

/// `GET /api/events` — all events of the user's groups with planner totals
/// (meal count, ingredient cost, time span), newest activity first.
async fn list_events(State(state): State<MyAppState>, ctx: AppCtx) -> Response {
    respond(events::list_summaries(&state, &ctx.ctx).await)
}

/// `GET /api/events/{id}` — event detail: frame data, planned meals, total
/// cost and the caller's resolved permissions for the event.
async fn get_event(State(state): State<MyAppState>, ctx: AppCtx, Path(id): Path<i32>) -> Response {
    respond(events::get_event_details(&state, &ctx.ctx, id).await)
}

/// `POST /api/events` — create an event owned by one of the user's groups.
async fn create_event(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Json(payload): Json<EventCreateInput>,
) -> Response {
    respond_created(events::create_new_event(&state, &ctx, payload).await)
}

/// `PUT /api/events/{id}` — update the event's frame data.
async fn update_event(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<EventUpdateInput>,
) -> Response {
    respond(events::update_event(&state, &ctx, id, payload).await)
}

/// `DELETE /api/events/{id}` — delete the event (cascades to meals/tours).
async fn delete_event(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond_no_content(events::delete_event(&state, &ctx, id).await)
}

/// `POST /api/events/{id}/clone` — duplicate the event incl. meals and source
/// overrides. Returns `{ "id": <new event id> }`.
async fn clone_event(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond_created(
        events::clone_created_event(&state, &ctx, id)
            .await
            .map(|id| json!({ "id": id })),
    )
}

/// `PUT /api/events/{id}/owner` — explicit Besitz-Übertragung (siehe
/// `crate::app::events::transfer_owner`).
async fn transfer_owner(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<OwnerTransferInput>,
) -> Response {
    respond_no_content(events::transfer_event_owner(&state, &ctx, id, payload.group_id).await)
}
