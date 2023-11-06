use axum::extract::{BodyStream, Json, State};
use futures_util::StreamExt;
use maud::{html, Markup};

use crate::MyAppState;

pub(crate) fn ingredients_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/search", axum::routing::post(search))
        .route("/", axum::routing::get(ingredients_view))
}

pub async fn search(State(state): State<MyAppState>, query: String) -> Markup {
    let query = query.replace("search=", "").to_lowercase();
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    let filtered_ingredients = ingredients
        .iter()
        .filter(|x| x.name.to_lowercase().contains(&query));

    html! {
        @for ingredient in filtered_ingredients {
            (format_ingredient(ingredient))
        }
    }
}

pub async fn ingredients_view(State(state): State<MyAppState>) -> Markup {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    html! {
        h1 { "Ingredients" }
        input type="search" placeholder="Search" id="search" name="search" autocomplete="off"
            autofocus="autofocus" hx-post="/ingredients/search" hx-trigger="keyup changed delay:20ms, search"
            hx-target="#search-results" hx-indicator=".htmx-indicator";
        span class="htmx-indicator" { "Searching..." }
        table {
            thead { tr { th { "Name" } th { "Energy" } th { "Comment" } } }
            tbody id="search-results" {
                @for ingredient in ingredients.iter() {
                    (format_ingredient(ingredient))
                }
            }
        }
        // Add Ingredient button
    }
}

fn format_ingredient(ingredient: &foodlib::Ingredient) -> Markup {
    html! {
        tr id=(format!("ingredient-{}", ingredient.ingredient_id)) {
            td { (ingredient.name) }
            td { (ingredient.energy) }
            td { (ingredient.comment.clone().unwrap_or_default()) }
        }
    }
}
