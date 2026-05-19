use crate::FoodLib;
use axum::{
    extract::{Form, Path, State},
    routing::{delete, get, post},
};
use bigdecimal::ToPrimitive;
use foodlib_new::{
    auth_context::AuthCtx,
    error::{Error, Result},
    event::{Event, SourceOverrideView},
    meal::Meal,
};
use maud::{html, Markup};
use num::FromPrimitive;
use serde::Deserialize;
use time::macros::format_description;

mod event_edit_meal_tab;
mod food_prep;
mod shopping_tours;

use crate::{
    frontend::{move_group, MResponse},
    MyAppState,
};

pub(crate) fn event_detail_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/{event_id}", post(update_event))
        .route("/{event_id}/overrides/{source_id}", post(update_override))
        .route("/{event_id}/overrides/{source_id}", delete(delete_override))
        .route(
            "/{event_id}/overrides/{source_id}/delete_dialog",
            get(delete_override_dialog),
        )
        .route("/export_pdf/{meal_id}", get(export_recipe_pdf))
        .route("/export_food_prep_pdf/{prep_id}", get(export_food_prep_pdf))
        .route("/dish_labels/{event_id}", get(export_dish_labels_flat))
        .route("/dish_labels_tent/{event_id}", get(export_dish_labels_tent))
        .route("/allergen_overview/{event_id}", get(export_event_allergens))
        .route("/{event_id}", get(event_form))
        .route("/{event_id}/meal-prices", get(render_meal_prices))
        .route("/{event_id}/uncovered-ingredients", get(render_ingredients_without_tours))
        .route(
            "/ingredients-per-serving/{meal_id}",
            get(ingredients_per_serving),
        )
        .route("/delete/{event_id}/{meal_id}", get(delete_meal_dialog))
        .nest(
            "/event_edit_meal",
            event_edit_meal_tab::event_edit_meal_router(),
        )
        .nest("/shopping_tours", shopping_tours::shopping_tour_router())
        .nest("/food_prep", food_prep::food_prep_router())
        .route("/shift/{event_id}", post(handle_shift_times))
}

#[derive(Deserialize)]
struct ShiftTimesForm {
    new_first_meal_start: String,
}

async fn handle_shift_times(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
    Form(form): Form<ShiftTimesForm>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let target = shopping_tours::parse_datetime_local(&form.new_first_meal_start)?;
    foodlib.events().shift_event_times(event_id, target).await?;
    event_form(foodlib, ctx, Path(event_id)).await
}


async fn render_meal_prices(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> MResponse {
    use bigdecimal::ToPrimitive;
    use std::collections::HashMap;

    ctx.assert_can_edit_event(event_id).await?;
    let meals = foodlib.meals().get_event_meals(event_id).await?;
    let prices = foodlib.meals().get_event_meal_prices(event_id).await?;
    let price_by_meal: HashMap<i32, f64> = prices
        .into_iter()
        .map(|(id, p)| (id, p.to_f64().unwrap_or_default()))
        .collect();

    Ok(html! {
        @for meal in &meals {
            @let per_serving = price_by_meal.get(&meal.meal_id).copied().unwrap_or(0.0)
                / meal.servings as f64;
            // hx-swap-oob targets the placeholder by id; the outer element is
            // discarded by HTMX once the swap completes.
            td id=(format!("meal-price-{}", meal.meal_id)) hx-swap-oob="true" data-label="Price" {
                (format!("{:.3}€", per_serving))
            }
        }
    })
}

pub async fn delete_override_dialog(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let overrides = foodlib.events().get_source_overrides(event_id).await?;
    let source = overrides
        .iter()
        .find(|s| s.source_id == source_id)
        .ok_or_else(|| Error::NotFound {
            entity: "SourceOverride",
            id: source_id.to_string(),
        })?;

    Ok(html! {
        dialog open="true" class="dialog" id="delete" {
            p class="text-2xl" { (format!("Do you really want to delete Source Override: {} from {}", source.ingredient_name, source.store_name)) }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get=(format!("/events/edit/{}", event_id)) hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/edit/{}/overrides/{}", event_id, source_id)) { "Confirm Delete" }
            }
        }
    })
}

