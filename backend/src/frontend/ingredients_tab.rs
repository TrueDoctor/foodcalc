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
    let Ok(_) = state
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
        div id="ingredients" class="flex flex-col items-center justify-center mb-16" {
            div  class="w-3/4 flex flex-col items-center justify-center" {
                div class="
                    flex flex-row items-center justify-stretch
                    mb-2 gap-5 h-10
                    w-full
                    " {
                    input class="grow text h-full" type="search" placeholder="Search for Ingredient" id="search" name="search" autocomplete="off"
                        autofocus="autofocus" hx-post="/ingredients/search" hx-trigger="keyup changed delay:20ms, search"
                        hx-target="#search-results" hx-indicator=".htmx-indicator";

                }
                div class = "grow-0 h-full m-2"
                    hx-target="this"  hx-swap="outerHTML" {
                    button class="btn btn-primary" hx-get="/ingredients/add" { "Add Ingredient (+)" }
                }
                table {
                    thead { tr class="p-2" {
                        th class="w-1/2" { "Name" }
                        th class="w-1/4" { "Energy" }
                        th class="w-1/4" { "Comment" }
                    } }
                    tbody id="search-results" {
                        @for ingredient in ingredients.iter() {
                            (format_ingredient(ingredient))
                        }
                    }
                }
            }
        }
    }
}

pub async fn edit_ingredient_form(State(_): State<MyAppState>) -> Markup {
    html! {
        form hx-put="/ingredients/edit" hx-target="#ingredients" hx-swap="outerHTML" class="w-full" {
            div class="flex flex-col items-center justify-center w-full" {
                div class="flex gap-2 w-full" {
                    input class="text" type="text" name="name" placeholder="Name" value="" required="required";
                    input class="text shrink" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy (kJ/g)" required="required";
                    input class="text grow" type="text" name="comment" placeholder="Comment" value="";
                    input class="text" type="hidden" name="ingredient_id" value="-1";
                    button class="btn btn-primary" type="submit" { "Submit" }
                    button class="btn btn-cancel" hx-get="/ingredients" { "Cancel" }
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
