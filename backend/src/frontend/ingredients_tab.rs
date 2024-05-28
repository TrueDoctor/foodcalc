use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum::response::{IntoResponse, Response};
use axum_login::RequireAuthorizationLayer;
use foodlib::{Ingredient, User};
use maud::{html, Markup};
use serde::Deserialize;

use crate::MyAppState;

use crate::frontend::LOGIN_URL;

use super::html_error;

pub(crate) fn ingredients_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", axum::routing::post(add_ingredient))
        .route("/:id", axum::routing::delete(delete))
        .route_layer(RequireAuthorizationLayer::<i64, User>::login_or_redirect(
            Arc::new(LOGIN_URL.into()),
            None,
        ))
        .route("/edit", axum::routing::get(edit_ingredient_form))
        .route("/delete/:id", axum::routing::get(delete_ingredient_form))
        .route("/search", axum::routing::post(search))
        .route("/", axum::routing::get(ingredients_view))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn search(State(state): State<MyAppState>, query: Form<SearchParameters>) -> Markup {
    let query = query.search.to_lowercase();
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
    form: axum::extract::Form<Ingredient>,
) -> Response {
    let ingredient = form.0;
    if ingredient.ingredient_id == -1 {
        match state
            .add_ingredient(ingredient.name, ingredient.energy, ingredient.comment)
            .await
        {
            Err(_) => html_error("Failed to add ingredient", "/ingredients").into_response(),
            Ok(_) => (
                [("HX-Retarget", "#content"), ("HX-Reswap", "innerHTML")],
                ingredients_view(State(state)).await,
            )
                .into_response(),
        }
    } else {
        match state.update_ingredient(&ingredient).await {
            Err(_) => html_error("Failed to update ingredient", "/ingredients"),
            Ok(_) => format_ingredient(&ingredient),
        }
        .into_response()
    }
}

pub async fn ingredients_view(State(state): State<MyAppState>) -> Markup {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    html! {
        div id="ingredients"{
            div class="
                flex flex-row items-center justify-stretch
                mb-2 gap-5 h-10
                w-full
                " {
                input class="grow text h-full" type="search" placeholder="Search for Ingredient" id="search" name="search" autocomplete="off"
                    autofocus="autofocus" hx-post="/ingredients/search" hx-trigger="keyup changed delay:20ms, search"
                    hx-target="#search-results" hx-indicator=".htmx-indicator";

            }
            (add_ingredient_button())
            table {
                thead { tr class="p-2" {
                    th class="w-1/3" { "Name" }
                    th class="w-1/8" { "Energy" }
                    th class="w-1/8" { "Comment" }
                    th {}
                    th {}
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

fn add_ingredient_button() -> Markup {
    html! {
        div class = "grow-0 h-full m-2" hx-target="this" id="ingredient--1" {
            button class="btn btn-primary" hx-get="/ingredients/edit" { "Add Ingredient (+)" }
        }
    }
}

async fn delete(state: State<MyAppState>, id: Path<i32>) -> Markup {
    match state.delete_ingredient(id.0).await {
        Ok(_) => ingredients_view(state).await,
        Err(e) => html_error(&format!("Failed to delete ingredient: {e}"), "/ingredients"),
    }
}

pub async fn edit_ingredient_form(old: Option<Form<Ingredient>>) -> Markup {
    let ingredient = old.unwrap_or(Form(Ingredient {
        ingredient_id: -1,
        ..Default::default()
    }));
    let id = ingredient.ingredient_id;
    html! {
        div class="flex flex-col items-center justify-center w-full" {
            div class="flex gap-2 w-full" {
                input class="text" type="text" name="name" placeholder="Name" value=(ingredient.name) required="required";
                input class="text shrink" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy (kJ/g)" required="required" value=(ingredient.energy);
                input class="text grow" type="text" name="comment" placeholder="Comment" value=(ingredient.comment.as_ref().unwrap_or(&String::new()));
                input class="text" type="hidden" name="ingredient_id" value=(ingredient.ingredient_id);
                button class="btn btn-primary" hx-include=(format!("closest #ingredient-{}", id)) hx-post="/ingredients" hx-target=(format!("#ingredient-{}", id)) hx-swap="outerHTML" { "Submit" }
                button class="btn btn-cancel" hx-get="/ingredients" hx-target="#content" { "Cancel" }
            }
        }
    }
}

async fn delete_ingredient_form(state: State<MyAppState>, id: Path<i32>) -> Markup {
    let Ok(usages) = state.ingredient_usages(id.0).await else {
        return html_error("Failed to get ingredient usages", "/ingredients");
    };
    html! {
        dialog open="true" id="delete" {
            span {(format!("The ingredient with id {} is used in the folowing recipes", id.0))}
            ul {
                @for recipe in usages {
                    li { (recipe.name) }
                }
            }
            button class="btn btn-canel" hx-target="#content" hx-delete=(format!("/ingredients/{}",id.0)) { "Delete" }
        }
    }
}

fn format_ingredient(ingredient: &Ingredient) -> Markup {
    html! {
        tr id=(format!("ingredient-{}", ingredient.ingredient_id)) {
            td { (ingredient.name) }
            td { (ingredient.energy) }
            td class="text-center" { (ingredient.comment.as_ref().unwrap_or(&String::new())) }
            td {
                button class="btn btn-primary" hx-get="/ingredients/edit" hx-target=(format!("#ingredient-{}", ingredient.ingredient_id)) hx-vals=(serde_json::to_string(ingredient).unwrap()) { "Edit" }
            }
            td {
                button class="btn btn-cancel"
                hx-get=(format!("/ingredients/delete/{}", ingredient.ingredient_id))
                hx-swap="beforebegin"
                hx-vals=(serde_json::to_string(ingredient).unwrap()) { "Delete" }
            }
        }
    }
}