pub async fn delete_override(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    foodlib
        .events()
        .delete_source_override(event_id, source_id)
        .await?;
    event_form(foodlib, ctx, Path(event_id)).await
}

pub async fn delete_meal_dialog(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let meal = foodlib.meals().get_meal(meal_id).await?;

    Ok(html! {
        dialog open="true" class="dialog" id="delete" {
            p class="text-2xl" { (format!("Do you really want to delete meal: {}", meal.name)) }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get=(format!("/events/edit/{}", event_id)) hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/edit/event_edit_meal/{}/{}", event_id, meal_id)) { "Confirm Delete" }
            }
        }
    })
}

pub async fn export_recipe_pdf(
    State(state): State<MyAppState>,
    Path(meal_id): Path<i32>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>)> {
    let recipe_info = state.export().fetch_meal_recipe(meal_id).await?;
    let title = recipe_info.name.to_owned();
    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_recipes(recipe_info).await;
    #[cfg(not(feature = "typst"))]
    let result = Err(foodlib_new::Error::Misc(
        "Server compiled without typst support".into(),
    ));
    let recipe = result?;

    let headers = [
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.pdf\"", title),
        ),
        (
            axum::http::header::CONTENT_TYPE,
            "application/pdf".to_string(),
        ),
    ];
    Ok((headers, recipe))
}

pub async fn export_food_prep_pdf(
    State(state): State<MyAppState>,
    Path(prep_id): Path<i32>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>)> {
    let recipe_info = state.export().fetch_food_prep_recipe(prep_id).await?;
    let title = format!("{}_prep", recipe_info.name);

    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_recipes(recipe_info).await;
    #[cfg(not(feature = "typst"))]
    let result = Err(foodlib_new::Error::Misc(
        "Server compiled without typst support".into(),
    ));
    let recipe = result?;

    let headers = [
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.pdf\"", title),
        ),
        (
            axum::http::header::CONTENT_TYPE,
            "application/pdf".to_string(),
        ),
    ];
    Ok((headers, recipe))
}

pub async fn export_dish_labels_flat(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>)> {
    ctx.assert_can_edit_event(event_id).await?;
    dish_labels_response(&state, event_id, "flat").await
}

pub async fn export_dish_labels_tent(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>)> {
    ctx.assert_can_edit_event(event_id).await?;
    dish_labels_response(&state, event_id, "tent").await
}

async fn dish_labels_response(
    state: &MyAppState,
    event_id: i32,
    layout_key: &'static str,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>)> {
    let info = state.export().fetch_event_allergens(event_id).await?;
    let filename = format!("{}_dish_labels_{}", sanitize_filename(&info.event_name), layout_key);

    #[cfg(feature = "typst")]
    let result = {
        let layout = match layout_key {
            "tent" => foodlib_new::typst::DishLabelLayout::Tent,
            _ => foodlib_new::typst::DishLabelLayout::Flat,
        };
        foodlib_new::typst::export_dish_labels(info, layout).await
    };
    #[cfg(not(feature = "typst"))]
    let _ = layout_key;
    #[cfg(not(feature = "typst"))]
    let result = Err(foodlib_new::Error::Misc(
        "Server compiled without typst support".into(),
    ));
    let pdf = result.map_err(|e| foodlib_new::Error::Misc(e.to_string()))?;

    Ok((
        [
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}.pdf\""),
            ),
            (
                axum::http::header::CONTENT_TYPE,
                "application/pdf".to_string(),
            ),
        ],
        pdf,
    ))
}

