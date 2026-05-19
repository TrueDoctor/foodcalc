use crate::{
    frontend::{events_tab::event_detail_tab, html_error, MResponse},
    FoodLib, MyAppState,
};
use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use bigdecimal::BigDecimal;
use foodlib_new::auth_context::AuthCtx;
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

pub(crate) fn event_edit_meal_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/{event_id}/{meal_id}", post(update_meal))
        .route("/{event_id}/{meal_id}", get(meal_form))
        .route("/{event_id}/{meal_id}", delete(delete_meal))
        .route("/{meal_id}/price", get(meal_price_cell))
        .route("/{meal_id}/price_per_serving", get(meal_price_per_serving_cell))
}

async fn meal_price_cell(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(meal_id): Path<i32>,
) -> MResponse {
    ctx.assert_can_edit_meal(meal_id).await?;
    let price = foodlib.meals().get_meal_price(meal_id).await?;
    Ok(html! { (price.round(2).to_string()) "€" })
}

async fn meal_price_per_serving_cell(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(meal_id): Path<i32>,
) -> MResponse {
    use bigdecimal::ToPrimitive;
    ctx.assert_can_edit_meal(meal_id).await?;
    let meal = foodlib.meals().get_meal(meal_id).await?;
    let price = foodlib.meals().get_meal_price(meal_id).await?;
    let per_serving = price.to_f64().unwrap_or_default() / meal.servings as f64;
    Ok(html! { (format!("{:.3}€", per_serving)) })
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct MealForm {
    pub recipe_id: i32,
    pub place_id: Option<i32>,
    pub start_time: String,
    pub end_time: String,
    pub energy: BigDecimal,
    pub servings: i32,
    pub comment: Option<String>,
}

pub async fn delete_meal(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    ctx.assert_can_edit_meal(meal_id).await?;
    foodlib.meals().remove_meal(meal_id).await?;
    event_detail_tab::event_form(foodlib, ctx, Path(event_id)).await
}

pub async fn update_meal(
    foodlib: FoodLib,
    ctx: AuthCtx,
    state: State<MyAppState>,
    Path((event_id, meal_id)): Path<(i32, i32)>,
    Form(meal): Form<MealForm>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    if meal_id != -1 {
        ctx.assert_can_edit_meal(meal_id).await?;
    }
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
    let place_id = meal.place_id.filter(|&id| id >= 0).ok_or_else(|| {
        foodlib_new::Error::Validation {
            message: "Please select a place for the meal.".into(),
        }
    })?;

    let result = if meal_id != -1 {
        state
            .meals()
            .update_meal(
                meal_id,
                meal.recipe_id,
                place_id,
                start_time,
                end_time,
                meal.energy,
                meal.servings,
                meal.comment,
            )
            .await
    } else {
        state
            .meals()
            .add_meal(
                event_id,
                meal.recipe_id,
                place_id,
                start_time,
                end_time,
                meal.energy,
                meal.servings,
                meal.comment,
            )
            .await
    };
    match result {
        Ok(_) => event_detail_tab::event_form(foodlib, ctx, Path(event_id)).await,
        Err(e) => Err(e.into()),
    }
}

#[derive(Deserialize, Default)]
pub struct MealPrefill {
    recipe_id: Option<i32>,
    start_time: Option<String>,
    servings: Option<i32>,
}

async fn meal_form(
    state: State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
    Query(prefill): Query<MealPrefill>,
) -> Result<Markup, Markup> {
    ctx.assert_can_edit_event(event_id)
        .await
        .map_err(|e| html_error(&format!("{e}"), "/events"))?;
    if meal_id != -1 {
        ctx.assert_can_edit_meal(meal_id)
            .await
            .map_err(|e| html_error(&format!("{e}"), "/events"))?;
    }
    let mut meal = foodlib_new::entities::meal::Meal {
        name: "Select Recipe".to_string(),
        place: "Select Place".to_string(),
        ..Default::default()
    };
    if meal_id != -1 {
        meal = state
            .meals()
            .get_meal(meal_id)
            .await
            .map_err(|e| html_error(&format!("Failed to fetch meal {e}"), "/events"))?;
    } else {
        if let Some(rid) = prefill.recipe_id {
            if let Ok(r) = state.recipes().get(rid).await {
                meal.recipe_id = r.id;
                meal.name = r.name;
            }
        }
        if let Some(s) = prefill.servings {
            meal.servings = s;
        }
        if let Some(ref st) = prefill.start_time {
            if let Ok(dt) = time::PrimitiveDateTime::parse(
                st,
                format_description!("[year]-[month]-[day]T[hour]:[minute]"),
            ) {
                let parsed = dt.assume_utc();
                meal.start_time = parsed;
                meal.end_time = parsed + time::Duration::hours(1);
            }
        }
    }
    let mut recipes = state
        .recipes()
        .list()
        .await
        .map_err(|e| html_error(&format!("Failed to fetch recipes {e}"), "/events"))?;
    recipes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut places = state
        .events()
        .get_places()
        .await
        .map_err(|e| html_error(&format!("Failed to fetch places {e}"), "/events"))?;
    places.sort_by(|a, b| a.name.cmp(&b.name));

    let time_format = format_description!("[year]-[month]-[day]T[hour]:[minute]");

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
                        @for recipe in &recipes {
                            option value=(recipe.id) selected[recipe.id == meal.recipe_id] { (recipe.name) }
                        }
                    } }
                }
                tr {
                    td { "Place" }
                    td { select name="place_id" class="text" required="required" {
                        @for place in &places {
                            option value=(place.id) selected[place.id == meal.place_id] { (place.name) }
                        }
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
                    td { (meal.weight.round(2).to_string()) "kg" }
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
                    @if meal_id != -1 {
                        td hx-get=(format!("/events/edit/event_edit_meal/{}/price", meal_id)) hx-trigger="load" hx-swap="innerHTML" { "…" }
                    } @else {
                        td { "—" }
                    }
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

            button class="btn btn-primary mx-4" hx-target="#content" hx-post=(format!("/events/edit/event_edit_meal/{}/{}", event_id, meal_id)) hx-include="*" { "Save" }
        }
    })
}
