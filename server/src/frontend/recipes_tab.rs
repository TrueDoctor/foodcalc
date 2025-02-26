use crate::FoodLib;
use axum::extract::{Form, Path, State};
use axum::routing::{delete, get, post};
use axum_login::login_required;
use foodlib_new::auth::AuthBackend;
use foodlib_new::recipe::Recipe;
use foodlib_new::user::User;
use maud::{html, Markup};
use serde::Deserialize;

use super::MResponse;
use crate::frontend::LOGIN_URL;
use crate::MyAppState;

mod recipes_edit_tab;

pub(crate) fn recipes_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", post(add_recipe))
        .route("/delete/{recipe_id}", get(delete_recipe))
        .route("/delete_nqa/{recipe_id}", delete(delete_recipe_nqa))
        .nest("/edit/", recipes_edit_tab::recipes_edit_router())
        .route_layer(login_required!(
            AuthBackend,
            login_url = LOGIN_URL,
            redirect_field = "protected"
        ))
        .route("/search", post(search))
        .route("/shopping-list/{recipe_id}", post(shopping_list))
        .route("/export/{recipe_id}", get(export_recipe))
        .route("/export_pdf/{recipe_id}", get(export_recipe_pdf))
        .route("/", get(recipes_view))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn search(foodlib: FoodLib, query: Form<SearchParameters>) -> MResponse {
    let query = query.search.to_lowercase();
    let recipes = foodlib.recipes().list().await?;

    let filtered_recipes = recipes
        .iter()
        .filter(|x| x.name.to_lowercase().contains(&query));

    Ok(html! {
        (recipe_add_form())
        @for recipe in filtered_recipes {
            (format_recipe(recipe))
        }
    })
}

pub async fn export_recipe(Path(recipe_id): Path<i32>) -> Markup {
    html! {
         dialog class="dialog" open="open" {
             div class="flex flex-col items-center justify-center p-2" {
                 div class="flex flex-col items-center justify-center gap-2" {
                     h1 class="text-lg mb-2" { "Export recipe" }
                     // Input mask for energy and number of servings as a form which downloads the recipe as a PDF on Submit
                        form class="flex flex-col items-center justify-center gap-x-8" action=(format!("/recipes/export_pdf/{}", recipe_id)) {
                            div class="flex flex-row items-center justify-center gap-4" {
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

#[derive(Deserialize)]
pub struct ExportRecipe {
    energy: f64,
    number_of_servings: u32,
}

pub async fn export_recipe_pdf(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Form(form): Form<ExportRecipe>,
) -> Result<([(axum::http::HeaderName, String); 2], Vec<u8>), foodlib_new::Error> {
    let energy = form.energy;
    let number_of_servings = form.number_of_servings;

    let recipe_info = state
        .fetch_user_input_meal(
            recipe_id,
            number_of_servings as f64,
            energy as u32,
            "".to_string(),
        )
        .await?;
    let title = recipe_info.name.to_owned();
    #[cfg(feature = "typst")]
    let result = foodlib::typst::export_recipes(recipe_info).await;
    #[cfg(not(feature = "typst"))]
    let result = Err(foodlib_new::Error::Misc(
        "Server compiled without typst support".into(),
    ));
    let recipe = result?;

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

pub async fn shopping_list(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
    Form(form): Form<ExportRecipe>,
) -> MResponse {
    let energy = form.energy;
    let number_of_servings = form.number_of_servings;

    let subrecipes = state
        .fetch_subrecipes_from_user_input(recipe_id, number_of_servings as f64, energy as u32)
        .await?;
    let shopping_list = subrecipes
        .iter()
        .filter(|&recipe| !recipe.is_subrecipe)
        .map(|recipe| (recipe.ingredient.clone(), recipe.weight.to_string()))
        .collect::<Vec<_>>();

    Ok(html! {
        div class="flex flex-col items-center justify-center" {
            h1 { "Shopping list" }
            table class="w-full text-inherit table-auto object-center table-fixed" {
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
    })
}

pub async fn delete_recipe_nqa(foodlib: FoodLib, Path(recipe_id): Path<i32>) -> MResponse {
    foodlib.recipes().delete(recipe_id).await?;
    Ok(recipes_view(foodlib).await)
}

pub async fn delete_recipe(Path(recipe_id): Path<i32>) -> Markup {
    html! {
        dialog class="dialog" open="open" id="dialog" {
            div class="flex flex-col items-center justify-center" {
                p { "Are you sure you want to delete this Recipe permanently?" }
                div class="flex justify-between w-full m-2 gap-2" {
                    button class="btn btn-abort" hx-on:click="document.getElementById('dialog').remove()" { "Abort" }
                    button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/recipes/delete_nqa/{}",recipe_id)) { "Confirm Delete" }
                }
            }
        }
    }
}

pub async fn recipes_view(foodlib: FoodLib) -> Markup {
    let recipes = foodlib.recipes().list().await.unwrap_or_default();

    html! {
        div id="recipes" class="w-full"  {
            div class="
                flex flex-row items-center justify-stretch
                mb-2 gap-5 h-10
                w-full
                " {
                input class="grow text h-full" type="search" placeholder="Search for recipe" id="search" name="search" autocomplete="off"
                    autofocus="autofocus" hx-post="/recipes/search" hx-trigger="keyup changed delay:100ms, search"
                    hx-target="#search-results" hx-indicator=".htmx-indicator";

            }
            table class="w-full text-inherit table-auto object-center table-fixed" {
                // We add extra table headers to account for the buttons
                thead { tr { th { "ID" } th { "Name" } th { "Comment" }  th {} th {} th {}} }
                    tbody id="search-results"  {
                        (recipe_add_form())
                        @for recipe in recipes.iter() {
                            (format_recipe(recipe))
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
        tr id="add"  { td {  }
            td { input class="grow text" type="text" name="name" placeholder="Recipe name" required="required"; }
            td { input class="grow text" type="text" name="comment" placeholder="Comment"; }
            td { button class="btn btn-primary" hx-include="[name='name'],[name='comment']" hx-target="#content" hx-post="/recipes" { "Add" } }
            td {} td {} td { div id="dialog"; }
        }
    }
}

#[derive(Debug, Deserialize)]
struct NewRecipe {
    name: String,
    comment: Option<String>,
}

async fn add_recipe(foodlib: FoodLib, user: User, Form(recipe_data): Form<NewRecipe>) -> MResponse {
    let recipe = Recipe {
        id: -1,
        name: recipe_data.name,
        comment: recipe_data.comment,
        owner_id: user.id,
    };

    let created_recipe = foodlib.recipes().create(recipe).await?;
    recipes_edit_tab::recipe_edit_view(foodlib, Path(created_recipe.id)).await
}

fn format_recipe(recipe: &Recipe) -> Markup {
    html! {
        tr id=(format!("recipe-{}", recipe.id)) {
            td { (recipe.id) }
            td { (recipe.name) }
            td class="text-center" { (recipe.comment.clone().unwrap_or_default()) }
            td { button class="btn btn-primary" type="button" hx-target="#content" hx-get=(format!("/recipes/edit/{}", recipe.id)) { "Edit" } }
            td { button class="btn btn-cancel"  type="button" hx-swap="beforebegin" hx-get=(format!("/recipes/delete/{}", recipe.id)) { "Delete" } }
            td { button class="btn btn-primary" type="button" hx-get=(format!("/recipes/export/{}", recipe.id)) hx-swap="afterend" { "Export" } }
            td { div id="dialog"; }
        }
    }
}