pub async fn export_event_allergens(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>)> {
    ctx.assert_can_edit_event(event_id).await?;
    let info = state.export().fetch_event_allergens(event_id).await?;
    let filename = format!("{}_allergene", sanitize_filename(&info.event_name));

    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_event_allergens(info).await;
    #[cfg(not(feature = "typst"))]
    let result = Err(foodlib_new::Error::Misc(
        "Server compiled without typst support".into(),
    ));
    let pdf = result.map_err(|e| foodlib_new::Error::Misc(e.to_string()))?;

    Ok((
        [
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}.pdf\""),
            ),
            (
                axum::http::header::CONTENT_TYPE,
                "application/pdf".to_string(),
            ),
        ],
        pdf,
    ))
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub async fn event_form(foodlib: FoodLib, ctx: AuthCtx, Path(event_id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let stores = foodlib.stores().list().await?;
    let overrides = foodlib.events().get_source_overrides(event_id).await?;
    let ingredients = foodlib.ingredients().list().await?;
    let meals = foodlib.meals().get_event_meals(event_id).await?;
    let mut recipes = foodlib.recipes().list().await?;
    recipes.sort_by(|a, b| a.name.cmp(&b.name));
    // Default start-time for the inline meal add-row: latest meal's start_time
    // if any, otherwise "now". Saves the user retyping when adding meals
    // back-to-back.
    let default_meal_start = meals
        .iter()
        .map(|m| m.start_time)
        .max()
        .unwrap_or_else(time::OffsetDateTime::now_utc);
    // Default date for tour + food-prep add-rows: a day before the earliest
    // meal (or food prep), so shopping/prep naturally precedes consumption.
    // Falls back to "now" when nothing is scheduled yet.
    let preps = foodlib.events().get_food_prep_tasks(event_id).await?;
    let earliest = meals
        .iter()
        .map(|m| m.start_time)
        .chain(preps.iter().map(|p| p.prep_date))
        .min();
    let default_shop_date = earliest
        .map(|t| t - time::Duration::days(1))
        .unwrap_or_else(time::OffsetDateTime::now_utc);

    // Create a dummy source for the "add new" row
    let dummy_source = SourceOverrideView {
        event_id,
        ingredient_id: -1,
        source_id: -1,
        ingredient_name: String::new(),
        store_id: -1,
        store_name: String::new(),
    };

    let event = foodlib.events().get(event_id).await?;
    let owner_select = move_group::owner_select(&foodlib, &ctx, event.group_id).await?;

    let first_meal_start = meals.iter().map(|m| m.start_time).min();
    Ok(html! {
        section class="w-full flex flex-col gap-3 p-4 mb-4 rounded-lg border border-gray-700" {
            div class="flex flex-row items-center justify-center flex-wrap gap-x-4 gap-y-2" id="event_form" {
                label for="name" { "Name:" };
                input name="name" class="text " type="text" value=(&event.name);
                label for="comment" { "Comment:" };
                input name="comment" class="text " type="text" value=(&event.comment.unwrap_or_default());
                label for="budget" { "Budget:" };
                input name="budget" class="text  w-24" type="text" value=(event.budget.and_then(|x|x.to_f64()).unwrap_or(0.));
                (owner_select)
                button class="btn btn-primary " hx-post=(format!("/events/edit/{}", event_id)) hx-include="closest #event_form" hx-target="#content" hx-swap="innerHTML" hx-indicator=".htmx-indicator" {"Submit"}
                span class="htmx-indicator" { "Saving\u{a0}…" }
            }
            @if let Some(first_meal_start) = first_meal_start {
                @let shift_value = first_meal_start
                    .format(format_description!("[year]-[month]-[day]T[hour]:[minute]"))
                    .unwrap_or_default();
                form class="flex flex-row items-center justify-center flex-wrap gap-2"
                    hx-post=(format!("/events/edit/shift/{}", event_id))
                    hx-target="#content" {
                    label class="whitespace-nowrap" for="new_first_meal_start" { "Shift first meal to:" }
                    input class="text " type="datetime-local" name="new_first_meal_start"
                        value=(shift_value) required="required";
                    button class="btn btn-primary " type="submit" { "Shift all times" }
                }
            }
        }
        div hx-get=(format!("/events/edit/{}/uncovered-ingredients", event_id)) hx-trigger="load" hx-swap="outerHTML" {}
        div class="flex flex-col items-center gap-2 mb-3 mt-3 p-3 rounded-lg border border-gray-700" {
            p class="text-lg font-semibold" { "Allergen exports" }
            div class="flex flex-row flex-wrap gap-2 justify-center" {
                a class="btn btn-primary" target="_blank" href=(format!("/events/edit/allergen_overview/{}", event_id)) { "Event overview (PDF)" }
                a class="btn btn-primary" target="_blank" href=(format!("/events/edit/dish_labels/{}", event_id)) { "Dish labels — flat (6 / page)" }
                a class="btn btn-primary" target="_blank" href=(format!("/events/edit/dish_labels_tent/{}", event_id)) { "Dish labels — tent (fold)" }
            }
        }
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Meals" }
        }
        table class="w-full text-inherit table-auto object-center mb-2 responsive-card" {
            thead { tr { th { "Recipe" } th {"Start Time"} th { "Servings" } th { "Energy" } th { "Weight" } th { "Price" } th {} th {} th {} th {} }  }
            tbody class="text-center" id="meals-tbody" {
                (meal_add_row(event_id, &recipes, default_meal_start))
                @for meal in &meals {
                    (format_event_meal(event_id, meal))
                }
                tr hx-get=(format!("/events/edit/{}/meal-prices", event_id)) hx-trigger="load" hx-swap="delete" {}
            }
        }
        datalist id="ingredients" {
            @for ingredient in ingredients {
                option value=(ingredient.name) {}
            }
        }
        (render_shopping_tours(&foodlib, event_id, &stores, default_shop_date).await?)
        (food_prep::render_food_prep(foodlib, event_id, default_shop_date).await?)
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Ingredient Sources Overrides" }
        }
        table class="w-full text-inherit table-auto object-center responsive-card" {
            thead { tr { th { "Ingredient" } th {"Store"} th {} th {} }  }
            tbody {
                (format_event_source_override(&dummy_source, &stores))
                @for over in overrides {
                    (format_event_source_override(&over, &stores))
                }
            }
        }
    })
}

