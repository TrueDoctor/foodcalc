use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use axum_login::login_required;
use bigdecimal::ToPrimitive;
#[cfg(feature = "typst")]
use foodlib::typst::export_recipes;
use foodlib::{Backend, Event, EventRecipeIngredient, Meal, SourceOverrideView, Store};
use maud::{html, Markup};
use num::FromPrimitive;
use serde::Deserialize;
use time::macros::format_description;

mod event_edit_meal_tab;

use crate::{
    frontend::{html_error, LOGIN_URL},
    MyAppState,
};

pub(crate) fn event_detail_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:event_id", post(update_event))
        .route("/:event_id/overrides/:source_id", post(update_override))
        .route("/:event_id/overrides/:source_id", delete(delete_override))
        .route("/export/pdf/:meal_id", get(export_recipe_pdf))
        .route(
            "/:event_id/overrides/:source_id/delete_dialog",
            get(delete_override_dialog),
        )
        .route("/export_pdf/:meal_id", get(export_recipe_pdf))
        .route_layer(login_required!(Backend, login_url = LOGIN_URL))
        .route("/:event_id", get(event_form))
        .route(
            "/ingredients-per-serving/:meal_id",
            get(ingredients_per_serving),
        )
        .route("/delete/:event_id/:meal_id", get(delete_meal_dialog))
        .nest(
            "/event_edit_meal",
            event_edit_meal_tab::event_edit_meal_router(),
        )
}

pub async fn delete_override_dialog(
    state: State<MyAppState>,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> Markup {
    let source = state
        .get_event_source_override(event_id, source_id)
        .await
        .unwrap_or_default();
    html! {
        dialog open="true" class="dialog" id="delete" {
            p class="text-2xl" { (format!("Do you really want to delete Source Override: {} from {}", source.ingredient, source.store)) }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get=(format!("/events/edit/{}", event_id)) hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/edit/{}/overrides/{}", event_id, source_id)) { "Confirm Delete" }
            }
        }
    }
}

