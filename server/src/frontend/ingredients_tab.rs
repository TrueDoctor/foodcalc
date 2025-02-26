use crate::FoodLib;
use axum::extract::{Form, Path};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum_login::login_required;
use bigdecimal::BigDecimal;
use foodlib_new::auth::AuthBackend;
use foodlib_new::ingredient::IngredientWithSource;
use foodlib_new::user::User;
use foodlib_new::{
    ingredient::{Ingredient, IngredientSource},
    store::Store,
};
use maud::{html, Markup};
use num::{FromPrimitive, ToPrimitive};
use serde::Deserialize;

use super::{IResponse, MResponse};
use crate::frontend::LOGIN_URL;
use crate::MyAppState;

pub(crate) fn ingredients_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", post(update_ingredient))
        .route("/sources/{ingredient}/{source}", post(update_source))
        .route("/{id}", delete(delete_ingredient))
        .route("/edit", get(edit_ingredient_form))
        .route("/delete/{id}", get(delete_ingredient_form))
        .route(
            "/sources/delete/{ingredient_id}/{source_id}",
            get(delete_source),
        )
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
        .route("/sources/{id}", get(sources_table))
        .route("/search", post(search))
        .route("/", get(ingredients_view))
}

#[derive(Deserialize)]
pub struct SearchParameters {
    search: String,
}

pub async fn search(
    foodlib: FoodLib,
    user: Option<User>,
    query: Form<SearchParameters>,
) -> MResponse {
    let query = query.search.to_lowercase();
    let ingredients = foodlib.ingredients().list_with_sources().await?;

    let filtered_ingredients = ingredients
        .iter()
        .filter(|x| x.name.to_lowercase().contains(&query));

    Ok(html! {
        @for ingredient in filtered_ingredients {
            (format_ingredient(ingredient, user.as_ref()))
        }
    })
}

pub async fn update_ingredient(
    foodlib: FoodLib,
    user: User,
    Form(mut ingredient): Form<Ingredient>,
) -> IResponse {
    if ingredient.id == -1 {
        ingredient.owner_id = user.id;
        foodlib.ingredients().create(ingredient).await?;
        Ok((
            [("HX-Retarget", "#content"), ("HX-Reswap", "innerHTML")],
            ingredients_view(foodlib, Some(user)).await,
        )
            .into_response())
    } else {
        let updated = foodlib.ingredients().update(ingredient).await?;
        Ok(format_ingredient(&updated.into(), Some(&user)).into_response())
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
    foodlib: FoodLib,
    Path((ingredient_id, source_id)): Path<(i32, i32)>,
    form: Form<SourceData>,
) -> MResponse {
    let data = form.0;

    if source_id == -1 {
        // Creating a new source
        let source = IngredientSource {
            id: -1,
            ingredient_id,
            store_id: data.store_id,
            package_size: data.weight,
            price: BigDecimal::from_f64(data.price).unwrap(),
            unit_id: 0,
            url: data.url,
            comment: data.comment,
        };

        foodlib.ingredients().add_source(source).await?;
        sources_table(foodlib, Path(ingredient_id)).await
    } else {
        // Updating existing source
        let source = IngredientSource {
            id: source_id,
            ingredient_id,
            store_id: data.store_id,
            package_size: data.weight,
            price: BigDecimal::from_f64(data.price).unwrap(),
            unit_id: 0,
            url: data.url,
            comment: data.comment,
        };

        foodlib.ingredients().update_source(source).await?;
        sources_table(foodlib, Path(ingredient_id)).await
    }
}

async fn delete_source(
    foodlib: FoodLib,
    Path((ingredient_id, source_id)): Path<(i32, i32)>,
) -> MResponse {
    foodlib.ingredients().delete_source(source_id).await?;
    sources_table(foodlib, Path(ingredient_id)).await
}

pub async fn ingredients_view(foodlib: FoodLib, user: Option<User>) -> Markup {
    let ingredients = foodlib
        .ingredients()
        .list_with_sources()
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
            (add_ingredient_button(user.as_ref()))

            span class="htmx-indicator" { "Searching..." }

            table class="table-fixed"{
                thead { tr class="p-2" {
                    th class="w-1/12" { "ID" }
                    th class="w-1/4" { "Name" }
                    th class="w-1/8" { "Energy" }
                    th class="w-1/8" { "Comment" }
                    th {}
                    th {}
                    th class="w-1/6" {}
                } }
                tbody id="search-results" {
                    @for ingredient in ingredients.iter() {
                        (format_ingredient(ingredient, user.as_ref()))
                    }
                }
            }
        }
    }
}

