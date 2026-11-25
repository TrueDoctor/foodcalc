use axum::extract::{Form, Path, State};
use axum::routing::{delete, get, post};
use axum_login::login_required;
use foodlib_new::entities::recipe::Recipe;
use foodlib_new::types::ids::{GroupId, RecipeId};
use foodlib_new::{AppCtx, AppState};
use maud::{html, Markup};
use serde::Deserialize;

use super::web_error::WebError;
use super::MResponse;
use crate::auth::AuthBackend;
use crate::frontend::{icons, OptWebCtx, WebCtx, LOGIN_URL};

mod recipes_edit_tab;

pub(crate) fn recipes_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(add_recipe))
        .route("/delete/{recipe_id}", get(delete_recipe))
        .route("/delete_nqa/{recipe_id}", delete(delete_recipe_nqa))
        .nest("/edit/", recipes_edit_tab::recipes_edit_router())
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
        .route("/search", post(search))
        .route("/shopping-list/{recipe_id}", post(shopping_list))
        .route("/export/{recipe_id}", get(export_recipe))
        .route("/export_pdf/{recipe_id}", get(export_recipe_pdf))
        .route("/allergens/{recipe_id}", get(allergens_dialog))
        .route("/", get(recipes_view))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
    #[serde(default)]
    mine_only: Option<String>,
}

pub async fn search(
    State(state): State<AppState>,
    OptWebCtx(actor): OptWebCtx,
    query: Form<SearchParameters>,
) -> MResponse {
    let needle = query.search.to_lowercase();
    let mine_only = query.mine_only.is_some();
    let (recipes, user_group_ids) =
        foodlib_new::recipes::gather_overview(&state, actor.as_ref()).await?;

    let filtered_recipes = recipes.iter().filter(|x| {
        x.name.to_lowercase().contains(&needle)
            && (!mine_only || user_group_ids.contains(&x.group_id))
    });

    Ok(html! {
        @if let Some(ref a) = actor {
            (recipe_add_row(a.user.is_admin))
        }
        @for recipe in filtered_recipes {
            (format_recipe(recipe, actor.as_ref(), &user_group_ids))
        }
    })
}

