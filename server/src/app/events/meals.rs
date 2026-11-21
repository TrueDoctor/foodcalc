//! Meal-planning use-cases for events.

use serde::Deserialize;
use serde_json::json;

use crate::app::audit;
use crate::app::lib::{get_default_place_for_event, parse_decimal, parse_time};
use crate::app::{AppCtx, Error, MyAppState, Result};
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::meal::PlannerMeal;
use foodlib_new::entities::types::ids::{EventId, MealId, PlaceId, RecipeId};
use foodlib_new::policy::EventPerm;

// ------------------------------------------------------------------ input

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

// ----------------------------------------------------------------- usecases

/// list all planned meals for an event.
/// Requires `ViewEvent`.
pub async fn list(state: &MyAppState, ctx: &AuthCtx, event_id: EventId) -> Result<Vec<PlannerMeal>> {
    ctx.assert_event_action(event_id, EventPerm::ViewEvent).await?;
    state.meals().get_event_meals_planner(event_id).await
}

/// plan a recipe as a meal. Energy scaling:
/// the total weight is derived from `energy_per_serving × servings` against
/// the recipe's energy. Returns the new meal id.
pub async fn add_meal_to_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    input: MealAddInput,
) -> Result<i32> {
    // actor.ctx.assert_global(foodlib_new::policy::GlobalPermission::AllowedToCreateNewDomainObjects).await?;
    actor.ctx.assert_event_action(event_id, EventPerm::EditEvent).await?;

    let start_time = parse_time(&input.start_time)?;
    let end_time = match input.end_time.as_deref() {
        Some(value) => parse_time(value)?,
        None => start_time,
    };
    let energy = parse_decimal(&input.energy_per_serving, "energy_per_serving")?;
    let place_id = match input.place_id {
        Some(place_id) => PlaceId(place_id),
        None => get_default_place_for_event(state).await?,
    };

    let meal_id = state
        .meals()
        .add_meal(
            event_id,
            RecipeId(input.recipe_id),
            place_id,
            start_time,
            end_time,
            energy,
            input.servings,
            input.comment,
        )
        .await?;
    audit::record(
        state,
        actor,
        "event.meal.added",
        Some("meal"),
        Some(meal_id.into()),
        None,
        Some(event_id),
        Some(json!({ "recipe_id": input.recipe_id, "servings": input.servings })),
    )
    .await;
    Ok(meal_id.get())
}

/// Update the meal
/// Requires `EditEvent` and checks event_id matches the meal's event_id.
pub async fn update_meal_of_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
    input: MealUpdateInput,
) -> Result<()> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::EditEvent)
        .await?;
    let current = state.meals().get_meal(meal_id).await?;
    // Guard against moving a meal to another event via its path.
    if current.event_id != event_id {
        return Err(Error::Validation {
            message: "meal does not belong to this event".into(),
        });
    }

    let start_time = match input.start_time.as_deref() {
        Some(value) => parse_time(value)?,
        None => current.start_time,
    };
    let end_time = match input.end_time.as_deref() {
        Some(value) => parse_time(value)?,
        None => current.end_time,
    };
    let energy = match input.energy_per_serving.as_deref() {
        Some(value) => parse_decimal(value, "energy_per_serving")?,
        None => current.energy,
    };
    let comment = match input.comment {
        Some(comment) if comment.is_empty() => None,
        Some(comment) => Some(comment),
        None => current.comment,
    };
    let place_id = PlaceId(input.place_id.unwrap_or(current.place_id.get()));
    let recipe_id = RecipeId(input.recipe_id.unwrap_or(current.recipe_id.get()));
    let servings = input.servings.unwrap_or(current.servings);

    state
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
        .await?;
    audit::record(
        state,
        actor,
        "event.meal.updated",
        Some("meal"),
        Some(meal_id.into()),
        None,
        Some(event_id),
        None,
    )
    .await;
    Ok(())
}

/// unplan the meal.
pub async fn remove_meal_from_event(state: &MyAppState, actor: &AppCtx, event_id: EventId, meal_id: MealId) -> Result<()> {
    // Permission check
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::EditEvent)
        .await?;
    let meal = state.meals().get_meal(meal_id).await?; // ensure meal exists
    if meal.event_id != event_id {
        return Err(Error::Validation {
            message: "meal does not belong to this event".into(),
        });
    }

    // Act
    state.meals().remove_meal(meal_id).await?;
    // Logging
    audit::record(
        state,
        actor,
        "event.meal.removed",
        Some("meal"),
        Some(meal_id.into()),
        None,
        Some(event_id),
        None,
    )
    .await;
    Ok(())
}

/// copy the meal within
/// the event. Returns the new meal id.
pub async fn duplicate_meal_within_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<i32> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::EditEvent)
        .await?;
    // check existence and event_id match
    let meal = state.meals().get_meal(meal_id).await?;
    if meal.event_id != event_id {
        return Err(Error::Validation {
            message: "meal does not belong to this event".into(),
        });
    }

    // act
    let new_id = state.meals().duplicate_meal(meal_id).await?;
    audit::record(
        state,
        actor,
        "event.meal.duplicated",
        Some("meal"),
        Some(new_id.into()),
        None,
        Some(event_id),
        Some(json!({ "duplicated_from": meal_id.get() })),
    )
    .await;
    Ok(new_id.get())
}