async fn ingredients_per_serving(foodlib: FoodLib, meal_id: Path<i32>) -> MResponse {
    let event_meal_ingredients = foodlib.meals().get_meal_ingredients(meal_id.0).await?;
    let meal = foodlib.meals().get_meal(meal_id.0).await?;

    Ok(html! {
        dialog open="true" id="popup" class="w-1/2 dialog" {
            div class="flex-col items-center justify-center" {
                table class="w-full table-auto object-center table-fixed" {
                    thead { tr { th { "Ingredient" } th {"Amount"} th {"Energy"} th {"Price"} } }
                    tbody {
                        @for item in event_meal_ingredients {
                            (format_event_meal_ingredient(&item, meal.servings as f64))
                        }
                    }
                }
                button class="btn btn-cancel" hx-trigger="click from:body" hx-swap="delete" hx-target="#popup" hx-get="/" {"Close"}
            }
        }
    })
}

use bigdecimal::BigDecimal;
use std::collections::HashMap;

async fn render_ingredients_without_tours(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let mut uncovered_ingredients = foodlib
        .events()
        .get_ingredients_without_tour(event_id)
        .await?;
    // Remove water from ingredient list
    uncovered_ingredients.retain(|i| i.ingredient_id != 4);

    if uncovered_ingredients.is_empty() {
        return Ok(html! {});
    }

    // Group by store (when available)
    let mut by_store: HashMap<String, Vec<String>> = HashMap::new();
    let mut total_weight = BigDecimal::from(0);

    for ingredient in &uncovered_ingredients {
        let store_name = ingredient.store_name.clone();
        by_store.entry(store_name).or_default().push(format!(
            "{} ({:.2} kg)",
            ingredient.ingredient_name,
            ingredient
                .weight
                .clone()
                .unwrap_or(BigDecimal::from_f64(-1.).unwrap())
        ));

        total_weight += &ingredient.weight.clone().unwrap_or_default();
    }

    // Sort stores alphabetically for consistent display
    let mut stores: Vec<_> = by_store.into_iter().collect();
    stores.sort_by(|(a, _), (b, _)| a.cmp(b));

    Ok(html! {
        div class="bg-red-100 dark:bg-red-900 p-4 rounded-lg mb-4" {
            details {
                summary class="font-bold cursor-pointer" {
                    span class="text-red-600 dark:text-red-400" { "⚠️ Warning: " }
                    (format!("{} ingredients ({:.2} kg total) are not covered by any shopping tour",
                        uncovered_ingredients.len(), total_weight))
                }

                div class="mt-4 pl-4" {
                    @for (store, ingredients) in stores {
                        div class="mb-4" {
                            h4 class="font-semibold text-lg" { (store) }
                            ul class="list-disc ml-8" {
                                @for ingredient in ingredients {
                                    li { (ingredient) }
                                }
                            }
                        }
                    }

                    div class="mt-4 text-sm text-gray-700 dark:text-gray-300 p-2 bg-yellow-50 dark:bg-yellow-900 rounded" {
                        "Add a shopping tour for these ingredients or they won't be included in your shopping lists. If you don't want to use the store for this event, consider adding an ingredient source override."
                    }
                }
            }
        }
    })
}

