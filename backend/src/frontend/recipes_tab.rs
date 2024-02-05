use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum_login::RequireAuthorizationLayer;
use foodlib::User;
use maud::{html, Markup};
use serde::Deserialize;

use crate::MyAppState;

mod recipes_edit_tab;

use crate::frontend::LOGIN_URL;

pub(crate) fn recipes_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/add", axum::routing::get(edit_recipe_form))
        .route("/delete/:recipe_id", axum::routing::get(delete_recipe))
        .route(
            "/delete_nqa/:recipe_id",
            axum::routing::delete(delete_recipe_nqa),
        )
        .route_layer(RequireAuthorizationLayer::<i64, User>::login_or_redirect(
            Arc::new(LOGIN_URL.into()),
            None,
        ))
        .route("/search", axum::routing::post(search))
        .route("/export/:recipe_id", axum::routing::get(export_recipe))
        .route(
            "/export_pdf/:recipe_id/",
            axum::routing::get(export_recipe_pdf),
        )
        .route("/", axum::routing::get(recipes_view))
        .nest("/edit/", recipes_edit_tab::recipes_edit_router())
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn search(State(state): State<MyAppState>, query: Form<SearchParameters>) -> Markup {
    let query = query.search.to_lowercase();
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

pub async fn export_recipe(Path(recipe_id): Path<i32>) -> Markup {
    html! {
         dialog class="dialog" open="open" {
             div class="flex flex-col items-center justify-center" {
                 div class="flex flex-col items-center justify-center" {
                     h1 { "Export recipe" }
                     // Input mask for energy and number of servings as a form which downloads the recipe as a PDF on Submit
                        form class="flex flex-col items-center justify-center" action=(format!("/recipes/export_pdf/{}/", recipe_id)) {
                            div class="flex flex-row items-center justify-center" {
                                input class="text" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy kJ/serving" required="required";
                                input class="text" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="number_of_servings" placeholder="Number of servings" required="required";
                                button class="btn btn-primary" type="submit" { "Export" }
                            }
                        }

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

    let latex = state
        .db_connection
        .format_recipe_latex_from_user_input(recipe_id, number_of_servings as f64, energy as u32)
        .await
        .unwrap();

    let title = state
        .db_connection
        .get_recipe(recipe_id)
        .await
        .unwrap()
        .name;

    let result = foodlib::compile_pdf(latex).await;

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
            Err(html! {
                div id="error" class="flex flex-col items-center justify-center text-red-500" {
                    div {
                        h1 { "Error" }
                        p { "Failed to save recipe export" }
                        p { (error) }
                        button class="btn btn-primary" hx-get="/recipes" hx-target="#content"  { "Back" }
                    }
                }
            })
        }
    }
}

pub async fn delete_recipe_nqa(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> Markup {
    if let Err(error) = state.db_connection.delete_recipe(recipe_id).await {
        log::error!("Failed to delete recipe: {}", error);
        return html! {
            div id="error" class="flex flex-col items-center justify-center text-red-500" {
                div {
                    h1 { "Error" }
                    p { "Failed to delete recipe" }
                    p { (error) }
                    button class="btn btn-primary" hx-get="/recipes" hx-target="#content"  { "Back" }
                }
            }
        };
    };

    recipes_view(State(state)).await
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
    let recipes = state.db_connection.get_recipes().await.unwrap_or_default();

    html! {
        div id="recipes" class="flex flex-col items-center justify-center mb-16" {
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
                    // We add extra table headers to account for the buttons
                    thead { tr { th { "Name" } th { "Energy" } th { "Comment" }  th {} th {} th {} th {}} }
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
            td { button class="btn btn-primary" hx-target="#recipes" hx-get=(format!("/recipes/edit/{}", recipe.recipe_id)) { "Edit" } }
            td { button class="btn btn-cancel" hx-target="next #dialog" hx-get=(format!("/recipes/delete/{}", recipe.recipe_id)) { "Delete" } }
            td { button class="btn btn-primary" hx-get=(format!("/recipes/export/{}", recipe.recipe_id)) hx-swap="afterend" { "Export" } }
            td { div id="dialog"; }
        }
    }
}
