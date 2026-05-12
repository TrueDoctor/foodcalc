use std::collections::hash_map::Entry;

use axum::{
    extract::{Form, Path, State},
    routing::{delete, get, post},
};
use foodlib_new::{auth_context::AuthCtx, event::FoodPrep, recipe::Recipe};
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

use crate::{
    frontend::{events_tab::event_detail_tab, html_error, MResponse},
    FoodLib, MyAppState,
};

pub(crate) fn food_prep_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/add/{event_id}", get(add_food_prep))
        .route("/create_inline/{event_id}", post(create_inline))
        .route("/edit/{event_id}/{prep_id}", get(food_prep_form))
        .route("/save", post(save_food_prep))
        .route("/delete/{event_id}/{prep_id}", delete(delete_food_prep))
        .route(
            "/delete_dialog/{event_id}/{prep_id}",
            get(delete_food_prep_dialog),
        )
}

#[derive(Deserialize)]
struct InlinePrepForm {
    recipe_id: i32,
    prep_date: String,
    use_from: Option<String>,
    use_until: String,
}

async fn create_inline(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
    Form(form): Form<InlinePrepForm>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let prep_date =
        super::shopping_tours::parse_datetime_local(&form.prep_date)?;
    let use_from = form
        .use_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(super::shopping_tours::parse_datetime_local)
        .transpose()?;
    let use_until =
        super::shopping_tours::parse_datetime_local(&form.use_until)?;
    foodlib
        .events()
        .add_food_prep(FoodPrep {
            id: -1,
            event_id,
            recipe_id: form.recipe_id,
            prep_date,
            use_from,
            use_until,
        })
        .await?;
    event_detail_tab::event_form(foodlib, ctx, Path(event_id)).await
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

async fn add_food_prep(
    state: State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Result<Markup, Markup> {
    food_prep_form(state, ctx, Path((event_id, -1))).await
}

async fn delete_food_prep(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((event_id, prep_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    ctx.assert_can_edit_food_prep(prep_id).await?;
    foodlib.events().delete_food_prep(prep_id).await?;
    event_detail_tab::event_form(foodlib, ctx, Path(event_id)).await
}

async fn delete_food_prep_dialog(
    state: State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, prep_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    ctx.assert_can_edit_food_prep(prep_id).await?;
    // Get food prep details if possible
    let prep = state.events().get_food_prep(prep_id).await?;
    // Get recipe name if possible
    let recipe = state.recipes().get(prep.recipe_id).await?;
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
    ctx: AuthCtx,
    Path((event_id, prep_id)): Path<(i32, i32)>,
) -> Result<Markup, Markup> {
    ctx.assert_can_edit_event(event_id)
        .await
        .map_err(|e| html_error(&format!("{e}"), "/events"))?;
    if prep_id > 0 {
        ctx.assert_can_edit_food_prep(prep_id)
            .await
            .map_err(|e| html_error(&format!("{e}"), "/events"))?;
    }
    let recipes = state.recipes().list().await.unwrap_or_default();

    // If prep_id is provided, get existing prep data
    let prep = if prep_id > 0 {
        state
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

    Ok(html! {
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
                            option value=(recipe.id)
                                selected[recipe.id == prep.recipe_id] {
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
    })
}

async fn save_food_prep(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(form): Form<FoodPrepForm>,
) -> MResponse {
    ctx.assert_can_edit_event(form.event_id).await?;
    if let Some(prep_id) = form.prep_id {
        if prep_id > 0 {
            ctx.assert_can_edit_food_prep(prep_id).await?;
        }
    }
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

    event_detail_tab::event_form(foodlib, ctx, Path(form.event_id)).await
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
            td data-label="Recipe" { (recipe.name) }
            td data-label="Prep Date" { (prep_date) }
            td data-label="Use From" { (use_from) }
            td data-label="Use Until" { (use_until) }
            td class="no-label" {
                form class="m-0" action=(format!("/events/edit/export_food_prep_pdf/{}", prep.id)) {
                    button class="btn btn-primary" { "Print" }
                }
            }
            td class="no-label" {
                button class="btn btn-primary"
                    hx-target="#content"
                    hx-push-url="true"
                    hx-get=(format!("/events/edit/food_prep/edit/{}/{}", event_id, prep.id)) {
                    "Edit"
                }
            }
            td class="no-label" {
                button class="btn btn-cancel"
                    hx-target="#content"
                    hx-get=(format!("/events/edit/food_prep/delete_dialog/{}/{}", event_id, prep.id)) {
                    "Delete"
                }
            }
        }
    }
}

pub async fn render_food_prep(
    foodlib: FoodLib,
    event_id: i32,
    default_date: time::OffsetDateTime,
) -> MResponse {
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
    let mut all_recipes = foodlib.recipes().list().await.unwrap_or_default();
    all_recipes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(html! {
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Food Preparation" }
        }
        table class="w-full text-inherit table-auto object-center responsive-card" {
            thead {
                tr {
                    th { "Recipe" }
                    th { "Prep Date" }
                    th { "Use From" }
                    th { "Use Until" }
                    th {} th {} th {}
                }
            }
            tbody {
                (food_prep_add_row(event_id, &all_recipes, default_date))
                @for prep in preps {
                    @if let Some(recipe) = recipe_map.get(&prep.recipe_id) {
                        (format_food_prep(event_id, &prep, recipe))
                    }
                }
            }
        }
    })
}

/// Inline add-row for food prep. All fields fit, so submit directly via POST.
/// Editing still uses the detail page (consistency with shopping tours).
/// Both `prep_date` and `use_until` default to the supplied date so the user
/// never submits an ambiguous empty `datetime-local` (which browsers serialize
/// as `""` when the time portion is blank).
fn food_prep_add_row(
    event_id: i32,
    recipes: &[Recipe],
    default_date: time::OffsetDateTime,
) -> Markup {
    let url = format!("/events/edit/food_prep/create_inline/{}", event_id);
    let fmt = format_description!("[year]-[month]-[day]T[hour]:[minute]");
    let prep_value = default_date.format(fmt).unwrap_or_default();
    let until_value = (default_date + time::Duration::days(1))
        .format(fmt)
        .unwrap_or_default();
    html! {
        tr id="food-prep--1" {
            td data-label="Recipe" {
                select name="recipe_id" class="text w-full" required="required" {
                    option value="" { "Select recipe..." }
                    @for r in recipes {
                        option value=(r.id) { (r.name) }
                    }
                }
            }
            td data-label="Prep Date" { input class="text w-full" type="datetime-local" name="prep_date" value=(prep_value) required="required"; }
            td data-label="Use From" { input class="text w-full" type="datetime-local" name="use_from"; }
            td data-label="Use Until" { input class="text w-full" type="datetime-local" name="use_until" value=(until_value) required="required"; }
            td class="no-label" {} td class="no-label" {}
            td class="no-label" {
                button class="btn btn-primary" type="button"
                    hx-post=(url)
                    hx-include="closest tr"
                    hx-target="#content" { "Add" }
            }
        }
    }
}
