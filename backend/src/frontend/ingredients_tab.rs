use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum::response::{IntoResponse, Response};
use axum_login::RequireAuthorizationLayer;
use bigdecimal::BigDecimal;
use foodlib::{Ingredient, IngredientSource, Store, User};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgMoney;

use crate::MyAppState;

use crate::frontend::LOGIN_URL;

use super::html_error;

pub(crate) fn ingredients_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", axum::routing::post(add_ingredient))
        .route(
            "/sources/:ingredient/:source",
            axum::routing::post(update_source),
        )
        .route("/:id", axum::routing::delete(delete))
        .route_layer(RequireAuthorizationLayer::<i64, User>::login_or_redirect(
            Arc::new(LOGIN_URL.into()),
            None,
        ))
        .route("/edit", axum::routing::get(edit_ingredient_form))
        .route("/delete/:id", axum::routing::get(delete_ingredient_form))
        .route("/sources/:id", axum::routing::get(sources_table))
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

#[derive(Deserialize, Clone, PartialEq)]
struct SourceData {
    store_id: i32,
    weight: BigDecimal,
    price: f64,
    url: Option<String>,
    comment: Option<String>,
}

async fn update_source(
    State(state): State<MyAppState>,
    Path((ingredient_id, source_id)): Path<(i32, i32)>,
    form: axum::extract::Form<SourceData>,
) -> Markup {
    let ingredient = form.0;
    if source_id == -1 {
        match state
            .add_ingredient_source(
                ingredient_id,
                ingredient.store_id,
                ingredient.weight,
                PgMoney((ingredient.price * 100.) as i64),
                ingredient.url,
                ingredient.comment,
                0,
            )
            .await
        {
            Err(_) => html_error("Failed to add ingredient source", "/ingredients"),
            Ok(_) => sources_table(State(state), Path(ingredient_id)).await,
        }
    } else {
        let source = IngredientSource {
            ingredient_source_id: source_id,
            ingredient_id,
            store_id: ingredient.store_id,
            package_size: ingredient.weight,
            price: PgMoney((ingredient.price * 100.) as i64),
            unit_id: 0,
            url: ingredient.url,
            comment: ingredient.comment,
        };
        match state.update_ingredient_source(&source).await {
            Err(_) => html_error("Failed to update ingredient source", "/ingredients"),
            Ok(_) => sources_table(State(state), Path(ingredient_id)).await,
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

async fn sources_table(state: State<MyAppState>, id: Path<i32>) -> Markup {
    let Ok(sources) = state.get_ingredient_sources(Some(id.0)).await else {
        return html_error("Failed to fetch ingredient_sources", "/ingredients");
    };

    let Ok(stores) = state.get_stores().await else {
        return html_error("Failed to fetch stores", "/ingredients");
    };

    let Ok(ingredient) = state.get_ingredient(id.0).await else {
        return html_error("Failed to fetch ingredient", "/ingredients");
    };

    let dummy_ingredient = IngredientSource::default();
    html! {
        dialog open="true" class="w-2/3 dialog"  id="popup"{
            h1 {(ingredient.name)}
            table {
                thead {
                    tr {
                        th { "Store" }
                        th { "Weight" }
                        th { "Price" }
                        th { "Url" }
                        th {}
                    }
                }
                tbody {
                    (format_ingredient_source(&dummy_ingredient, &stores, id.0))
                    @for source in sources {
                        (format_ingredient_source(&source, &stores, id.0))
                    }
                }
            }
            button  class="btn btn-cancel" hx-swap="delete" hx-target="#popup" hx-get="/" {"Close"}
        }
    }
}

fn format_ingredient_source(
    source: &IngredientSource,
    stores: &[Store],
    ingredient_id: i32,
) -> Markup {
    let button = |text, id| {
        html! {
            button class="btn btn-primary" hx-target="#popup" hx-swap="outerHTML" hx-post=(format!("/ingredients/sources/{ingredient_id}/{id}")) hx-include="closest tr" { (text) }
        }
    };

    let button = if source.ingredient_source_id == -1 {
        button("Add", -1)
    } else {
        button("Save", source.ingredient_source_id)
    };
    let option = |store: &Store, source_store| match store.store_id == source_store {
        false => html! {
            option
                label=(store.name)
                value=(store.store_id) {
                (store.name)
            }
        },
        true => html! {
            option
                label=(store.name)
                value=(store.store_id)
                selected {(store.name)}
        },
    };
    html! {
        tr {
            td {
                label for="stores" style="display:none" { "Pick a Store" }
                select name="store_id" id="stores" required="true" class="text" {
                    @for store in stores {
                        (option(store, source.store_id))
                    }
                }
            }
            td {input class="text" name="weight" value=(source.package_size);}
            td {input class="text" name="price" type="number" value=(&(source.price.0 as f64 / 100.));}
            td {input class="text" name="url" value=(source.url.clone().unwrap_or_default());}
            td {(button)}
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
                button class="btn btn-primary"
                hx-get="/ingredients/edit"
                hx-target=(format!("#ingredient-{}", ingredient.ingredient_id))
                hx-vals=(serde_json::to_string(ingredient).unwrap()) { "Edit" }
            }
            td {
                button class="btn btn-cancel"
                hx-get=(format!("/ingredients/delete/{}", ingredient.ingredient_id))
                hx-swap="beforebegin" { "Delete" }
            }
            td {
                button class="btn btn-primary"
                hx-get=(format!("/ingredients/sources/{}", ingredient.ingredient_id))
                hx-swap="afterend"
                // hx-target=(format!("#ingredient-{}", ingredient.ingredient_id))
                { "Sources ▼" }
            }
        }
    }
}
