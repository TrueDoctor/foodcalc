use crate::FoodLib;
use axum::extract::{Form, Path, Query};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum_login::login_required;
use bigdecimal::BigDecimal;
use foodlib_new::auth::AuthBackend;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::ingredient::IngredientWithSource;
use foodlib_new::user::User;
use foodlib_new::{
    error::Error,
    ingredient::{Ingredient, IngredientSource},
    store::Store,
};
use maud::{html, Markup};
use num::{FromPrimitive, ToPrimitive};
use serde::Deserialize;

use super::{IResponse, MResponse};
use crate::frontend::{move_group, LOGIN_URL};
use crate::MyAppState;

pub(crate) fn ingredients_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", post(update_ingredient))
        .route("/sources/{ingredient}/{source}", post(update_source))
        .route("/{id}", delete(delete_ingredient))
        .route("/edit", get(edit_ingredient_form))
        .route("/move/{id}", get(move_ingredient_dialog))
        .route("/move/{id}", post(move_ingredient))
        .route("/delete/{id}", get(delete_ingredient_form))
        .route(
            "/sources/delete/{ingredient_id}/{source_id}",
            get(delete_source),
        )
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
        .route("/sources/{id}", get(sources_table))
        .route("/", get(ingredients_view))
}

#[derive(Deserialize, Default, Clone)]
pub struct IngredientFilters {
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub mine_only: Option<String>,
}

impl IngredientFilters {
    fn mine_only(&self) -> bool {
        self.mine_only.is_some()
    }
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .map_or(false, |v| v == "true")
}

pub async fn update_ingredient(
    foodlib: FoodLib,
    user: User,
    Form(mut ingredient): Form<Ingredient>,
) -> IResponse {
    if ingredient.id == -1 {
        let group = foodlib.users().get_personal_group(user.id).await?;
        ingredient.group_id = group.id;
        foodlib.ingredients().create(ingredient).await?;
        Ok((
            [("HX-Retarget", "#content"), ("HX-Reswap", "innerHTML")],
            render_ingredients_view(foodlib, Some(user)).await,
        )
            .into_response())
    } else {
        let existing = foodlib.ingredients().get(ingredient.id).await?;
        let user_groups = foodlib.users().get_user_groups(user.id).await?;
        let user_group_ids: Vec<i32> = user_groups.iter().map(|g| g.id).collect();
        if !can_edit(existing.group_id, &user_group_ids, &user) {
            return Err(Error::Forbidden(
                "You don't have permission to edit this ingredient".into(),
            ));
        }

        // Keep the original group_id to prevent ownership transfer via form manipulation
        ingredient.group_id = existing.group_id;
        let updated = foodlib.ingredients().update(ingredient).await?;
        Ok(format_ingredient(&updated.into(), Some(&user), &user_group_ids).into_response())
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
    user: User,
    Path((ingredient_id, source_id)): Path<(i32, i32)>,
    form: Form<SourceData>,
) -> MResponse {
    let ingredient = foodlib.ingredients().get(ingredient_id).await?;
    let user_groups = foodlib.users().get_user_groups(user.id).await?;
    let user_group_ids: Vec<i32> = user_groups.iter().map(|g| g.id).collect();
    if !can_edit(ingredient.group_id, &user_group_ids, &user) {
        return Err(Error::Forbidden(
            "You don't have permission to edit sources for this ingredient".into(),
        ));
    }

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
        sources_table(foodlib, Some(user), Path(ingredient_id)).await
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
        sources_table(foodlib, Some(user), Path(ingredient_id)).await
    }
}

async fn delete_source(
    foodlib: FoodLib,
    user: User,
    Path((ingredient_id, source_id)): Path<(i32, i32)>,
) -> MResponse {
    let ingredient = foodlib.ingredients().get(ingredient_id).await?;
    let user_groups = foodlib.users().get_user_groups(user.id).await?;
    let user_group_ids: Vec<i32> = user_groups.iter().map(|g| g.id).collect();
    if !can_edit(ingredient.group_id, &user_group_ids, &user) {
        return Err(Error::Forbidden(
            "You don't have permission to delete sources for this ingredient".into(),
        ));
    }

    foodlib.ingredients().delete_source(source_id).await?;
    sources_table(foodlib, Some(user), Path(ingredient_id)).await
}

/// Route handler for `GET /ingredients`. Reads filters from the query so the
/// URL is shareable; returns just the filtered `<tbody>` for HX requests
/// (triggered by the filter form below) and the full page otherwise.
pub async fn ingredients_view(
    foodlib: FoodLib,
    user: Option<User>,
    headers: HeaderMap,
    Query(filters): Query<IngredientFilters>,
) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&foodlib, user.as_ref())
        .await
        .unwrap_or_default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);

    if is_htmx(&headers) {
        ingredient_rows(&filtered, user.as_ref(), &user_group_ids)
    } else {
        render_ingredients_page(&filtered, user.as_ref(), &user_group_ids, &filters)
    }
}

