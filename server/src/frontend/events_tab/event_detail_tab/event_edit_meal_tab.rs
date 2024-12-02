use crate::{frontend::events_tab::event_detail_tab, frontend::html_error, MyAppState};
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use bigdecimal::BigDecimal;
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

pub(crate) fn event_edit_meal_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:event_id/:meal_id", post(update_meal))
        .route("/:event_id/:meal_id", get(meal_form))
        .route("/:event_id/:meal_id", delete(delete_meal))
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

pub async fn delete_meal(
    state: State<MyAppState>,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    match state.remove_meal(meal_id).await {
        Ok(_) => Ok(event_detail_tab::event_form(state, Path(event_id))
            .await
            .unwrap_or_else(|e| e)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_meal(
    state: State<MyAppState>,
    Path((event_id, meal_id)): Path<(i32, i32)>,
    Form(meal): Form<MealForm>,
) -> impl IntoResponse {
    let append_start = format!("{}:00-00:00", meal.start_time);
    let start_time = OffsetDateTime::parse(
        &append_start,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Parse end time
    let append_end = format!("{}:00-00:00", meal.end_time);
    let end_time =
        OffsetDateTime::parse(&append_end, &time::format_description::well_known::Rfc3339)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    let result = if meal_id != -1 {
        state
            .update_single_meal(
                meal_id,
                meal.recipe_id,
                meal.place_id,
                start_time,
                end_time,
                meal.energy,
                meal.servings,
                meal.comment,
            )
            .await
    } else {
        state
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
    };
    match result {
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
    let mut meal = foodlib::Meal {
        name: "Select Recipe".to_string(),
        place: "Select Place".to_string(),
        ..Default::default()
    };
    if meal_id != -1 {
        meal = state
            .get_event_meal(dbg!(meal_id))
            .await
            .map_err(|e| html_error(&format!("Failed to fetch meal {e}"), "/events"))?;
    }
    let mut recipes = state
        .get_recipes()
        .await
        .map_err(|e| html_error(&format!("Failed to fetch recipes {e}"), "/events"))?;
    recipes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut places = state
        .get_places()
        .await
        .map_err(|e| html_error(&format!("Failed to fetch places {e}"), "/events"))?;
    places.sort_by(|a, b| a.name.cmp(&b.name));

    let time_format = format_description!("[day]-[month]-[year][hour]:[minute]");

    Ok(html! {
        div class="flex justify-center w-full mb-4" {
            p class="text-3xl" { "Edit Meal" }
        }
        table class="table-auto" {
            thead {
                tr {
                    th { "Field" }
                    th { "Value" }
                }
            }
            tbody {
                tr {
                    td { "Recipe" }
                    td { select name="recipe_id" class="text" required="required" {
                        @for recipe in recipes {
                            (html! {
                                option value=(recipe.recipe_id) selected { (recipe.name) }
                            })
                        }
                        (html! {option value=(meal.recipe_id) selected { (meal.name) }})
                    } }
                }
                tr {
                    td { "Place" }
                    td { select name="place_id" class="text" required="required" {
                        @for place in places {
                            (html! {
                                option value=(place.place_id) selected { (place.name) }
                            })
                        }
                        (html! {option value=(meal.place_id) selected { (meal.place) }})
                    } }
                }
                tr {
                    td { "Start Time" }
                    td { input class="text" type="datetime-local" name="start_time" required="required" value=(meal.start_time.format(&time_format).unwrap()); }
                }
                tr {
                    td { "End Time" }
                    td { input class="text" type="datetime-local" name="end_time" required="required" value=(meal.end_time.format(&time_format).unwrap()); }
                }
                tr {
                    td { "Weight" }
                    td { (meal.weight.to_string()) "g" }
                }
                tr {
                    td {
                    div class="group" {
                        p { "Energy" }
                        span class="absolute z-50 hidden px-6 py-2 -mt-16 text-center text-white bg-blue-900 border border-grey-600 rounded tooltip-text group-hover:block" {"Size of 1 Serving"}
                    }}
                    td { input class="text" type="text" name="energy" required="required" value=(meal.energy.to_string()); }
                }
                tr {
                    td { "Price" }
                    td { (meal.price.to_string()) "€" }
                }
                tr {
                    td { "Servings" }
                    td { input class="text" type="text" name="servings" required="required" value=(meal.servings.to_string()); }
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
