use std::collections::hash_map::Entry;

use axum::{
    extract::{Form, Path, State},
    routing::{delete, get, post},
};
use foodlib_new::{event::FoodPrep, recipe::Recipe};
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

use crate::{
    frontend::{events_tab::event_detail_tab, MResponse},
    FoodLib, MyAppState,
};

pub(crate) fn food_prep_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/add/{event_id}", get(add_food_prep))
        .route("/edit/{event_id}/{prep_id}", get(food_prep_form))
        .route("/save", post(save_food_prep))
        .route("/delete/{event_id}/{prep_id}", delete(delete_food_prep))
        .route(
            "/delete_dialog/{event_id}/{prep_id}",
            get(delete_food_prep_dialog),
        )
}

#[derive(Deserialize)]
struct FoodPrepForm {
    event_id: i32,
    prep_id: Option<i32>,
    recipe_id: i32,
    prep_date: String,
    use_from: Option<String>,
    use_until: String,
}

async fn add_food_prep(state: State<MyAppState>, Path(event_id): Path<i32>) -> Markup {
    food_prep_form(state, Path((event_id, -1))).await
}

async fn delete_food_prep(
    foodlib: FoodLib,
    Path((event_id, prep_id)): Path<(i32, i32)>,
) -> MResponse {
    foodlib.events().delete_food_prep(prep_id).await?;
    event_detail_tab::event_form(foodlib, Path(event_id)).await
}

async fn delete_food_prep_dialog(
    state: State<MyAppState>,
    Path((event_id, prep_id)): Path<(i32, i32)>,
) -> MResponse {
    // Get food prep details if possible
    let prep = state.new_lib().events().get_food_prep(prep_id).await?;
    // Get recipe name if possible
    let recipe = state.new_lib().recipes().get(prep.recipe_id).await?;
    let recipe_name = recipe.name;
    Ok(html! {
        dialog open="true" class="dialog" id="delete" {
            p class="text-2xl" { (format!("Do you really want to delete food prep for: {}", recipe_name)) }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get=(format!("/events/edit/{}", event_id)) hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/edit/food_prep/delete/{}/{}", event_id, prep_id)) { "Confirm Delete" }
            }
        }
    })
}

async fn food_prep_form(
    State(state): State<MyAppState>,
    Path((event_id, prep_id)): Path<(i32, i32)>,
) -> Markup {
    let recipes = state.get_recipes().await.unwrap_or_default();

    // If prep_id is provided, get existing prep data
    let prep = if prep_id > 0 {
        state
            .new_lib()
            .events()
            .get_food_prep(prep_id)
            .await
            .unwrap_or(FoodPrep {
                id: -1,
                event_id,
                recipe_id: 0,
                prep_date: OffsetDateTime::now_utc(),
                use_from: None,
                use_until: OffsetDateTime::now_utc(),
            })
    } else {
        FoodPrep {
            id: -1,
            event_id,
            recipe_id: 0,
            prep_date: OffsetDateTime::now_utc(),
            use_from: None,
            use_until: OffsetDateTime::now_utc(),
        }
    };

    let time_format = format_description!("[year]-[month]-[day]T[hour]:[minute]");

    html! {
        div class="flex-col space-y-4 w-full" {
            h2 class="text-xl" {
                @if prep_id > 0 {
                    "Edit Food Preparation"
                } @else {
                    "Add Food Preparation"
                }
            }

            form class="w-full space-y-4" hx-post="/events/edit/food_prep/save" hx-target="#content" {
                input type="hidden" name="event_id" value=(event_id);
                input type="hidden" name="prep_id" value=(prep_id);

                div class="flex flex-col space-y-2" {
                    label for="recipe_id" { "Recipe" }
                    select name="recipe_id" class="text" required {
                        option value="" { "Select recipe..." }
                        @for recipe in recipes {
                            option value=(recipe.recipe_id)
                                selected[recipe.recipe_id == prep.recipe_id] {
                                (recipe.name)
                            }
                        }
                    }
                }

                div class="flex flex-col space-y-2" {
                    label for="prep_date" { "Preparation Date & Time" }
                    input type="datetime-local"
                        name="prep_date"
                        class="text"
                        value=(prep.prep_date.format(time_format).unwrap_or_default())
                        required;
                }

                div class="flex flex-col space-y-2" {
                    label for="use_from" { "Use From (Optional)" }
                    input type="datetime-local"
                        name="use_from"
                        class="text"
                        value=(prep.use_from.map(|dt| dt.format(time_format).unwrap_or_default()).unwrap_or_default());
                }

                div class="flex flex-col space-y-2" {
                    label for="use_until" { "Use Until" }
                    input type="datetime-local"
                        name="use_until"
                        class="text"
                        value=(prep.use_until.format(time_format).unwrap_or_default())
                        required;
                }

                div class="flex gap-2 mt-4" {
                    button type="submit" class="btn btn-primary" {
                        @if prep_id > 0 {
                            "Update"
                        } @else {
                            "Create"
                        }
                    }
                    button type="button"
                        class="btn btn-cancel"
                        hx-get=(format!("/events/edit/{}", event_id))
                        hx-target="#content" { "Cancel" }
                }
            }
        }
    }
}