pub async fn export_recipe(Path(recipe_id): Path<i32>) -> Markup {
    html! {
         dialog class="dialog" id="popup" open="open" {
             div class="flex flex-col items-center justify-center p-2" {
                 div class="flex flex-col items-center justify-center gap-2" {
                     h1 class="text-lg mb-2" { "Export recipe" }
                     form class="flex flex-col items-stretch justify-center gap-4 w-full" action=(format!("/recipes/export_pdf/{}", recipe_id)) {
                         div class="flex flex-col gap-2 w-full" {
                             input class="text w-full" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy kJ/serving" required="required";
                             input class="text w-full" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="number_of_servings" placeholder="Number of servings" required="required";
                         }
                         div class="flex flex-row flex-wrap items-center justify-center gap-4" {
                             button class="btn btn-primary" type="submit" hx-post=(format!("/recipes/shopping-list/{}", recipe_id)) hx-target="#shopping-list" { "Shopping list" }
                             button class="btn btn-primary" type="submit" { "Export" }
                             button class="btn btn-cancel" hx-swap="delete" hx-target="#popup" hx-get="/" {"Close"}
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
    State(state): State<AppState>,
    Path(recipe_id): Path<i32>,
    Form(form): Form<ExportRecipe>,
) -> std::result::Result<([(axum::http::HeaderName, String); 2], Vec<u8>), WebError> {
    let recipe_info = foodlib_new::exports::fetch_user_input_meal(
        &state,
        RecipeId(recipe_id),
        form.number_of_servings as f64,
        form.energy as u32,
    )
        .await?;
    let filename = format!("{}.pdf", recipe_info.name);
    let exported = foodlib_new::exports::render_pdf(&state, filename, move |pdf| {
        pdf.export_recipes(vec![recipe_info])
    })
        .await?;

    let headers = [
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", exported.filename),
        ),
        (
            axum::http::header::CONTENT_TYPE,
            exported.mime.to_string(),
        ),
    ];
    Ok((headers, exported.bytes))
}

pub async fn shopping_list(
    State(state): State<AppState>,
    Path(recipe_id): Path<i32>,
    Form(form): Form<ExportRecipe>,
) -> MResponse {
    let subrecipes = foodlib_new::exports::fetch_subrecipes_from_user_input(
        &state,
        RecipeId(recipe_id),
        form.number_of_servings as f64,
        form.energy as u32,
    )
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

pub async fn delete_recipe_nqa(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(recipe_id): Path<i32>,
) -> MResponse {
    foodlib_new::recipes::delete_recipe(&state, &actor, RecipeId(recipe_id)).await?;
    Ok(recipes_view(State(state), OptWebCtx(Some(actor))).await)
}

pub async fn delete_recipe(Path(recipe_id): Path<i32>, WebCtx(actor): WebCtx) -> Markup {
    // Only show delete dialog if user is admin
    if !actor.user.is_admin {
        return html! {};
    }

    html! {
        dialog class="dialog" open="open" id="dialog" {
            div class="flex flex-col items-center justify-center" {
                p { "Are you sure you want to delete this Recipe permanently?" }
                div class="flex justify-between w-full m-2 gap-2" {
                    button class="btn btn-abort" hx-on:click="document.getElementById('dialog').remove()" { "Abort" }
                    button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/recipes/delete_nqa/{}", recipe_id)) { "Confirm Delete" }
                }
            }
        }
    }
}

pub async fn recipes_view(State(state): State<AppState>, OptWebCtx(actor): OptWebCtx) -> Markup {
    let (recipes, user_group_ids) =
        foodlib_new::recipes::gather_overview(&state, actor.as_ref())
            .await
            .unwrap_or_default();

    html! {
        div id="recipes" class="w-full" {
            form id="recipes-filter" class="
                flex flex-row items-center justify-stretch
                mb-2 gap-5 h-10
                w-full
                "
                hx-post="/recipes/search"
                hx-trigger="keyup changed delay:100ms from:#search, change from:#mine-only, search"
                hx-target="#search-results"
                hx-indicator=".htmx-indicator" {
                input class="grow text h-full" type="search" placeholder="Search for recipe" id="search" name="search" autocomplete="off" autofocus="autofocus";
                @if actor.is_some() {
                    label class="flex items-center gap-2 whitespace-nowrap" {
                        input type="checkbox" id="mine-only" name="mine_only" value="1";
                        "Mine only"
                    }
                }
            }
            table class="w-full text-inherit table-auto object-center responsive-card" {
                @let is_admin = actor.as_ref().is_some_and(|a| a.user.is_admin);
                thead { tr {
                    @if is_admin { th class="w-1/12" { "ID" } }
                    th { "Name" } th { "Comment" } th {} th {}
                } }
                tbody id="search-results" {
                    @if actor.is_some() {
                        (recipe_add_row(is_admin))
                    }
                    @for recipe in recipes.iter() {
                        (format_recipe(recipe, actor.as_ref(), &user_group_ids))
                    }
                }
                span class="htmx-indicator" {
                    "Searching..."
                }
            }
        }
    }
}

fn recipe_add_row(is_admin: bool) -> Markup {
    html! {
        tr id="recipe--1" {
            input type="hidden" name="id" value="-1";
            input type="hidden" name="group_id" value="-1";
            @if is_admin {
                td class="text-center opacity-70 no-label" { "+" }
            }
            td data-label="Name" { input class="grow text" type="text" name="name" placeholder="Recipe name" required="required"; }
            td data-label="Comment" { input class="grow text" type="text" name="comment" placeholder="Comment"; }
            td class="no-label" {
                button class="btn btn-primary"
                    hx-post="/recipes"
                    hx-include="closest tr"
                    hx-target="#content" { "Add" }
            }
            td class="no-label" {}
        }
    }
}

#[derive(Debug, Deserialize)]
struct NewRecipe {
    name: String,
    comment: Option<String>,
}

async fn add_recipe(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Form(recipe_data): Form<NewRecipe>,
) -> MResponse {
    let created_recipe = foodlib_new::recipes::create_recipe(
        &state,
        &actor,
        recipe_data.name,
        recipe_data.comment,
    )
        .await?;
    recipes_edit_tab::recipe_edit_view(State(state), WebCtx(actor), Path(created_recipe)).await
}

fn format_recipe(recipe: &Recipe, actor: Option<&AppCtx>, _user_group_ids: &[GroupId]) -> Markup {
    let can_edit_this = actor.is_some_and(|a| a.can_access_group(recipe.group_id));
    let is_admin = actor.is_some_and(|a| a.user.is_admin);
    let comment = recipe.comment.as_deref().unwrap_or("").trim().to_string();

    html! {
        tr id=(format!("recipe-{}", recipe.id)) {
            @if is_admin {
                td class="text-center opacity-70" data-label="ID" { (recipe.id) }
            }
            td data-label="Name" {
                span class="inline-flex items-center gap-1" {
                    (recipe.name)
                    button class="opacity-60 hover:opacity-100 cursor-pointer text-blue-600 dark:text-blue-400"
                        hx-get=(format!("/recipes/allergens/{}", recipe.id))
                        hx-target="#content"
                        hx-swap="afterbegin"
                        title="View allergens"
                        type="button"
                        style="background: none; border: none; padding: 0; font-size: 0.85em;"
                        { "ⓘ" }
                }
            }
            td data-label=(if comment.is_empty() { "" } else { "Comment" })
               class=(if comment.is_empty() { "no-label" } else { "" }) {
                (comment)
            }
            td class="no-label whitespace-nowrap" {
                div class="flex flex-row flex-nowrap gap-2 justify-end items-center" {
                    @if can_edit_this {
                        button class="icon-btn" type="button" title="Edit" aria-label="Edit" hx-push-url="true" hx-target="#content" hx-get=(format!("/recipes/edit/{}", recipe.id)) { (icons::edit()) }
                    }
                    @if is_admin {
                        button class="icon-btn" type="button" title="Delete" aria-label="Delete" hx-swap="beforebegin" hx-get=(format!("/recipes/delete/{}", recipe.id)) { (icons::delete()) }
                    }
                    button class="btn btn-primary" type="button" hx-get=(format!("/recipes/export/{}", recipe.id)) hx-swap="afterend" { "Export" }
                }
            }
            td class="no-label" { div id="dialog"; }
        }
    }
}

fn pretty_property(name: &str) -> String {
    let stripped = name.strip_prefix("~spuren-").unwrap_or(name);
    let mut chars = stripped.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

async fn allergens_dialog(
    State(state): State<AppState>,
    Path(recipe_id): Path<i32>,
) -> MResponse {
    let foodlib_new::recipes::RecipeAllergenData {
        recipe_name,
        dietary,
        contains,
        may_contain,
        hinweise,
    } = foodlib_new::recipes::gather_recipe_allergens(&state, RecipeId(recipe_id)).await?;

    Ok(html! {
        dialog open="true" class="dialog z-50" id="popup" {
            div class="flex justify-between items-center w-full mb-3 gap-2" {
                p class="text-xl font-semibold" { "Allergens: " (recipe_name) }
                button class="btn btn-cancel" hx-swap="delete" hx-target="#popup" hx-get="/" { "Close" }
            }

            // Dietary flag badges. Same visual idiom as the PDF. A label that
            // holds only because may-contain traces were ignored gets a trailing
            // asterisk, explained by the footnote below.
            div class="flex flex-wrap gap-2 mb-4" {
                @if dietary.vegan {
                    span class="px-2 py-1 rounded-full text-sm font-semibold bg-green-100 text-green-800 border border-green-300" { "vegan" (if dietary.vegan_trace { "*" } else { "" }) }
                }
                @if dietary.vegetarian {
                    span class="px-2 py-1 rounded-full text-sm font-semibold bg-lime-100 text-lime-800 border border-lime-300" { "vegetarisch" (if dietary.vegetarian_trace { "*" } else { "" }) }
                }
                @if dietary.lactose_free {
                    span class="px-2 py-1 rounded-full text-sm font-semibold bg-blue-100 text-blue-800 border border-blue-300" { "laktosefrei" (if dietary.lactose_free_trace { "*" } else { "" }) }
                }
                @if dietary.gluten_free {
                    span class="px-2 py-1 rounded-full text-sm font-semibold bg-purple-100 text-purple-800 border border-purple-300" { "glutenfrei" (if dietary.gluten_free_trace { "*" } else { "" }) }
                }
                @if !dietary.vegetarian && !dietary.lactose_free && !dietary.gluten_free && !dietary.vegan {
                    span class="text-sm opacity-70" { "No dietary flags apply." }
                }
            }
            @if dietary.vegan_trace || dietary.vegetarian_trace || dietary.lactose_free_trace || dietary.gluten_free_trace {
                p class="text-xs italic opacity-70 mb-3" {
                    "* vegan / vegetarisch / laktosefrei / glutenfrei ohne Berücksichtigung von Spuren "
                    "(„kann Spuren enthalten“-Hinweise werden für diese Kennzeichnung ignoriert)."
                }
            }

            @if contains.is_empty() && may_contain.is_empty() {
                p class="text-center opacity-70 py-4" { "Keine deklarationspflichtigen Allergene." }
            } @else {
                @if !contains.is_empty() {
                    div class="mb-4" {
                        p class="font-semibold mb-2" { "Enthält" }
                        p { (contains.iter().map(|s| pretty_property(s)).collect::<Vec<_>>().join(", ")) }
                    }
                }
                @if !may_contain.is_empty() {
                    div class="mb-4" {
                        p class="font-semibold mb-2" { "Kann Spuren enthalten" }
                        p class="text-sm italic opacity-80" {
                            (may_contain.iter().map(|s| pretty_property(s)).collect::<Vec<_>>().join(", "))
                        }
                    }
                }
            }

            // Non-EU-14 informational avoidances (onions, mushrooms, ...). Kept
            // separate from the regulatory disclosure above; matches common guest
            // dietary requests that aren't legally-declarable allergens.
            @if !hinweise.is_empty() {
                div class="mb-4 pt-3 border-t border-gray-300 dark:border-gray-600" {
                    p class="font-semibold mb-2" { "Hinweise (keine EU-Allergene)" }
                    p class="text-sm opacity-80" {
                        (hinweise.iter().map(|s| pretty_property(s)).collect::<Vec<_>>().join(", "))
                    }
                }
            }

            p class="text-xs opacity-60 mt-3" {
                "Aggregated from all ingredients in this recipe (including subrecipes). "
                "To remove specific tags, edit the source ingredients."
            }
        }
    })
}