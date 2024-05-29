use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum::routing::{get, post};
use axum_login::RequireAuthorizationLayer;
use foodlib::{Recipe, User};
use maud::{html, Markup};
use serde::Deserialize;

use crate::MyAppState;

use crate::frontend::LOGIN_URL;
mod recipes_edit_tab;

pub(crate) fn recipes_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", post(add_recipe))
        .route("/delete/:recipe_id", get(delete_recipe))
        .route(
            "/delete_nqa/:recipe_id",
            axum::routing::delete(delete_recipe_nqa),
        )
        .nest("/edit/", recipes_edit_tab::recipes_edit_router())
        .route_layer(RequireAuthorizationLayer::<i64, User>::login_or_redirect(
            Arc::new(LOGIN_URL.into()),
            Some(Arc::new("protected".into())),
        ))
        .route("/search", post(search))
        .route("/shopping-list/:recipe_id", post(shopping_list))
        .route("/export/:recipe_id", get(export_recipe))
        .route("/export_pdf/:recipe_id", get(export_recipe_pdf))
        .route("/", get(recipes_view))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn search(State(state): State<MyAppState>, query: Form<SearchParameters>) -> Markup {
    let query = query.search.to_lowercase();
    let Ok(recipes) = state.get_recipes().await else {
        return html_error("Failed to fetch recipes");
    };

    let filtered_recipes = recipes
        .iter()
        .filter(|x| x.name.to_lowercase().contains(&query));

    // (recipe_add_form())
    html! {
        (recipe_add_form())
        @for recipe in filtered_recipes {
            (format_recipe(recipe))
        }
    }
}

pub async fn export_recipe(Path(recipe_id): Path<i32>) -> Markup {
    html! {
         dialog class="dialog" open="open" {
             div class="flex flex-col items-center justify-center" {
                 div class="flex flex-col items-center justify-center" {
                     h1 { "Export recipe" }
                     // Input mask for energy and number of servings as a form which downloads the recipe as a PDF on Submit
                        form class="flex flex-col items-center justify-center" action=(format!("/recipes/export_pdf/{}", recipe_id)) {
                            div class="flex flex-row items-center justify-center" {
                                input class="text" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy kJ/serving" required="required";
                                input class="text" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="number_of_servings" placeholder="Number of servings" required="required";
                                button class="btn btn-primary" type="submit" hx-post=(format!("/recipes/shopping-list/{}", recipe_id)) hx-target="#shopping-list" { "Shopping list" }
                                button class="btn btn-primary" type="submit" { "Export" }
                            }
                        }
                    div id="shopping-list";
                 }
             }
         }
    }
}

#[derive(serde::Deserialize)]
pub struct ExportRecipe {
    energy: f64,
    number_of_servings: u32,
}

pub async fn export_recipe_pdf(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Form(form): Form<ExportRecipe>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>), Markup> {
    let energy = form.energy;
    let number_of_servings = form.number_of_servings;

    let subrecipes = state
        .db_connection
        .fetch_subrecipes_from_user_input(recipe_id, number_of_servings as f64, energy as u32)
        .await
        .unwrap();

    let title = state
        .db_connection
        .get_recipe(recipe_id)
        .await
        .unwrap()
        .name;

    let result = state
        .db_connection
        .generate_recipes_typst(&subrecipes)
        .await;

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
            Err(html_error("Failed to save recipe export"))
        }
    }
}

pub async fn shopping_list(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Form(form): Form<ExportRecipe>,
) -> Markup {
    let energy = form.energy;
    let number_of_servings = form.number_of_servings;

    let subrecipes = state
        .db_connection
        .fetch_subrecipes_from_user_input(recipe_id, number_of_servings as f64, energy as u32)
        .await
        .unwrap();
    let shopping_list = subrecipes
        .iter()
        .filter_map(|recipe| {
            (!recipe.is_subrecipe).then(|| (recipe.ingredient.clone(), recipe.weight.to_string()))
        })
        .collect::<Vec<_>>();

    html! {
        div class="flex flex-col items-center justify-center" {
            h1 { "Shopping list" }
            table class="w-full text-inherit table-auto object-center" {
                thead { tr { th { "Ingredient" } th { "Amount" } th { "Unit" } } }
                tbody {
                    @for (ingredient, amount) in shopping_list {
                        tr {
                            td { (ingredient) }
                            td { (amount) }
                            td { "kg" }
                        }
                    }
                }
            }
        }
    }
}

