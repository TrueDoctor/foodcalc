use crate::{frontend::events_tab::event_detail_tab, frontend::html_error, MyAppState};
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use bigdecimal::BigDecimal;
use chrono::DateTime;
use maud::{html, Markup};
use serde::Deserialize;

pub(crate) fn event_edit_meal_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:event_id/:meal_id", post(update_meal))
        .route("/:event_id/:meal_id", get(meal_form))
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct MealForm {
    pub recipe_id: i32,
    pub place_id: i32,
    pub start_time: String,
    pub end_time: String,
    pub energy: BigDecimal,
    pub servings: i32,
    pub comment: Option<String>,
}

pub async fn update_meal(
    state: State<MyAppState>,
    Path((event_id, meal_id)): Path<(i32, i32)>,
    Form(meal): Form<MealForm>,
) -> impl IntoResponse {
    let append_start = meal.start_time + ":00-00:00";
    let start_time = DateTime::parse_from_rfc3339(&append_start)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .naive_utc();
    let append_end = meal.end_time + ":00-00:00";
    let end_time = DateTime::parse_from_rfc3339(&append_end)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .naive_utc();
    match state.remove_meal(meal_id).await {
        Ok(_) => (),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
    match state
        .add_meal(
            event_id,
            meal.recipe_id,
            meal.place_id,
            start_time,
            end_time,
            meal.energy,
            meal.servings,
            meal.comment,
        )
        .await
    {
        Ok(_) => Ok(event_detail_tab::event_form(state, Path(event_id))
            .await
            .unwrap_or_else(|e| e)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn meal_form(
    state: State<MyAppState>,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> Result<Markup, Markup> {
    let meal = state
        .get_event_meal(dbg!(meal_id))
        .await
        .map_err(|e| html_error(&format!("Failed to fetch meal {e}"), "/events"))?;
    Ok(html! {
        table class="table-auto" {
            thead {
                tr {
                    th { "Field" }
                    th { "Value" }
                }
            }
            tbody {
                tr {
                    td { "Recipe_id" }
                    td { input class="text" type="number" name="recipe_id" value=(meal.recipe_id.to_string()); }
                }
                tr {
                    td { "Place_id" }
                    td { input class="text" type="number" name="place_id" value=(meal.place_id.to_string()); }
                }
                tr {
                    td { "Start Time" }
                    td { input class="text" type="datetime-local" name="start_time" value=(meal.start_time.format("%Y-%m-%dT%H:%M").to_string()); }
                }
                tr {
                    td { "End Time" }
                    td { input class="text" type="datetime-local" name="end_time" value=(meal.end_time.format("%Y-%m-%dT%H:%M").to_string()); }
                }
                tr {
                    td { "Energy" }
                    td { input class="text" type="number" name="energy" value=(meal.energy.to_string()); }
                }
                tr {
                    td { "Servings" }
                    td { input class="text" type="number" name="servings" value=(meal.servings.to_string()); }
                }
                tr {
                    td { "Comment" }
                    td { input class="text" type="text" name="comment" value=(meal.comment.unwrap_or_default()); }
                }
            }
        }

        div class="flex justify-between w-full mt-4 gap-2" {
            button class="btn btn-abort" hx-target="#content" hx-get=(format!("/events/edit/{}", event_id)) { "Abort" }
            button class="btn btn-primary mx-4" hx-target="#content" hx-post=(format!("events/edit/event_edit_meal/{}/{}", event_id, meal_id)) hx-include="*" { "Save" }
        }
    })
}