pub async fn render_shopping_tours(
    foodlib: &FoodLib,
    event_id: i32,
    stores: &[foodlib_new::store::Store],
    default_date: time::OffsetDateTime,
) -> MResponse {
    let tours = foodlib.events().get_shopping_tours(event_id).await?;

    Ok(html! {
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Shopping Tours" }
        }
        table class="w-full text-inherit table-auto object-center responsive-card" {
            thead {
                tr {
                    th { "Date" }
                    th { "Store" }
                    th {} th {}
                }
            }
            tbody {
                (tour_add_row(event_id, stores, default_date))
                @for tour in tours {
                    tr {
                        td data-label="Date" { (tour.tour_date.format(&time::format_description::parse("[day].[month] [hour]:[minute]").unwrap()).unwrap()) }
                        td data-label="Store" { (tour.store_name.clone().unwrap_or_default()) }
                        td class="no-label" {
                            button class="btn btn-primary"
                                hx-get=(format!("/events/edit/shopping_tours/edit/{}/{}", event_id, tour.id))
                                hx-swap="innerHtml show:window:top"
                                hx-push-url="true"
                                hx-target="#content" { "Edit" }
                        }
                        td class="no-label" {
                            button class="btn btn-cancel"
                                hx-delete=(format!("/events/edit/shopping_tours/{}", tour.id))
                                hx-swap="delete"
                                hx-target="closest tr" { "Delete" }
                        }
                    }
                }
            }
        }
    })
}

/// Inline add-row for shopping tours. All fields fit in the row, so submit
/// directly — no detail-page round-trip. Editing an existing tour still goes
/// through the detail page (it carries the shopping-list + inventory UI).
fn tour_add_row(
    event_id: i32,
    stores: &[foodlib_new::store::Store],
    default_date: time::OffsetDateTime,
) -> Markup {
    let url = format!("/events/edit/shopping_tours/create_inline/{}", event_id);
    let date_value = default_date
        .format(format_description!("[year]-[month]-[day]T[hour]:[minute]"))
        .unwrap_or_default();
    html! {
        tr id="tour--1" {
            td data-label="Date" { input class="text w-full" type="datetime-local" name="date" value=(date_value) required="required"; }
            td data-label="Store" {
                select name="store_id" class="text w-full" required="required" {
                    option value="" { "Select store..." }
                    @for s in stores {
                        option value=(s.id) { (s.name) }
                    }
                }
            }
            td class="no-label" {}
            td class="no-label" {
                button class="btn btn-primary" type="button"
                    hx-post=(url)
                    hx-include="closest tr"
                    hx-target="#content" { "Add" }
            }
        }
    }
}