/// Render the full ingredients page. Used both by the route handler (on
/// non-HX requests) and by mutation handlers that want to return the user
/// to a fresh page; mutations don't preserve filter state today.
pub async fn render_ingredients_view(foodlib: FoodLib, user: Option<User>) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&foodlib, user.as_ref())
        .await
        .unwrap_or_default();
    let filters = IngredientFilters::default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);
    render_ingredients_page(&filtered, user.as_ref(), &user_group_ids, &filters)
}

fn filter_ingredients<'a>(
    ingredients: &'a [IngredientWithSource],
    filters: &IngredientFilters,
    user_group_ids: &[i32],
) -> Vec<&'a IngredientWithSource> {
    let needle = filters.search.to_lowercase();
    let mine_only = filters.mine_only();
    ingredients
        .iter()
        .filter(|x| {
            x.name.to_lowercase().contains(&needle)
                && (!mine_only || user_group_ids.contains(&x.group_id))
        })
        .collect()
}

fn ingredient_rows(
    ingredients: &[&IngredientWithSource],
    user: Option<&User>,
    user_group_ids: &[i32],
) -> Markup {
    html! {
        @for ingredient in ingredients {
            (format_ingredient(ingredient, user, user_group_ids))
        }
    }
}

fn render_ingredients_page(
    ingredients: &[&IngredientWithSource],
    user: Option<&User>,
    user_group_ids: &[i32],
    filters: &IngredientFilters,
) -> Markup {
    let search_value = filters.search.clone();
    let mine_checked = filters.mine_only();
    html! {
        div id="ingredients"{
            // Filter form is a GET targeting this same page. Each input change
            // re-issues GET /ingredients?... with hx-replace-url so the URL
            // bar tracks the filter state without polluting history.
            form id="ingredients-filter" class="
                flex flex-row items-center justify-stretch
                mb-2 gap-5 h-10
                w-full
                "
                hx-get="/ingredients"
                hx-trigger="keyup changed delay:20ms from:#search, change from:#mine-only, search"
                hx-target="#search-results"
                hx-replace-url="true"
                hx-indicator=".htmx-indicator" {
                input class="grow text h-full" type="search" placeholder="Search for Ingredient" id="search" name="search" autocomplete="off" autofocus="autofocus" value=(search_value);
                @if user.is_some() {
                    label class="flex items-center gap-2 whitespace-nowrap" {
                        input type="checkbox" id="mine-only" name="mine_only" value="1" checked[mine_checked];
                        "Mine only"
                    }
                }
            }
            (add_ingredient_button(user))

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
                    (ingredient_rows(ingredients, user, user_group_ids))
                }
            }
        }
    }
}