fn add_ingredient_button(user: Option<&User>) -> Markup {
    let ingredient: IngredientWithSource = Ingredient {
        id: -1,
        name: String::new(),
        energy: BigDecimal::from(0),
        comment: None,
        owner_id: user.map(|x| x.id).unwrap_or(-1),
    }
    .into();
    html! {
        div class = "m-2" hx-target="this" id="ingredient--1" {
            button class="btn bg-light-primary-normal dark:bg-dark-primary-normal"
            hx-get="/ingredients/edit"
            hx-vals=(serde_json::to_string(&ingredient).unwrap())
            hx-swap="innerHTML" { "Add Ingredient (+)" }
        }
    }
}

async fn delete_ingredient(foodlib: FoodLib, user: User, id: Path<i32>) -> MResponse {
    if !user.is_admin {
        return Err(foodlib_new::Error::Forbidden(
            "You don't have permission to delete ingredients".into(),
        ));
    }
    foodlib.ingredients().delete(id.0).await?;
    Ok(ingredients_view(foodlib, Some(user)).await)
}

pub async fn edit_ingredient_form(Form(ingredient): Form<IngredientWithSource>) -> Markup {
    let id = ingredient.id;
    html! {
        td colspan="6" {
            div class="flex flex-col items-center justify-center w-full" {
                div class="flex gap-2 w-full" {

                    input type="hidden" name=("owner_id") value=(ingredient.owner_id);
                    input class="text" type="text" name="name" placeholder="Name" value=(ingredient.name) required="required";
                    input class="text shrink" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy (kJ/g)" required="required" value=(ingredient.energy);
                    input class="text grow" type="text" name="comment" placeholder="Comment" value=(ingredient.comment.as_ref().unwrap_or(&String::new()));
                    input class="text" type="hidden" name="id" value=(ingredient.id);
                    button class="btn btn-primary" hx-include=(format!("closest #ingredient-{}", id)) hx-post="/ingredients" hx-target=(format!("#ingredient-{}", id)) hx-swap="outerHTML" { "Submit" }
                    button class="btn btn-cancel" hx-get="/ingredients" hx-target="#content" { "Cancel" }
                }
            }
        }
    }
}

async fn delete_ingredient_form(foodlib: FoodLib, id: Path<i32>) -> MResponse {
    let usages = foodlib.ingredients().usages(id.0).await?;

    Ok(html! {
        dialog open="true" class="dialog" id="delete" {
            span {(format!("The ingredient with id {} is used in the following {} recipes", id.0, usages.len()))}
            ul class="list-disc mx-4 m-2" {
                @for recipe in usages {
                    li { (recipe.name) }
                }
            }
            div class="flex justify-between w-full m-2 gap-2" {
                button class="btn btn-abort" hx-get="/ingredients" hx-target="#content" { "Abort" }
                button class="btn btn-cancel mx-4" hx-target="#content" hx-delete=(format!("/ingredients/{}",id.0)) { "Confirm Delete" }
            }
        }
    })
}

async fn sources_table(foodlib: FoodLib, id: Path<i32>) -> MResponse {
    let sources = foodlib.ingredients().get_sources(id.0).await?;
    let stores = foodlib.stores().list().await?;
    let ingredient = foodlib.ingredients().get(id.0).await?;

    Ok(html! {
        dialog open="true" class="w-2/3 dialog z-50" id="popup"{
            div class="flex justify-between w-full mb-2" {
                div class="flex gap-2" {}
                p class="text-2xl" { (format!("Sources for {}", ingredient.name)) }
                button class="btn bg-btn-cancel-normal" hx-swap="delete" hx-target="#popup" hx-get="/" {"Close"}
            }
            table class="table-fixed" {
                thead {
                    tr {
                        th { "Store" }
                        th { "Weight (kg)" }
                        th { "Price (€)" }
                        th { "Url" }
                        th {}
                        th {}
                    }
                }
                tbody {
                    (format_add_ingredient_source(&stores, id.0))
                    @for source in sources {
                        (format_ingredient_source(&source, &stores, id.0))
                    }
                }
            }
        }
    })
}

