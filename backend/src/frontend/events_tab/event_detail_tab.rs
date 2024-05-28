use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use bigdecimal::ToPrimitive;
use foodlib::{Event, EventRecipeIngredient, Meal};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgMoney;

use crate::{frontend::html_error, MyAppState};

pub(crate) fn event_detail_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:event_id", get(event_form))
        .route("/:event_id", post(update_event))
        .route(
            "/ingredients-per-serving/:meal_id",
            get(ingredients_per_serving),
        )
}

async fn event_form(state: State<MyAppState>, event_id: Path<i32>) -> Markup {
    let Ok(meals) = state.db_connection.get_event_meals(event_id.0).await else {
        return html_error("Failed to fetch meals", "/events");
    };
    html! {
        form class="flex flex-row items-center justify-center" action=(format!("/{}", event_id.0)) {
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
    }
}

async fn ingredients_per_serving(state: State<MyAppState>, meal_id: Path<i32>) -> Markup {
    let Ok(event_meal_ingredints) = state
        .db_connection
        .get_event_recipe_ingredients(meal_id.0)
        .await
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
    let format = |x, unit| html! { td { (&format!("{:.3}{}", x,unit)) } };

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

    if let Ok(result) = state.db_connection.update_event(&event).await {
        (StatusCode::OK, event_form(state, event_id).await).into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
            (format(event_meal.weight.to_f64().unwrap_or_default() /  event_meal.servings as f64 * 100., "g"))
            (format(event_meal.price.0 as f64 / 100. / event_meal.servings as f64, "€"))
            td { button hx-swap="afterend" hx-get=(format!("/events/edit/ingredients-per-serving/{}", event_meal.meal_id)) {"Ingredients per serving"} }
        }
    }
}
