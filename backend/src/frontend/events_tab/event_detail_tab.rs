use std::sync::Arc;

use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use axum_login::RequireAuthorizationLayer;
use bigdecimal::ToPrimitive;
use foodlib::{Event, EventRecipeIngredient, Meal, SourceOverrideView, Store, User};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgMoney;

use crate::{
    frontend::{html_error, LOGIN_URL},
    MyAppState,
};

pub(crate) fn event_detail_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:event_id", post(update_event))
        .route("/:event_id/overrides/:source_id", post(update_override))
        .route_layer(RequireAuthorizationLayer::<i64, User>::login_or_redirect(
            Arc::new(LOGIN_URL.into()),
            None,
        ))
        .route("/:event_id", get(event_form))
        .route(
            "/ingredients-per-serving/:meal_id",
            get(ingredients_per_serving),
        )
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

    Ok(html! {
        form class="flex flex-row items-center justify-center" action=(format!("/{}", event_id)) {
            input name="name" class="text" type="text";
            input name="comment" class="text" type="text";
            input name="budget" class="text" type="number";
            button class="btn btn-primary" type="submit" {"Submit"}
        }
        table class="w-full text-inherit table-auto object-center" {
            thead { tr { th { "Recipe" } th {"Start Time"} th { "servings" } th { "Energy" } th { "Weight" } th { "Price" } th {} }  }
            tbody {
                @for meal in meals {
                    (format_event_meal(&meal))
                }
            }
        }
        datalist id="ingredients" {
            @for ingredient in ingredients {
                option value=(ingredient.name) {}
            }
        }
        table class="w-full text-inherit table-auto object-center" {
            thead { tr { th { "Ingredient" } th {"Store"} th {} }  }
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
            td { input name="ingredient" class="text" type="text" list="ingredients" value=(source_override.ingredient); }
            td {
                select name="store_id" id="stores" required="true" class="text" {
                    @for store in stores {
                        (option(store, source_override.store_id))
                    }
                }
            }
            td { (button) }
        }
    }
}

async fn ingredients_per_serving(state: State<MyAppState>, meal_id: Path<i32>) -> Markup {
    let Ok(event_meal_ingredints) = dbg!(state.get_event_recipe_ingredients(meal_id.0).await)
    else {
        return html_error("Failed to fetch recipe ingredients", "/events");
    };

    html! {
        dialog open="true" id="popup" {
            div class="flex-col items-center justify-center" {
                table class="w-full table-auto object-center" {
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
            (format(event_meal_ingredint.price.0 as f64 / 100., "€"))
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
    let budget = event_data
        .budget
        .map(|budget| PgMoney((budget * 100.) as i64));

    dbg!(budget);
    let event = Event {
        event_id: event_id.clone(),
        event_name: event_data.name.clone(),
        comment: (!event_data.comment.is_empty()).then(|| event_data.comment.clone()),
        budget,
    };

    if let Ok(_) = state.update_event(&event).await {
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
    Path((event_id, ingredient_id)): Path<(i32, i32)>,
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

fn format_event_meal(event_meal: &Meal) -> Markup {
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x,unit)) } };
    html! {
        tr {
            td { (event_meal.name) }
            td { (event_meal.start_time) }
            td { (event_meal.servings) }
            (format(event_meal.energy.to_f64().unwrap_or_default(), "kj"))
            (format(event_meal.weight.to_f64().unwrap_or_default() /  event_meal.servings as f64 * 1000., "g"))
            (format(event_meal.price.0 as f64 / 100. / event_meal.servings as f64, "€"))
            td { button class="btn btn-primary" hx-swap="afterend" hx-get=(format!("/events/edit/ingredients-per-serving/{}", event_meal.meal_id)) {"Ingredients per serving"} }
        }
    }
}
