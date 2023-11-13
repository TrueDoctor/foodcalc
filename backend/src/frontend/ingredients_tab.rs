use axum::extract::State;
use maud::{html, Markup};

use crate::MyAppState;

pub(crate) fn ingredients_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/search", axum::routing::post(search))
        .route("/edit", axum::routing::put(add_ingredient))
        .route("/add", axum::routing::get(edit_ingredient_form))
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

pub async fn add_ingredient(
    State(state): State<MyAppState>,
    form: axum::extract::Form<foodlib::Ingredient>,
) -> Markup {
    let ingredient = form.0;
    let Ok(ingredient_id) = state
        .db_connection
        .add_ingredient(ingredient.name, ingredient.energy, ingredient.comment)
        .await
    else {
        return html! {
            div id="error" class="flex flex-col items-center justify-center text-red-500" {
                div {
                    h1 { "Error" }
                    p { "Failed to add ingredient" }
                    button class="btn btn-primary" hx-get="/ingredients" hx-target="#content"  { "Back" }
                }
            }
        };
    };

    ingredients_view(State(state)).await
}

pub async fn ingredients_view(State(state): State<MyAppState>) -> Markup {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    html! {
        div id="ingredients" class="flex flex-col items-center justify-center" {
            div {
                h1 { "Ingredients" }
                input type="search" placeholder="Search for Ingredient" id="search" name="search" autocomplete="off"
                    autofocus="autofocus" hx-post="/ingredients/search" hx-trigger="keyup changed delay:20ms, search"
                    hx-target="#search-results" hx-indicator=".htmx-indicator";
                span class="htmx-indicator" { "Searching..." }
                div hx-target="this"  hx-swap="outerHTML" {
                    button hx-get="/ingredients/add" class="btn btn-primary"  { "+" }
                }
                table class="text-inherit table-auto object-center" {
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
    }
}

pub async fn edit_ingredient_form(State(state): State<MyAppState>) -> Markup {
    html! {
        form hx-put="/ingredients/edit" hx-target="#ingredients" hx-swap="outerHTML" {
            div class="flex flex-col items-center justify-center" {
                div {
                    h1 { "Edit Ingredient" }
                    input type="text" name="name" placeholder="Name" value="" required="required";
                    input type="number" name="energy" placeholder="Energy" value="0" required="required";
                    input type="text" name="comment" placeholder="Comment" value="";
                    input type="hidden" name="ingredient_id" value="-1";
                    button class="btn btn-primary" type="submit" { "Submit" }
                }
            }
        }
    }
}

fn format_ingredient(ingredient: &foodlib::Ingredient) -> Markup {
    html! {
        tr id=(format!("ingredient-{}", ingredient.ingredient_id)) {
            td { (ingredient.name) }
            td { (ingredient.energy) }
            td class="text-center" { (ingredient.comment.clone().unwrap_or_default()) }
        }
    }
}
