//! Direct per-event collaborators (`event_user_roles`).
//!
//! Group members inherit their group permissions; the members listed here
//! hold roles granted *only* for this event (e.g. a guest cook without any
//! group membership). List requires `events:view`; mutations require
//! `events:transfer_owner` (group baseline or direct override).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::{bad_request, map_api_error, respond_created};
use crate::MyAppState;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::Event;
use foodlib_new::error::Error;
use foodlib_new::ops::audit::{action, AuditRecord};
use foodlib_new::policy::{EventPerm, GroupPerm, Permission};

pub fn members_router() -> Router<MyAppState> {
    Router::new()
        .route("/", get(list_members).post(add_member))
        .route("/{user_id}", put(set_member_roles).delete(remove_member))
}

/// Direct collaborator of an event with their event-level role slugs.
#[derive(Debug, Serialize)]
pub struct ApiEventMember {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

/// Resolves the event and verifies the caller may manage its collaborators:
/// `group:manage_members` on the owning group OR `events:transfer_owner`
/// resolved for this event (group baseline or direct override).
async fn ensure_can_manage(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: i32,
) -> std::result::Result<Event, Response> {
    let event = match state.events().get(event_id).await {
        Ok(event) => event,
        Err(err) => return Err(map_api_error(err).into_response()),
    };
    let via_group = ctx
        .has_group_permission(event.group_id, Permission::Group(GroupPerm::ManageMembers));
    let via_event = ctx.has_event_permission(
        event_id,
        event.group_id,
        Permission::Event(EventPerm::TransferOwner),
    );
    if !(via_group || via_event) {
        return Err(map_api_error(Error::Forbidden(
            "managing collaborators requires events:transfer_owner or group:manage_members".into(),
        ))
        .into_response());
    }
    Ok(event)
}

/// `GET /api/events/{id}/members` — direct collaborators. Guard `events:view`.
pub async fn list_members(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_event_action(id, EventPerm::View).await {
        return map_api_error(err).into_response();
    }
    match state.events().list_event_members(id).await {
        Ok(members) => {
            let items: Vec<ApiEventMember> = members
                .into_iter()
                .map(|(user, roles)| ApiEventMember {
                    user_id: user.id,
                    username: user.username,
                    email: user.email,
                    roles,
                })
                .collect();
            (StatusCode::OK, Json(items)).into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddEventMemberInput {
    /// Username, email or numeric id of the user to share the event with.
    pub user: String,
    /// Role slugs granted for this event only.
    pub roles: Vec<String>,
}

/// `POST /api/events/{id}/members` — grant a user event-level roles.
pub async fn add_member(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<AddEventMemberInput>,
) -> Response {
    if let Err(response) = ensure_can_manage(&state, &ctx, id).await {
        return response;
    }
    let user = match state
        .users()
        .get_user_by_string_reference(payload.user.trim().to_string())
        .await
    {
        Some(user) => user,
        None => return bad_request(format!("unknown user: {}", payload.user)),
    };

    let roles: Vec<String> = payload
        .roles
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    if roles.is_empty() {
        return bad_request("at least one role is required".into());
    }
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    match state.events().add_event_member(user.id, id, &roles).await {
        Ok(()) => {
            audit_member(
                &state,
                &ctx,
                action::EVENT_MEMBER_ADDED,
                id,
                user.id,
                Some(roles.clone()),
            )
            .await;
            respond_created(Ok(serde_json::json!({
                "user_id": user.id,
                "username": user.username,
                "roles": roles,
            })))
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct EventMemberRolesInput {
    pub roles: Vec<String>,
}

/// `PUT /api/events/{id}/members/{user_id}` — replaces the collaborator's
/// event-level role set.
pub async fn set_member_roles(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, user_id)): Path<(i32, i64)>,
    Json(payload): Json<EventMemberRolesInput>,
) -> Response {
    if let Err(response) = ensure_can_manage(&state, &ctx, id).await {
        return response;
    }
    let mut roles: Vec<String> = payload
        .roles
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    roles.sort();
    roles.dedup();
    if roles.is_empty() {
        return bad_request("use DELETE to revoke event-level access".into());
    }
    if !state.users().validate_role_slugs(&roles) {
        return bad_request("one or more unknown role slugs provided".into());
    }

    match state
        .events()
        .set_event_member_roles(user_id, id, &roles)
        .await
    {
        Ok(()) => {
            audit_member(&state, &ctx, action::EVENT_MEMBER_ROLES_CHANGED, id, user_id, Some(roles))
                .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `DELETE /api/events/{id}/members/{user_id}` — revokes all event-level
/// overrides for that user (group access is unaffected).
pub async fn remove_member(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((id, user_id)): Path<(i32, i64)>,
) -> Response {
    if let Err(response) = ensure_can_manage(&state, &ctx, id).await {
        return response;
    }
    match state.events().remove_event_member(user_id, id).await {
        Ok(()) => {
            audit_member(&state, &ctx, action::EVENT_MEMBER_REMOVED, id, user_id, None).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => map_api_error(err).into_response(),
    }
}

/// Best-effort audit for collaborator changes.
async fn audit_member(
    state: &MyAppState,
    ctx: &AuthCtx,
    action: &'static str,
    event_id: i32,
    target_user_id: i64,
    roles: Option<Vec<String>>,
) {
    if let Err(err) = state
        .audit()
        .record(AuditRecord {
            actor_user_id: Some(ctx.user.id),
            actor_username: &ctx.user.username,
            action,
            event_id: Some(event_id),
            target_user_id: Some(target_user_id),
            details: roles.map(|roles| serde_json::json!({ "roles": roles })),
            ..Default::default()
        })
        .await
    {
        log::error!("event member change not audited: {err}");
    }
}