async fn save_food_prep(foodlib: FoodLib, Form(form): Form<FoodPrepForm>) -> MResponse {
    // Parse dates
    let date_format = format_description!("[year]-[month]-[day]T[hour]:[minute]");

    let prep_date = time::PrimitiveDateTime::parse(&form.prep_date, date_format)?;
    let prep_date = prep_date.assume_utc();

    let use_from = if let Some(date_str) = form.use_from {
        if !date_str.is_empty() {
            let dt = time::PrimitiveDateTime::parse(&date_str, date_format)?;
            Some(dt.assume_utc())
        } else {
            None
        }
    } else {
        None
    };

    let use_until = time::PrimitiveDateTime::parse(&form.use_until, date_format)?.assume_utc();

    // Create or update prep
    match form.prep_id {
        Some(prep_id) if prep_id > 0 => {
            foodlib
                .events()
                .update_food_prep(FoodPrep {
                    id: prep_id,
                    event_id: form.event_id,
                    recipe_id: form.recipe_id,
                    prep_date,
                    use_from,
                    use_until,
                })
                .await?;
        }
        _ => {
            foodlib
                .events()
                .add_food_prep(FoodPrep {
                    id: -1,
                    event_id: form.event_id,
                    recipe_id: form.recipe_id,
                    prep_date,
                    use_from,
                    use_until,
                })
                .await?;
        }
    }

    event_detail_tab::event_form(foodlib, Path(form.event_id)).await
}

fn format_food_prep(event_id: i32, prep: &FoodPrep, recipe: &Recipe) -> Markup {
    let time_format = format_description!("[day].[month] [hour]:[minute]");
    let prep_date = prep.prep_date.format(time_format).unwrap_or_default();
    let use_from = prep
        .use_from
        .map(|d| d.format(time_format).unwrap_or_default())
        .unwrap_or_else(|| "-".to_string());
    let use_until = prep.use_until.format(time_format).unwrap_or_default();

    html! {
        tr {
            td { (recipe.name) }
            td { (prep_date) }
            td { (use_from) }
            td { (use_until) }
            td {
                button class="btn btn-primary"
                    hx-target="#content"
                    hx-push-url="true"
                    hx-get=(format!("/events/edit/food_prep/edit/{}/{}", event_id, prep.id)) {
                    "Edit"
                }
            }
            td {
                button class="btn btn-cancel"
                    hx-target="#content"
                    hx-get=(format!("/events/edit/food_prep/delete_dialog/{}/{}", event_id, prep.id)) {
                    "Delete"
                }
            }
        }
    }
}

pub async fn render_food_prep(foodlib: FoodLib, event_id: i32) -> MResponse {
    let preps = foodlib.events().get_food_prep_tasks(event_id).await?;

    // Get recipes for all food preps
    let mut recipe_map = std::collections::HashMap::new();
    for prep in &preps {
        if let Entry::Vacant(e) = recipe_map.entry(prep.recipe_id) {
            if let Ok(recipe) = foodlib.recipes().get(prep.recipe_id).await {
                e.insert(recipe);
            }
        }
    }

    Ok(html! {
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Food Preparation" }
        }
        div class="flex flex-row items-center justify-center mb-2" {
            button class="btn btn-primary"
                hx-get=(format!("/events/edit/food_prep/add/{}", event_id))
                hx-swap="innerHtml"
                hx-push-url="true"
                hx-target="#content" { "Add Food Prep" }
        }
        table class="w-full text-inherit table-auto object-center table-fixed" {
            thead {
                tr {
                    th { "Recipe" }
                    th { "Prep Date" }
                    th { "Use From" }
                    th { "Use Until" }
                    th {} th {}
                }
            }
            tbody {
                @for prep in preps {
                    @if let Some(recipe) = recipe_map.get(&prep.recipe_id) {
                        (format_food_prep(event_id, &prep, recipe))
                    }
                }
            }
        }
    })
}