pub async fn delete_override(
    state: State<MyAppState>,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> Markup {
    match state
        .delete_event_source_override_with_event_id(event_id, source_id)
        .await
    {
        Ok(_) => event_form(state, Path(event_id))
            .await
            .unwrap_or_else(|e| e),
        Err(_) => html_error("Failed to delete source override", "/events"),
    }
}

pub async fn delete_meal_dialog(
    state: State<MyAppState>,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> Markup {
    let meal = state.get_event_meal(meal_id).await.unwrap_or_default();

    html! {
        dialog open="true" class="dialog" id="delete" {
            p class="text-2xl" { (format!("Do you really want to delete meal: {}", meal.name)) }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get=(format!("/events/edit/{}", event_id)) hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/events/edit/event_edit_meal/{}/{}", event_id, meal_id)) { "Confirm Delete" }
            }
        }
    }
}

pub async fn export_recipe_pdf(
    State(state): State<MyAppState>,
    Path(meal_id): Path<i32>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>), Markup> {
    let Ok(recipe_info) = state.db_connection.fetch_meal_recipe(meal_id).await else {
        return Err(html_error("Meal fetching failed", "/events"));
    };
    let title = recipe_info.name.to_owned();

    #[cfg(feature = "typst")]
    let result = export_recipes(recipe_info).await;
    #[cfg(not(feature = "typst"))]
    let result = Err("Server not compiled with typst support.");

    match result {
        Ok(recipe) => {
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
        Err(error) => {
            log::error!("Failed to save recipe export: {}", error);
            Err(html_error("Failed to save recipe export", "/events"))
        }
    }
}

async fn event_form(state: State<MyAppState>, Path(event_id): Path<i32>) -> Result<Markup, Markup> {
    let stores = state
        .get_stores()
        .await
        .map_err(|e| html_error(&format!("Failed to fetch stores {e}"), "/events"))?;

    let overrides = state
        .get_event_source_overrides(event_id)
        .await
        .map_err(|e| html_error(&format!("Failed to fetch sources {e}"), "/events"))?;
    let ingredients = state
        .get_ingredients()
        .await
        .map_err(|e| html_error(&format!("Failed to fetch ingredients {e}"), "/events"))?;
    let meals = state
        .get_event_meals(event_id)
        .await
        .map_err(|e| html_error(&format!("Failed to fetch meals: {e}"), "/events"))?;
    let dummy_source = SourceOverrideView {
        ingredient_source_id: -1,
        ingredient_id: -1,
        event_id,
        ..Default::default()
    };

    let event = state
        .get_event(event_id)
        .await
        .ok_or(html_error("Failed to fetch event", "/events"))?;

    Ok(html! {
        div class="flex flex-row items-center justify-center gap-4" id="event_form" {
            label for="name" { "Name:" };
            input name="name" class="text" type="text" value=(&event.event_name);
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
            button class="btn btn-primary" hx-target="#content" hx-get=(format!("/events/edit/event_edit_meal/{}/-1", event_id)) {"Add Meal"}
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

fn format_event_source_override(source_override: &SourceOverrideView, stores: &[Store]) -> Markup {
    let option = |store: &Store, source_store| match store.store_id == source_store {
        false => html! {
            option
                label=(store.name)
                value=(store.store_id)
                { (store.name) }
        },
        true => html! {
            option
                label=(store.name)
                value=(store.store_id)
                selected {(store.name)}
        },
    };
    let button = |text| {
        html! {
            button class="btn btn-primary" hx-target="#content" hx-post=(format!("/events/edit/{}/overrides/{}", source_override.event_id, source_override.ingredient_id)) hx-include="closest tr" { (text) }
        }
    };

    let button = match source_override.ingredient_id {
        -1 => button("Add"),
        _ => button("Save"),
    };
    html! {
        tr {
            td { input name="ingredient" class="text" type="text" list="ingredients" value=(source_override.ingredient) placeholder="Ingredient Name" required="true"; }
            td {
                select name="store_id" id="stores" required="true" class="text" {
                    @for store in stores {
                        (option(store, source_override.store_id))
                    }
                }
            }
            td { (button) }
            td { @if source_override.ingredient_id != -1 {
                button class="btn btn-cancel" hx-get=(format!("/events/edit/{}/overrides/{}/delete_dialog", source_override.event_id, source_override.ingredient_source_id)) hx-target="this" hx-swap="outerHTML" { "Delete" } }}
        }
    }
}

async fn ingredients_per_serving(state: State<MyAppState>, meal_id: Path<i32>) -> Markup {
    let Ok(event_meal_ingredints) = dbg!(state.get_event_recipe_ingredients(meal_id.0).await)
    else {
        return html_error("Failed to fetch recipe ingredients", "/events");
    };

    html! {
        dialog open="true" id="popup" class="w-1/2 dialog" {
            div class="flex-col items-center justify-center" {
                table class="w-full table-auto object-center table-fixed" {
                    thead { tr { th { "Ingredient" } th {"Amount"} th {"Energy"} th {"Price"} } }
                    tbody {
                        @for event_meal_ingredint in event_meal_ingredints {
                            (format_event_meal_ingredient(&event_meal_ingredint))
                        }
                    }
                }
                button class="btn btn-primary" hx-swap="delete" hx-target="#popup" hx-get="/" {"Close"}
            }
        }
    }
}

fn format_event_meal_ingredient(event_meal_ingredint: &EventRecipeIngredient) -> Markup {
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x, unit)) } };

    html! {
        tr {
            td { (event_meal_ingredint.name) }
            (format(event_meal_ingredint.weight.to_f64().unwrap_or_default() * 1000., "g"))
            (format(event_meal_ingredint.energy.to_f64().unwrap_or_default() / 100. , "kj"))
            (format(event_meal_ingredint.price.to_f64().unwrap(), "€"))
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
    state: State<MyAppState>,
    event_id: Path<i32>,
    event_data: Form<EventForm>,
) -> impl IntoResponse {
    let budget = event_data.budget.and_then(FromPrimitive::from_f64);

    let event = Event {
        event_id: *event_id,
        event_name: event_data.name.clone(),
        comment: (!event_data.comment.is_empty()).then(|| event_data.comment.clone()),
        budget,
    };

    if (state.update_event(&event).await).is_ok() {
        (StatusCode::OK, event_form(state, event_id).await).into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Deserialize)]
struct SourceData {
    ingredient: String,
    store_id: i32,
}

async fn update_override(
    State(state): State<MyAppState>,
    Path((event_id, _ingredient_id)): Path<(i32, i32)>,
    Form(source): axum::extract::Form<SourceData>,
) -> Markup {
    match state
        .set_default_event_source_override(event_id, source.ingredient, source.store_id)
        .await
    {
        Err(_) => html_error("Failed to add ingredient source", "/events/edit/{event_id}"),
        Ok(_) => event_form(State(state), Path(event_id))
            .await
            .unwrap_or_else(|e| e),
    }
}

fn format_event_meal(event_id: i32, event_meal: &Meal) -> Markup {
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x,unit)) } };
    let time_format = format_description!("[day].[month] [hour]:[minute]");

    html! {
        tr {
            td { (event_meal.name) }
            td { (event_meal.start_time.format(&time_format).unwrap()) }
            td { (event_meal.servings) }
            (format(event_meal.energy.to_f64().unwrap_or_default(), "kj"))
            (format(event_meal.weight.to_f64().unwrap_or_default() /  event_meal.servings as f64 * 1000., "g"))
            (format(event_meal.price.to_f64().unwrap() / event_meal.servings as f64, "€"))
            td { button class="btn btn-primary" hx-swap="afterend" hx-get=(format!("/events/edit/ingredients-per-serving/{}", event_meal.meal_id)) {"Ingredients per serving"} }
            td { form class="m-0" action=(format!("/events/edit/export_pdf/{}", event_meal.meal_id)) { button class="btn btn-primary" {"Print"} } }
            td { button class="btn btn-primary" hx-target="#content" hx-get=(format!("/events/edit/event_edit_meal/{}/{}", event_id, event_meal.meal_id)) {"Edit"} }
            td { button class="btn btn-cancel" hx-target="#content" hx-get=(format!("/events/edit/delete/{}/{}", event_id, event_meal.meal_id)) {"Delete"} }

        }
    }
}