fn format_add_ingredient_source(stores: &[Store], ingredient_id: i32) -> Markup {
    html! {
        tr {
            td {
                label for="stores" style="display:none" { "Pick a Store" }
                select name="store_id" id="stores" required="true" class="text w-full" {
                    option label="Pick a Store" value="" { "Pick a Store" }
                    @for store in stores {
                        (html! {
                            option
                                label=(store.name)
                                value=(store.id) {
                                (store.name)
                            }
                        })
                    }
                }
            }
            td {input class="text" name="weight" placeholder="Weight (kg)";}
            td {input class="text" name="price" placeholder="Price (€)";}
            td {input class="text" name="url" placeholder="Url";}
            td {button class="btn btn-primary" hx-post=(format!("/ingredients/sources/{}/-1", ingredient_id)) hx-include="closest tr" hx-target="#popup" hx-swap="outerHTML" { "Add" }}
            td {}
        }
    }
}

fn format_ingredient_source(
    source: &IngredientSource,
    stores: &[Store],
    ingredient_id: i32,
) -> Markup {
    let option = |store: &Store, source_store| match store.id == source_store {
        false => html! {
            option
                label=(store.name)
                value=(store.id) {
                (store.name)
            }
        },
        true => html! {
            option
                label=(store.name)
                value=(store.id)
                selected {(store.name)}
        },
    };
    html! {
        tr {
            td {
                label for="stores" style="display:none" { "Pick a Store" }
                select name="store_id" id="stores" required="true" class="text w-full" {
                    @for store in stores {
                        (option(store, source.store_id))
                    }
                }
            }
            td {input class="text" name="weight" value=(source.package_size);}
            td {input class="text" name="price" value=(source.price.to_f64().unwrap());}
            td {input class="text" name="url" value=(source.url.clone().unwrap_or_default());}
            td {button class="btn btn-primary" hx-post=(format!("/ingredients/sources/{}/{}", ingredient_id, source.id)) hx-include="closest tr" hx-target="#popup" hx-swap="outerHTML" { "Update" }}
            td {button class="btn btn-cancel" hx-get=(format!("/ingredients/sources/delete/{}/{}", ingredient_id, source.id)) hx-target="#popup" hx-swap="outerHTML" { "Delete" }}
        }
    }
}

fn format_ingredient(ingredient: &IngredientWithSource, user: Option<&User>) -> Markup {
    let sources_button_text = if ingredient.has_sources { "" } else { "⚠️" };
    let can_edit = |id: i64| user.is_some_and(|x| x.is_admin || x.id == id);
    html! {
        tr id=(format!("ingredient-{}", ingredient.id)) {
            td class="text-center opacity-70" { (ingredient.id) }
            td class="text-center" { (ingredient.name) }
            td class="text-center" { (ingredient.energy) }
            td class="text-center" { (ingredient.comment.as_ref().unwrap_or(&String::new())) }
            td {
                @if can_edit(ingredient.owner_id) {
                    button class="btn btn-primary"
                    hx-get="/ingredients/edit"
                    hx-target=(format!("#ingredient-{}", ingredient.id))
                    hx-vals=(serde_json::to_string(ingredient).unwrap()) { "Edit" }
                }
            }
            td {
                @if user.is_some_and(|x|x.is_admin) {
                    button class="btn btn-cancel"
                    hx-get=(format!("/ingredients/delete/{}", ingredient.id))
                    hx-swap="beforebegin" { "Delete" }
                }
            }
            td {
                button class="btn btn-primary relative"
                hx-get=(format!("/ingredients/sources/{}", ingredient.id))
                hx-swap="afterend"
                {
                    span class="absolute left-0 transform translate-x-6" { (sources_button_text) }
                    span class="block" { "Sources ▼" }
                }
            }
        }
    }
}