fn add_ingredient_button(user: Option<&User>) -> Markup {
    if user.is_none() {
        return html! {};
    }

    let ingredient: IngredientWithSource = Ingredient {
        id: -1,
        name: String::new(),
        energy: BigDecimal::from(0),
        comment: None,
        group_id: -1,
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
    // Only admins can delete ingredients
    if !user.is_admin {
        return Err(foodlib_new::Error::Forbidden(
            "You don't have permission to delete ingredients".into(),
        ));
    }
    foodlib.ingredients().delete(id.0).await?;
    Ok(render_ingredients_view(foodlib, Some(user)).await)
}

pub async fn edit_ingredient_form(Form(ingredient): Form<IngredientWithSource>) -> Markup {
    let id = ingredient.id;
    html! {
        td colspan="6" {
            div class="flex flex-col items-center justify-center w-full" {
                div class="flex gap-2 w-full" {

                    input type="hidden" name=("group_id") value=(ingredient.group_id);
                    input class="text" type="text" name="name" placeholder="Name" value=(ingredient.name) required="required";
                    input class="text shrink" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy (kJ/g)" required="required" value=(ingredient.energy);
                    input class="text grow" type="text" name="comment" placeholder="Comment" value=(ingredient.comment.as_ref().unwrap_or(&String::new()));
                    input class="text" type="hidden" name="id" value=(ingredient.id);
                    button class="btn btn-primary" hx-include=(format!("closest #ingredient-{}", id)) hx-post="/ingredients" hx-target=(format!("#ingredient-{}", id)) hx-swap="outerHTML" { "Submit" }
                    @if id > 0 {
                        button class="btn btn-primary" type="button"
                            hx-get=(format!("/ingredients/move/{id}"))
                            hx-target="body"
                            hx-swap="beforeend" { "Move" }
                    }
                    button class="btn btn-cancel" hx-get="/ingredients" hx-target="#content" { "Cancel" }
                }
            }
        }
    }
}

async fn delete_ingredient_form(foodlib: FoodLib, user: User, id: Path<i32>) -> MResponse {
    // Only show delete form if user is an admin
    if !user.is_admin {
        return Err(foodlib_new::Error::Forbidden(
            "You don't have permission to delete ingredients".into(),
        ));
    }

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

async fn move_ingredient_dialog(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> MResponse {
    ctx.assert_can_edit_ingredient(id).await?;
    let ingredient = foodlib.ingredients().get(id).await?;
    let panel = move_group::move_panel(
        &foodlib,
        &ctx,
        ingredient.group_id,
        &format!("/ingredients/move/{id}"),
        "#content",
    )
    .await?;
    Ok(html! {
        dialog open="true" class="dialog fixed top-1/4 left-1/2 -translate-x-1/2 z-50 shadow-xl" id="move-dialog"
            hx-on::after-request="if(event.detail.successful) this.remove()" {
            div class="flex flex-col m-2 gap-2 min-w-80" {
                p class="text-lg font-semibold" { "Move \"" (ingredient.name) "\"" }
                (panel)
                button class="btn btn-abort" hx-on:click="document.getElementById('move-dialog').remove()" { "Cancel" }
            }
        }
    })
}

#[derive(Deserialize)]
struct MoveIngredientForm {
    group_id: i32,
}

async fn move_ingredient(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Form(form): Form<MoveIngredientForm>,
) -> MResponse {
    ctx.assert_can_edit_ingredient(id).await?;
    move_group::assert_can_move_to(&ctx, form.group_id)?;
    foodlib.ingredients().set_group(id, form.group_id).await?;
    Ok(render_ingredients_view(foodlib, Some(ctx.user)).await)
}

async fn sources_table(foodlib: FoodLib, user: Option<User>, id: Path<i32>) -> MResponse {
    let sources = foodlib.ingredients().get_sources(id.0).await?;
    let stores = foodlib.stores().list().await?;
    let ingredient = foodlib.ingredients().get(id.0).await?;

    let user_group_ids: Vec<i32> = if let Some(u) = user.as_ref() {
        foodlib
            .users()
            .get_user_groups(u.id)
            .await
            .unwrap_or_default()
            .iter()
            .map(|g| g.id)
            .collect()
    } else {
        vec![]
    };
    let can_edit_sources = user
        .as_ref()
        .map_or(false, |u| can_edit(ingredient.group_id, &user_group_ids, u));

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
                    @if can_edit_sources {
                        (format_add_ingredient_source(&stores, id.0))
                    }
                    @for source in sources {
                        (format_ingredient_source(&source, &stores, id.0, can_edit_sources))
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
    can_edit: bool,
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
                select name="store_id" id="stores" required="true" class="text w-full" disabled[!can_edit] {
                    @for store in stores {
                        (option(store, source.store_id))
                    }
                }
            }
            td {input class="text" name="weight" value=(source.package_size) disabled[!can_edit];}
            td {input class="text" name="price" value=(source.price.to_f64().unwrap()) disabled[!can_edit];}
            td {input class="text" name="url" value=(source.url.clone().unwrap_or_default()) disabled[!can_edit];}
            @if can_edit {
                td {button class="btn btn-primary" hx-post=(format!("/ingredients/sources/{}/{}", ingredient_id, source.id)) hx-include="closest tr" hx-target="#popup" hx-swap="outerHTML" { "Update" }}
                td {button class="btn btn-cancel" hx-get=(format!("/ingredients/sources/delete/{}/{}", ingredient_id, source.id)) hx-target="#popup" hx-swap="outerHTML" { "Delete" }}
            } @else {
                td {}
                td {}
            }
        }
    }
}

fn can_edit(group_id: i32, user_group_ids: &[i32], user: &User) -> bool {
    user.is_admin || user_group_ids.contains(&group_id)
}

async fn fetch_ingredients_and_groups(
    foodlib: &FoodLib,
    user: Option<&User>,
) -> Result<(Vec<IngredientWithSource>, Vec<i32>), foodlib_new::Error> {
    let ingredients = foodlib.ingredients().list_with_sources().await?;
    let user_group_ids = match user {
        Some(u) => foodlib
            .users()
            .get_user_groups(u.id)
            .await?
            .into_iter()
            .map(|g| g.id)
            .collect(),
        None => vec![],
    };
    Ok((ingredients, user_group_ids))
}

fn format_ingredient(ingredient: &IngredientWithSource, user: Option<&User>, user_group_ids: &[i32]) -> Markup {
    let sources_button_text = if ingredient.has_sources { "" } else { "⚠️" };
    let can_edit_this = user.map_or(false, |u| can_edit(ingredient.group_id, user_group_ids, u));
    let is_admin = user.map_or(false, |u| u.is_admin);

    html! {
        tr id=(format!("ingredient-{}", ingredient.id)) {
            td class="text-center opacity-70" { (ingredient.id) }
            td class="text-center" { (ingredient.name) }
            td class="text-center" { (ingredient.energy) }
            td class="text-center" { (ingredient.comment.as_ref().unwrap_or(&String::new())) }
            td {
                @if can_edit_this {
                    button class="btn btn-primary"
                    hx-get="/ingredients/edit"
                    hx-target=(format!("#ingredient-{}", ingredient.id))
                    hx-vals=(serde_json::to_string(ingredient).unwrap()) { "Edit" }
                }
            }
            td {
                @if is_admin {
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