fn format_event_meal_ingredient(
    event_meal_ingredient: &foodlib_new::meal::MealIngredient,
    portions: f64,
) -> Markup {
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x, unit)) } };

    html! {
        tr {
            td { (event_meal_ingredient.ingredient) }
            (format(event_meal_ingredient.weight.to_f64().unwrap_or_default() / portions * 1000., "g"))
            (format(event_meal_ingredient.energy.to_f64().unwrap_or_default() / portions / 100. , "kj"))
            (format(event_meal_ingredient.price.to_f64().unwrap_or_default() / portions, "€"))
        }
    }
}

#[derive(Deserialize)]
struct EventForm {
    name: String,
    comment: String,
    budget: Option<f64>,
    group_id: i32,
}

async fn update_event(
    foodlib: FoodLib,
    ctx: AuthCtx,
    event_id: Path<i32>,
    event_data: Form<EventForm>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id.0).await?;
    let current = foodlib.events().get(event_id.0).await?;
    if current.group_id != event_data.group_id {
        move_group::assert_can_move_to(&ctx, event_data.group_id)?;
    }
    let budget = event_data.budget.and_then(bigdecimal::BigDecimal::from_f64);

    let event = Event {
        id: event_id.0,
        name: event_data.name.clone(),
        comment: (!event_data.comment.is_empty()).then(|| event_data.comment.clone()),
        budget,
        group_id: 0, // Placeholder; ops-level update ignores group_id
    };

    foodlib.events().update(event).await?;

    if current.group_id != event_data.group_id {
        foodlib
            .events()
            .set_group(event_id.0, event_data.group_id)
            .await?;
    }

    event_form(foodlib, ctx, event_id).await
}

#[derive(Deserialize, Debug)]
struct SourceData {
    ingredient: String,
    store_id: i32,
}

async fn update_override(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((event_id, _ingredient_id)): Path<(i32, i32)>,
    Form(source): Form<SourceData>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    let ingredient = foodlib
        .ingredients()
        .get_by_name(&source.ingredient)
        .await?;
    let sources = foodlib.ingredients().get_sources(ingredient.id).await?;
    let source = sources
        .iter()
        .find(|s| s.store_id == source.store_id)
        .ok_or(Error::Misc("This Source is not defined for this ingredient! \n Add this source to the ingredient in the ingredients tab.".to_string()))?;
    foodlib
        .events()
        .set_source_override(event_id, source.id)
        .await?;

    event_form(foodlib, ctx, Path(event_id)).await
}

fn format_event_source_override(
    source_override: &SourceOverrideView,
    stores: &[foodlib_new::store::Store],
) -> Markup {
    let option = |store: &foodlib_new::store::Store, source_store| match store.id == source_store {
        false => html! {
            option
                label=(store.name)
                value=(store.id) {
                (store.name)
            }
        },
        true => html! {
            option
                label=(store.name)
                value=(store.id)
                selected {(store.name)}
        },
    };

    let button = match source_override.ingredient_id {
        -1 => html! {
            button class="btn btn-primary" hx-target="#content" hx-post=(format!("/events/edit/{}/overrides/{}", source_override.event_id, source_override.ingredient_id)) hx-include="closest tr" { "Add" }
        },
        _ => html! {
            button class="btn btn-primary" hx-target="#content" hx-post=(format!("/events/edit/{}/overrides/{}", source_override.event_id, source_override.ingredient_id)) hx-include="closest tr" { "Save" }
        },
    };

    html! {
        tr {
            td data-label="Ingredient" { input name="ingredient" class="text" type="text" list="ingredients" value=(source_override.ingredient_name) placeholder="Ingredient Name" required="true"; }
            td data-label="Store" {
                select name="store_id" id="stores" required="true" class="text" {
                    @for store in stores {
                        (option(store, source_override.store_id))
                    }
                }
            }
            td class="no-label" { (button) }
            td class="no-label" { @if source_override.ingredient_id != -1 {
                button class="btn btn-cancel" hx-get=(format!("/events/edit/{}/overrides/{}/delete_dialog", source_override.event_id, source_override.source_id)) hx-target="this" hx-swap="outerHTML" { "Delete" } }}
        }
    }
}

