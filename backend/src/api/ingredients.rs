use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::types::BigDecimal;

use crate::MyAppState;

#[derive(Clone, Deserialize)]
pub struct Ingredient {
    name: String,
    energy: BigDecimal,
    comment: Option<String>,
}

pub async fn create(
    State(state): State<MyAppState>,
    Json(ingredient): Json<Ingredient>,
) -> impl IntoResponse {
    let id = state
        .db_connection
        .add_ingredient(ingredient.name, ingredient.energy, ingredient.comment)
        .await;
    match id {
        Ok(id) => (StatusCode::OK, Json(id)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(-1)),
    }
}

pub async fn update(
    State(state): State<MyAppState>,
    Path(ingredient_id): Path<i32>,
    Json(ingredient): Json<Ingredient>,
) -> impl IntoResponse {
    let db = state.db_connection.clone();
    let ingredient = foodlib::Ingredient {
        ingredient_id,
        name: ingredient.name,
        energy: ingredient.energy,
        comment: ingredient.comment,
    };
    db.update_ingredient(ingredient).await.unwrap();

    StatusCode::OK
}

pub async fn list(State(state): State<MyAppState>) -> impl IntoResponse {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();
    Json(ingredients)
}

pub async fn search(State(state): State<MyAppState>, Json(query): Json<String>) -> Markup {
    let query = query.to_lowercase();
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

pub async fn list_html(State(state): State<MyAppState>) -> Markup {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    html! {
        h1 { "Ingredients" }
        input type="search" placeholder="Search" id="search" name="search" autocomplete="off" autofocus="autofocus" hx-post="/search" hx-trigger="keýup changed delay:500ms, search" hx-target="#search-resutls" hx-indicator=".htmx-indicator";
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
        tr id=(format!("ingredient-{}", ingredient.name)) {
            td { (ingredient.name) }
            td { (ingredient.energy) }
            td { (ingredient.comment.clone().unwrap_or_default()) }
        }
    }
}
