use crate::{
    frontend::{events_tab::event_detail_tab, html_error, icons, MResponse, WebCtx},
    AppState,
};
use axum::{
    extract::{Form, Path, State},
    routing::{delete, get, post},
};
use foodlib_new::entities::event::{CreateFoodPrep, FoodPrep};
use foodlib_new::entities::recipe::Recipe;
use foodlib_new::types::ids::{EventId, FoodPrepId, RecipeId};
use foodlib_new::AppCtx;
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

pub(crate) fn food_prep_router() -> axum::Router<AppState> {
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
    recipe_id: RecipeId,
    prep_date: String,
    use_from: Option<String>,
    use_until: String,
}

async fn create_inline(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
    Form(form): Form<InlinePrepForm>,
) -> MResponse {
    let prep_date = super::shopping_tours::parse_datetime_local(&form.prep_date)?;
    let use_from = form
        .use_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(super::shopping_tours::parse_datetime_local)
        .transpose()?;
    let use_until = super::shopping_tours::parse_datetime_local(&form.use_until)?;
    foodlib_new::events::logistics::add_food_prep(
        &state,
        &actor,
        CreateFoodPrep {
            event_id,
            recipe_id: form.recipe_id,
            prep_date,
            use_from,
            use_until,
        },
    )
        .await?;
    event_detail_tab::event_form(State(state), WebCtx(actor), Path(event_id)).await
}

#[derive(Deserialize)]
struct FoodPrepForm {
    event_id: EventId,
    prep_id: Option<FoodPrepId>,
    recipe_id: RecipeId,
    prep_date: String,
    use_from: Option<String>,
    use_until: String,
}

async fn add_food_prep(
    state: State<AppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
) -> Result<Markup, Markup> {
    food_prep_form(state, WebCtx(actor), Path((event_id, FoodPrepId(-1)))).await
}

async fn delete_food_prep(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((event_id, prep_id)): Path<(EventId, FoodPrepId)>,
) -> MResponse {
    foodlib_new::events::logistics::delete_food_prep(&state, &actor, event_id, prep_id).await?;
    event_detail_tab::event_form(State(state), WebCtx(actor), Path(event_id)).await
}

async fn delete_food_prep_dialog(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((event_id, prep_id)): Path<(EventId, FoodPrepId)>,
) -> MResponse {
    let prep = foodlib_new::events::logistics::get_food_prep_with_permission(
        &state, &actor, event_id, prep_id,
    )
        .await?;
    let recipe = foodlib_new::recipes::get_recipe(&state, prep.recipe_id).await?;
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
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((event_id, prep_id)): Path<(EventId, FoodPrepId)>,
) -> Result<Markup, Markup> {
    let existing = foodlib_new::events::logistics::get_food_prep_with_permission(
        &state, &actor, event_id, prep_id,
    )
        .await
        .map_err(|e| html_error(&format!("{e}"), "/events"))?;
    let recipes = foodlib_new::recipes::list_recipes(&state)
        .await
        .unwrap_or_default();

    let prep = if prep_id.is_valid() {
        existing
    } else {
        FoodPrep {
            id: FoodPrepId(-1),
            event_id,
            recipe_id: RecipeId(0),
            prep_date: OffsetDateTime::now_utc(),
            use_from: None,
            use_until: OffsetDateTime::now_utc(),
        }
    };

    let time_format = format_description!("[year]-[month]-[day]T[hour]:[minute]");

    Ok(html! {
        div class="flex-col space-y-4 w-full" {
            h2 class="text-xl" {
                @if prep_id.is_valid() {
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
                        @if prep_id.is_valid() {
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
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<FoodPrepForm>,
) -> MResponse {
    let prep_date = super::shopping_tours::parse_datetime_local(&form.prep_date)?;
    let use_from = form
        .use_from
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(super::shopping_tours::parse_datetime_local)
        .transpose()?;
    let use_until = super::shopping_tours::parse_datetime_local(&form.use_until)?;

    match form.prep_id {
        Some(prep_id) if prep_id.is_valid() => {
            foodlib_new::events::logistics::update_food_prep(
                &state,
                &actor,
                FoodPrep {
                    id: prep_id,
                    event_id: form.event_id,
                    recipe_id: form.recipe_id,
                    prep_date,
                    use_from,
                    use_until,
                },
            )
                .await?;
        }
        _ => {
            foodlib_new::events::logistics::add_food_prep(
                &state,
                &actor,
                CreateFoodPrep {
                    event_id: form.event_id,
                    recipe_id: form.recipe_id,
                    prep_date,
                    use_from,
                    use_until,
                },
            )
                .await?;
        }
    }

    event_detail_tab::event_form(State(state), WebCtx(actor), Path(form.event_id)).await
}

fn format_food_prep(event_id: EventId, prep: &FoodPrep, recipe_name: &str) -> Markup {
    let time_format = format_description!("[day].[month] [hour]:[minute]");
    let prep_date = prep.prep_date.format(time_format).unwrap_or_default();
    let use_from = prep
        .use_from
        .map(|d| d.format(time_format).unwrap_or_default())
        .unwrap_or_else(|| "-".to_string());
    let use_until = prep.use_until.format(time_format).unwrap_or_default();

    html! {
        tr {
            td data-label="Recipe" { (recipe_name) }
            td data-label="Prep Date" { (prep_date) }
            td data-label="Use From" { (use_from) }
            td data-label="Use Until" { (use_until) }
            td class="no-label" {
                div class="flex flex-row flex-nowrap gap-2 justify-end items-center" {
                    form class="m-0" action=(format!("/events/edit/export_food_prep_pdf/{}", prep.id)) {
                        button class="icon-btn" type="submit" title="Print" aria-label="Print" { (icons::print()) }
                    }
                    button class="icon-btn" title="Edit" aria-label="Edit"
                        hx-target="#content"
                        hx-push-url="true"
                        hx-get=(format!("/events/edit/food_prep/edit/{}/{}", event_id, prep.id)) { (icons::edit()) }
                    button class="icon-btn" title="Delete" aria-label="Delete"
                        hx-target="#content"
                        hx-get=(format!("/events/edit/food_prep/delete_dialog/{}/{}", event_id, prep.id)) { (icons::delete()) }
                }
            }
        }
    }
}

pub async fn render_food_prep(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    default_date: OffsetDateTime,
) -> MResponse {
    let foodlib_new::events::timeline::EventItemSources { preps, recipes, .. } =
        foodlib_new::events::timeline::gather_event_item_sources(state, actor, event_id).await?;

    let recipe_names: std::collections::HashMap<RecipeId, String> =
        recipes.iter().map(|r| (r.id, r.name.clone())).collect();
    let mut all_recipes = recipes;
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
                    th {}
                }
            }
            tbody {
                (food_prep_add_row(event_id, &all_recipes, default_date))
                @for prep in preps {
                    @if let Some(recipe_name) = recipe_names.get(&prep.recipe_id) {
                        (format_food_prep(event_id, &prep, recipe_name))
                    }
                }
            }
        }
    })
}

fn food_prep_add_row(
    event_id: EventId,
    recipes: &[Recipe],
    default_date: OffsetDateTime,
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
            td class="no-label" {
                div class="flex flex-row justify-end" {
                    button class="btn btn-primary" type="button"
                        hx-post=(url)
                        hx-include="closest tr"
                        hx-target="#content" { "Add" }
                }
            }
        }
    }
}