/// Inline add-row for meals. The visible columns mirror the table; the Add
/// button navigates (push-url) to the meal-detail page with the typed values
/// prepopulated via querystring, so the user can fill the remaining required
/// fields (place, end_time, energy, comment) before saving.
fn meal_add_row(
    event_id: i32,
    recipes: &[foodlib_new::recipe::Recipe],
    default_start: time::OffsetDateTime,
) -> Markup {
    let url = format!("/events/edit/event_edit_meal/{}/-1", event_id);
    let start_value = default_start
        .format(format_description!("[year]-[month]-[day]T[hour]:[minute]"))
        .unwrap_or_default();
    html! {
        tr class="meal-row" id="meal--1" {
            td data-label="Recipe" {
                select name="recipe_id" class="text w-full" required="required" {
                    option value="" { "Select recipe..." }
                    @for r in recipes {
                        option value=(r.id) { (r.name) }
                    }
                }
            }
            td data-label="Start Time" { input class="text w-full" type="datetime-local" name="start_time" value=(start_value); }
            td data-label="Servings" { input class="text w-full" type="number" name="servings" min="1" placeholder="Servings"; }
            td class="no-label" {} td class="no-label" {} td class="no-label" {} td class="no-label" {} td class="no-label" {} td class="no-label" {}
            td class="no-label" {
                button class="btn btn-primary" type="button"
                    hx-get=(url)
                    hx-include="closest tr"
                    hx-target="#content"
                    hx-push-url="true" { "Add" }
            }
        }
    }
}

fn format_event_meal(event_id: i32, event_meal: &Meal) -> Markup {
    let format = |label: &'static str, x, unit| html! { td data-label=(label) { (&format!("{:.3}{}", x, unit)) } };
    let time_format = format_description!("[day].[month] [hour]:[minute]");
    let comment = event_meal.comment.as_deref().unwrap_or("").trim().to_string();

    html! {
        tr class="meal-row" {
            td data-label="Recipe" {
                (event_meal.name)
                @if !comment.is_empty() {
                    br;
                    span class="text-sm opacity-60 italic" { (comment) }
                }
            }
            td data-label="Start Time" { (event_meal.start_time.format(&time_format).unwrap()) }
            td data-label="Servings" { (event_meal.servings) }
            (format("Energy", event_meal.energy.to_f64().unwrap_or_default(), "kj"))
            (format("Weight", event_meal.weight.to_f64().unwrap_or_default() / event_meal.servings as f64 * 1000., "g"))
            td data-label="Price" id=(format!("meal-price-{}", event_meal.meal_id)) { "…" }
            td class="no-label" { button class="btn btn-primary" hx-swap="afterend" hx-get=(format!("/events/edit/ingredients-per-serving/{}", event_meal.meal_id)) {"Ing./serving"} }
            td class="no-label" { form class="m-0" action=(format!("/events/edit/export_pdf/{}", event_meal.meal_id)) { button class="btn btn-primary" {"Print"} } }
            td class="no-label" { button class="btn btn-primary"
                hx-target="#content"
                hx-push-url="true"
                hx-get=(format!("/events/edit/event_edit_meal/{}/{}", event_id, event_meal.meal_id)) {"Edit"} }
            td class="no-label" { button class="btn btn-cancel" hx-target="#content" hx-get=(format!("/events/edit/delete/{}/{}", event_id, event_meal.meal_id)) {"Delete"} }
        }
    }
}