pub async fn delete_recipe_nqa(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> Markup {
    if let Err(error) = state.delete_recipe(recipe_id).await {
        log::error!("Failed to delete recipe: {}", error);
        return html_error("Failed to delete recipe");
    };

    recipes_view(State(state)).await
}

fn html_error(reason: &str) -> Markup {
    crate::frontend::html_error(reason, "/recipes")
}

pub async fn delete_recipe(Path(recipe_id): Path<i32>) -> Markup {
    html! {
        dialog class="dialog" open="open" {
            div class="flex flex-col items-center justify-center" {
                div class="flex flex-col items-center justify-center" {
                    h1 { "Are you sure you want to delete this recipe?" }
                    div class="flex flex-row items-center justify-center mt-6" {
                        button class="btn btn-success mr-4" hx-target="#content" hx-delete=(format!("/recipes/delete_nqa/{}", recipe_id)) { "Yes" }
                        button class="btn btn-cancel" hx-target="#content" hx-get="/recipes" { "No" }
                    }
                }
            }
        }
    }
}

pub async fn recipes_view(State(state): State<MyAppState>) -> Markup {
    let Ok(recipes) = state.get_recipes().await else {
        return html_error("Failed to fetch recipes");
    };

    html! {
        div id="recipes"  {
            div class="
                flex flex-row items-center justify-stretch
                mb-2 gap-5 h-10
                w-full
                " {
                input class="grow text h-full" type="search" placeholder="Search for recipe" id="search" name="search" autocomplete="off"
                    autofocus="autofocus" hx-post="/recipes/search" hx-trigger="keyup changed delay:100ms, search"
                    hx-target="#search-results" hx-indicator=".htmx-indicator";

            }
            table class="w-full text-inherit table-auto object-center" {
                // We add extra table headers to account for the buttons
                thead { tr { th { "Name" } th { "Energy" } th { "Comment" }  th {} th {} th {} th {}} }
                form hx-post="/recipes" hx-target="#content"  class="w-full" {
                    tbody id="search-results"  {
                        (recipe_add_form())
                        @for recipe in recipes.iter() {
                            (format_recipe(recipe))
                        }
                    }
                }
                span class="htmx-indicator" {
                    "Searching..."
                }
            }
        }
    }
}

fn recipe_add_form() -> Markup {
    html! {
        tr  { td {  }
            td { input class="grow text" type="text" name="name";}
            td { input class="grow text" type="text" name="comment";}
            td { button class="btn btn-primary" type="submit"  { "Add" } }
            td {} td {} td { div id="dialog"; }
        }
    }
}

#[derive(Debug, Deserialize)]
struct NewRecipe {
    name: String,
    comment: Option<String>,
}

async fn add_recipe(state: State<MyAppState>, Form(recipe): Form<NewRecipe>) -> Markup {
    let recipe = Recipe {
        name: recipe.name,
        comment: recipe.comment,
        recipe_id: -1,
    };
    match state.insert_recipe(&recipe).await {
        Ok(recipe) => recipes_edit_tab::recipe_edit_view(state, Path(recipe.recipe_id)).await,
        Err(e) => html_error(&e.to_string()),
    }
}

fn format_recipe(recipe: &foodlib::Recipe) -> Markup {
    html! {
        tr id=(format!("recipe-{}", recipe.recipe_id)) {
            td { (recipe.recipe_id) }
            td { (recipe.name) }
            td class="text-center" { (recipe.comment.clone().unwrap_or_default()) }
            td { button class="btn btn-primary" type="button" hx-target="#content" hx-get=(format!("/recipes/edit/{}", recipe.recipe_id)) { "Edit" } }
            td { button class="btn btn-cancel"  type="button" hx-target="next #dialog" hx-get=(format!("/recipes/delete/{}", recipe.recipe_id)) { "Delete" } }
            td { button class="btn btn-primary" type="button" hx-get=(format!("/recipes/export/{}", recipe.recipe_id)) hx-swap="afterend" { "Export" } }
            td { div id="dialog"; }
        }
    }
}
