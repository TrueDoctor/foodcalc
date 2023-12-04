use axum::extract::State;
use maud::{html, Markup};

use crate::MyAppState;

pub(crate) fn recipes_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/search", axum::routing::post(search))
        .route("/add", axum::routing::get(edit_recipe_form))
        .route("/", axum::routing::get(recipes_view))
}

pub async fn search(State(state): State<MyAppState>, query: String) -> Markup {
    let query = query.replace("search=", "").to_lowercase();
    let recipes = state.db_connection.get_recipes().await.unwrap_or_default();

    let filtered_recipes = recipes
        .iter()
        .filter(|x| x.name.to_lowercase().contains(&query));

    html! {
        @for recipe in filtered_recipes {
            (format_recipe(recipe))
        }
    }
}

pub async fn recipes_view(State(state): State<MyAppState>) -> Markup {
    let recipes = state.db_connection.get_recipes().await.unwrap_or_default();

    html! {
        div id="recipes" class="flex flex-col items-center justify-center" {
            div  class="w-3/4 flex flex-col items-center justify-center" {
                div class="
                    flex flex-row items-center justify-stretch
                    mb-2 gap-5 h-10
                    w-full
                    " {
                    input class="grow text h-full" type="search" placeholder="Search for recipe" id="search" name="search" autocomplete="off"
                        autofocus="autofocus" hx-post="/recipes/search" hx-trigger="keyup changed delay:20ms, search"
                        hx-target="#search-results" hx-indicator=".htmx-indicator";

                }
                div class = "grow-0 h-full m-2"
                    hx-target="this"  hx-swap="outerHTML" {
                    button class="btn btn-primary" hx-get="/recipes/add" { "Add recipe (+)" }
                }
                table class="w-full text-inherit table-auto object-center" {
                    thead { tr { th { "Name" } th { "Energy" } th { "Comment" } } }
                    tbody id="search-results" {
                        @for recipe in recipes.iter() {
                            (format_recipe(recipe))
                        }
                    }
                }
            }
        }
    }
}

pub async fn edit_recipe_form(State(_): State<MyAppState>) -> Markup {
    html! {
        form hx-put="/recipes/edit" hx-target="#recipes" hx-swap="outerHTML" class="w-full" {
            div class="flex flex-col items-center justify-center w-full" {
                div class="flex gap-2 w-full" {
                    input class="text" type="text" name="name" placeholder="Name" value="" required="required";
                    input class="text shrink" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy (kJ/g)" required="required";
                    input class="text grow" type="text" name="comment" placeholder="Comment" value="";
                    input class="text" type="hidden" name="recipe_id" value="-1";
                    button class="btn btn-primary" type="submit" { "Submit" }
                    button class="btn btn-cancel" hx-get="/recipes" { "Cancel" }
                }
            }
        }
    }
}

fn format_recipe(recipe: &foodlib::Recipe) -> Markup {
    html! {
        tr id=(format!("recipe-{}", recipe.recipe_id)) {
            td { (recipe.recipe_id) }
            td { (recipe.name) }
            td class="text-center" { (recipe.comment.clone().unwrap_or_default()) }
        }
    }
}
