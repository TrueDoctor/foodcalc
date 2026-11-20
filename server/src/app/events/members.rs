//! Direct per-event collaborators (`event_user_roles`) use-cases.
//!
//! Group members inherit their group permissions; the members listed here
//! hold roles granted *only* for this event (e.g. a guest cook without any
//! group membership). List requires `events:view`; mutations require
//! `events:transfer_owner` (group baseline or direct override) or
//! `group:manage_members` on the owning group.

use serde::{Deserialize, Serialize};
use serde_json::json;

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::Event;
use foodlib_new::entities::user::User;
use foodlib_new::ops::audit::action;
use foodlib_new::policy::{EventPerm, GroupPerm, Permission};

use crate::app::audit;
use crate::app::{AppCtx, Error, MyAppState, Result};

/// Direct collaborator of an event with their event-level role slugs.
#[derive(Debug, Serialize)]
pub struct EventMember {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddEventMemberInput {
    /// Username, email or numeric id of the user to share the event with.
    pub user: String,
    /// Role slugs granted for this event only.
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventMemberRolesInput {
    pub roles: Vec<String>,
}

/// Newly granted collaborator (returned by `add` for instant UI feedback).
#[derive(Debug, Serialize)]
pub struct AddedEventMember {
    pub user_id: i64,
    pub username: String,
    pub roles: Vec<String>,
}

/// Resolves the event and verifies the caller may manage its collaborators:
/// `group:manage_members` on the owning group OR `events:transfer_owner`
/// resolved for this event (group baseline or direct override).
async fn ensure_can_manage(state: &MyAppState, ctx: &AuthCtx, event_id: i32) -> Result<Event> {
    let event = state.events().get(event_id).await?;
    let via_group = ctx
        .has_group_permission(event.group_id, Permission::Group(GroupPerm::ManageMembers));
    let via_event = ctx.has_event_permission(
        event_id,
        event.group_id,
        Permission::Event(EventPerm::TransferOwner),
    );
    if !(via_group || via_event) {
        return Err(Error::Forbidden(
            "managing collaborators requires events:transfer_owner or group:manage_members".into(),
        ));
    }
    Ok(event)
}

/// Trims, drops empties, dedups and sorts role slugs.
fn normalize_roles(roles: Vec<String>) -> Vec<String> {
    let mut roles: Vec<String> = roles
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    roles.sort();
    roles.dedup();
    roles
}

/// `GET /api/events/{id}/members` — direct collaborators. Guard `events:view`.
pub async fn list(state: &MyAppState, ctx: &AuthCtx, event_id: i32) -> Result<Vec<EventMember>> {
    ctx.assert_event_action(event_id, EventPerm::View).await?;
    let members = state.events().list_event_members(event_id).await?;
    Ok(members
        .into_iter()
        .map(|(user, roles)| EventMember {
            user_id: user.id,
            username: user.username,
            email: user.email,
            roles,
        })
        .collect())
}

/// `POST /api/events/{id}/members` — grant a user event-level roles.
pub async fn add(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    input: AddEventMemberInput,
) -> Result<AddedEventMember> {
    let ctx = &actor.ctx;
    ensure_can_manage(state, ctx, event_id).await?;

    let user: User = state
        .users()
        .get_user_by_string_reference(input.user.trim().to_string())
        .await
        .ok_or_else(|| Error::Validation {
            message: format!("unknown user: {}", input.user),
        })?;

    let roles = normalize_roles(input.roles);
    if roles.is_empty() {
        return Err(Error::Validation {
            message: "at least one role is required".into(),
        });
    }
    if !state.users().validate_role_slugs(&roles) {
        return Err(Error::Validation {
            message: "one or more unknown role slugs provided".into(),
        });
    }

    state.events().add_event_member(user.id, event_id, &roles).await?;
    audit::record(
        state,
        actor,
        action::EVENT_MEMBER_ADDED,
        None,
        None,
        None,
        Some(event_id),
        Some(json!({ "target_user_id": user.id, "roles": roles })),
    )
    .await;
    Ok(AddedEventMember {
        user_id: user.id,
        username: user.username,
        roles,
    })
}

/// `PUT /api/events/{id}/members/{user_id}` — replaces the collaborator's
/// event-level role set.
pub async fn set_roles(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    user_id: i64,
    input: EventMemberRolesInput,
) -> Result<()> {
    let ctx = &actor.ctx;
    ensure_can_manage(state, ctx, event_id).await?;

    let roles = normalize_roles(input.roles);
    if roles.is_empty() {
        return Err(Error::Validation {
            message: "use DELETE to revoke event-level access".into(),
        });
    }
    if !state.users().validate_role_slugs(&roles) {
        return Err(Error::Validation {
            message: "one or more unknown role slugs provided".into(),
        });
    }

    state.events().set_event_member_roles(user_id, event_id, &roles).await?;
    audit::record(
        state,
        actor,
        action::EVENT_MEMBER_ROLES_CHANGED,
        None,
        None,
        None,
        Some(event_id),
        Some(json!({ "target_user_id": user_id, "roles": roles })),
    )
    .await;
    Ok(())
}

/// `DELETE /api/events/{id}/members/{user_id}` — revokes all event-level
/// overrides for that user (group access is unaffected).
pub async fn remove(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    user_id: i64,
) -> Result<()> {
    ensure_can_manage(state, &actor.ctx, event_id).await?;
    state.events().remove_event_member(user_id, event_id).await?;
    audit::record(
        state,
        actor,
        action::EVENT_MEMBER_REMOVED,
        None,
        None,
        None,
        Some(event_id),
        Some(json!({ "target_user_id": user_id })),
    )
    .await;
    Ok(())
}
