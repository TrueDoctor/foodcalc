use crate::FoodLib;
use axum::{
    extract::{Form, Path, State},
    routing::{delete, get, post},
};
use bigdecimal::ToPrimitive;
use foodlib_new::{
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

use crate::{frontend::MResponse, MyAppState};

pub(crate) fn event_detail_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/{event_id}", post(update_event))
        .route("/{event_id}/overrides/{source_id}", post(update_override))
        .route("/{event_id}/overrides/{source_id}", delete(delete_override))
        .route("/export/pdf/{meal_id}", get(export_recipe_pdf))
        .route(
            "/{event_id}/overrides/{source_id}/delete_dialog",
            get(delete_override_dialog),
        )
        .route("/export_pdf/{meal_id}", get(export_recipe_pdf))
        .route("/{event_id}", get(event_form))
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
}

pub async fn delete_override_dialog(
    foodlib: FoodLib,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> MResponse {
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
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> MResponse {
    foodlib
        .events()
        .delete_source_override(event_id, source_id)
        .await?;
    event_form(foodlib, Path(event_id)).await
}

pub async fn delete_meal_dialog(
    foodlib: FoodLib,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> MResponse {
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
    let recipe_info = state.fetch_meal_recipe(meal_id).await?;
    let title = recipe_info.name.to_owned();
    #[cfg(feature = "typst")]
    let result = foodlib::typst::export_recipes(recipe_info).await;
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

pub async fn event_form(foodlib: FoodLib, Path(event_id): Path<i32>) -> MResponse {
    let stores = foodlib.stores().list().await?;
    let overrides = foodlib.events().get_source_overrides(event_id).await?;
    let ingredients = foodlib.ingredients().list().await?;
    let meals = foodlib.meals().get_event_meals(event_id).await?;

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

    Ok(html! {
        div class="flex flex-row items-center justify-center gap-4" id="event_form" {
            label for="name" { "Name:" };
            input name="name" class="text" type="text" value=(&event.name);
            label for="comment" { "Comment:" };
            input name="comment" class="text" type="text" value=(&event.comment.unwrap_or_default());
            label for="budget" { "Budget:" };
            input name="budget" class="text" type="text" value=(event.budget.and_then(|x|x.to_f64()).unwrap_or(0.));
            div class="flex flex-row items-center justify-center gap-4" {
                button class="btn btn-primary" hx-post=(format!("/events/edit/{}", event_id)) hx-include="closest #event_form" hx-target="#content" hx-swap="innerHTML" hx-indicator=".htmx-indicator" {"Submit"}
                span class="htmx-indicator" { "Saving\u{a0}…" }
            }
        }
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Meals" }
        }
        div class="flex flex-row items-center justify-center mb-2" {
            button class="btn btn-primary" hx-target="#content"
                hx-push-url="true"
                hx-get=(format!("/events/edit/event_edit_meal/{}/-1", event_id)) {"Add Meal"}
        }
        table class="w-full text-inherit table-auto object-center mb-2 table-fixed" {
            thead { tr { th { "Recipe" } th {"Start Time"} th { "servings" } th { "Energy" } th { "Weight" } th { "Price" } th {} th {} th {} th {} }  }
            tbody class="text-center" {
                @for meal in meals {
                    (format_event_meal(event_id, &meal))
                }
            }
        }
        datalist id="ingredients" {
            @for ingredient in ingredients {
                option value=(ingredient.name) {}
            }
        }
        (render_shopping_tours(&foodlib, event_id).await?)
        (food_prep::render_food_prep(foodlib, event_id).await?)
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Ingredient Sources Overrides" }
        }
        table class="w-full text-inherit table-auto object-center table-fixed" {
            thead { tr { th class="w-1/3" { "Ingredient" } th class="w-1/3" {"Store"} th {} th {} }  }
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

    Ok(html! {
        dialog open="true" id="popup" class="w-1/2 dialog" {
            div class="flex-col items-center justify-center" {
                table class="w-full table-auto object-center table-fixed" {
                    thead { tr { th { "Ingredient" } th {"Amount"} th {"Energy"} th {"Price"} } }
                    tbody {
                        @for item in event_meal_ingredients {
                            (format_event_meal_ingredient(&item))
                        }
                    }
                }
                button class="btn btn-cancel" hx-trigger="click from:body" hx-swap="delete" hx-target="#popup" hx-get="/" {"Close"}
            }
        }
    })
}

pub async fn render_shopping_tours(foodlib: &FoodLib, event_id: i32) -> MResponse {
    let tours = foodlib.events().get_shopping_tours(event_id).await?;

    Ok(html! {
        div class="flex-col items-center justify-center mb-2" {
            p class="text-2xl" { "Shopping Tours" }
        }
        div class="flex flex-row items-center justify-center mb-2" {
            button class="btn btn-primary"
                hx-get=(format!("/events/edit/shopping_tours/add/{}", event_id))
                hx-swap="innerHtml show:window:top"
                hx-push-url="true"
                hx-target="#content" { "Add Shopping Tour" }
        }
        table class="w-full text-inherit table-auto object-center table-fixed" {
            thead {
                tr {
                    th { "Date" }
                    th { "Store" }
                    th {} th {} th {}
                }
            }
            tbody {
                @for tour in tours {
                    tr {
                        td { (tour.tour_date.format(&time::format_description::parse("[day].[month] [hour]:[minute]").unwrap()).unwrap()) }
                        td { (tour.store_name.clone().unwrap_or_default()) }
                        td {
                            button class="btn btn-primary"
                                hx-get=(format!("/events/edit/shopping_tours/edit/{}/{}", event_id, tour.id))
                                hx-swap="innerHtml show:window:top"
                                hx-push-url="true"
                                hx-target="#content" { "Edit" }
                        }
                        td {
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

fn format_event_meal_ingredient(
    event_meal_ingredient: &foodlib_new::meal::MealIngredient,
) -> Markup {
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x, unit)) } };

    html! {
        tr {
            td { (event_meal_ingredient.ingredient) }
            (format(event_meal_ingredient.weight.to_f64().unwrap_or_default() * 1000., "g"))
            (format(event_meal_ingredient.energy.to_f64().unwrap_or_default() / 100. , "kj"))
            (format(event_meal_ingredient.price.to_f64().unwrap_or_default(), "€"))
        }
    }
}

#[derive(Deserialize)]
struct EventForm {
    name: String,
    comment: String,
    budget: Option<f64>,
}

async fn update_event(
    foodlib: FoodLib,
    event_id: Path<i32>,
    event_data: Form<EventForm>,
) -> MResponse {
    let budget = event_data.budget.and_then(bigdecimal::BigDecimal::from_f64);

    let event = Event {
        id: event_id.0,
        name: event_data.name.clone(),
        comment: (!event_data.comment.is_empty()).then(|| event_data.comment.clone()),
        budget,
        owner_id: 0, // We don't change the owner
    };

    foodlib.events().update(event).await?;
    event_form(foodlib, event_id).await
}

#[derive(Deserialize, Debug)]
struct SourceData {
    ingredient: String,
    store_id: i32,
}

async fn update_override(
    foodlib: FoodLib,
    Path((event_id, _ingredient_id)): Path<(i32, i32)>,
    Form(source): Form<SourceData>,
) -> MResponse {
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

    event_form(foodlib, Path(event_id)).await
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
            td { input name="ingredient" class="text" type="text" list="ingredients" value=(source_override.ingredient_name) placeholder="Ingredient Name" required="true"; }
            td {
                select name="store_id" id="stores" required="true" class="text" {
                    @for store in stores {
                        (option(store, source_override.store_id))
                    }
                }
            }
            td { (button) }
            td { @if source_override.ingredient_id != -1 {
                button class="btn btn-cancel" hx-get=(format!("/events/edit/{}/overrides/{}/delete_dialog", source_override.event_id, source_override.source_id)) hx-target="this" hx-swap="outerHTML" { "Delete" } }}
        }
    }
}

fn format_event_meal(event_id: i32, event_meal: &Meal) -> Markup {
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x, unit)) } };
    let time_format = format_description!("[day].[month] [hour]:[minute]");

    html! {
        tr {
            td { (event_meal.name) }
            td { (event_meal.start_time.format(&time_format).unwrap()) }
            td { (event_meal.servings) }
            (format(event_meal.energy.to_f64().unwrap_or_default(), "kj"))
            (format(event_meal.weight.to_f64().unwrap_or_default() / event_meal.servings as f64 * 1000., "g"))
            (format(event_meal.price.to_f64().unwrap_or_default() / event_meal.servings as f64, "€"))
            td { button class="btn btn-primary" hx-swap="afterend" hx-get=(format!("/events/edit/ingredients-per-serving/{}", event_meal.meal_id)) {"Ingredients per serving"} }
            td { form class="m-0" action=(format!("/events/edit/export_pdf/{}", event_meal.meal_id)) { button class="btn btn-primary" {"Print"} } }
            td { button class="btn btn-primary"
                hx-target="#content"
                hx-push-url="true"
                hx-get=(format!("/events/edit/event_edit_meal/{}/{}", event_id, event_meal.meal_id)) {"Edit"} }
            td { button class="btn btn-cancel" hx-target="#content" hx-get=(format!("/events/edit/delete/{}/{}", event_id, event_meal.meal_id)) {"Delete"} }
        }
    }
}
