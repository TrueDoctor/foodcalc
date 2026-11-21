//! Event collaborator endpoints — thin handlers over
//! `crate::app::events::members`. Permission checks, role validation and
//! audit logging live in the app layer.

use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, put},
    Json, Router,
};

use foodlib_new::entities::types::ids::{EventId, UserId};

use crate::api::{respond, respond_created, respond_no_content};
use crate::app::events::members;
use crate::app::AppCtx;
use crate::MyAppState;

// Input DTOs shared with the app layer.
pub use crate::app::events::members::{AddEventMemberInput, EventMemberRolesInput};

pub fn members_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_members).post(add_member))
        .route("/{user_id}", put(set_member_roles).delete(remove_member))
}

/// `GET /api/events/{id}/members` — direct collaborators. Guard `events:view`.
async fn list_members(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
) -> Response {
    respond(members::list(&state, &ctx.ctx, EventId(id)).await)
}

/// `POST /api/events/{id}/members` — grant a user event-level roles.
async fn add_member(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(id): Path<i32>,
    Json(payload): Json<AddEventMemberInput>,
) -> Response {
    respond_created(members::add(&state, &ctx, EventId(id), payload).await)
}

/// `PUT /api/events/{id}/members/{user_id}` — replaces the collaborator's
/// event-level role set.
async fn set_member_roles(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path((id, user_id)): Path<(i32, i64)>,
    Json(payload): Json<EventMemberRolesInput>,
) -> Response {
    respond_no_content(members::set_roles(&state, &ctx, EventId(id), UserId(user_id), payload).await)
}

/// `DELETE /api/events/{id}/members/{user_id}` — revokes all event-level
/// overrides for that user (group access is unaffected).
async fn remove_member(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path((id, user_id)): Path<(i32, i64)>,
) -> Response {
    respond_no_content(members::remove(&state, &ctx, EventId(id), UserId(user_id)).await)
}
