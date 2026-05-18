use crate::FoodLib;
use axum::extract::{Form, Path, Query};
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
        .route("/delete/{id}", get(delete_ingredient_form))
        .route(
            "/sources/delete/{ingredient_id}/{source_id}",
            get(delete_source),
        )
        .route(
            "/properties/{ingredient_id}/{property_id}",
            delete(delete_property),
        )
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
        .route("/sources/{id}", get(sources_table))
        .route("/properties/{id}", get(properties_dialog))
        .route("/", get(ingredients_view))
        .route("/rows", get(ingredient_rows_view))
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

pub async fn update_ingredient(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(mut ingredient): Form<Ingredient>,
) -> IResponse {
    if ingredient.id == -1 {
        let group = foodlib.users().get_personal_group(ctx.user.id).await?;
        ingredient.group_id = group.id;
        foodlib.ingredients().create(ingredient).await?;
        Ok((
            [("HX-Retarget", "#content"), ("HX-Reswap", "innerHTML")],
            render_ingredients_view(foodlib, Some(ctx.user)).await,
        )
            .into_response())
    } else {
        let existing = foodlib.ingredients().get(ingredient.id).await?;
        if !can_edit(existing.group_id, &ctx.group_ids, &ctx.user) {
            return Err(Error::Forbidden(
                "You don't have permission to edit this ingredient".into(),
            ));
        }

        let target_group = ingredient.group_id;
        ingredient.group_id = existing.group_id;
        if existing.group_id != target_group {
            move_group::assert_can_move_to(&ctx, target_group)?;
        }
        let updated = foodlib.ingredients().update(ingredient).await?;
        if existing.group_id != target_group {
            foodlib.ingredients().set_group(updated.id, target_group).await?;
        }
        let refreshed = foodlib.ingredients().get(updated.id).await?;
        Ok(format_ingredient(&refreshed.into(), Some(&ctx.user), &ctx.group_ids).into_response())
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

/// Route handler for `GET /ingredients`. Always returns the full page shell;
/// the filter form refreshes only the rows via `GET /ingredients/rows`. URL
/// is shareable — query params drive the initial filter state on full loads.
pub async fn ingredients_view(
    foodlib: FoodLib,
    user: Option<User>,
    Query(filters): Query<IngredientFilters>,
) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&foodlib, user.as_ref())
        .await
        .unwrap_or_default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);
    render_ingredients_page(&filtered, user.as_ref(), &user_group_ids, &filters)
}

/// Returns just the filtered `<tr>` rows for the filter form to swap into
/// `#search-results`. Same query-param shape as `ingredients_view`.
pub async fn ingredient_rows_view(
    foodlib: FoodLib,
    user: Option<User>,
    Query(filters): Query<IngredientFilters>,
) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&foodlib, user.as_ref())
        .await
        .unwrap_or_default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);
    ingredient_rows(&filtered, user.as_ref(), &user_group_ids)
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
                hx-get="/ingredients/rows"
                hx-trigger="keyup changed delay:20ms from:#search, change from:#mine-only, search"
                hx-target="#search-results"
                hx-on::after-request="if(event.detail.successful){const fd=new FormData(this);const qs=new URLSearchParams();for(const [k,v] of fd){if(v) qs.set(k,v);}const q=qs.toString();history.replaceState(null,'','/ingredients'+(q?'?'+q:''));}"
                hx-indicator=".htmx-indicator" {
                input class="grow text h-full" type="search" placeholder="Search for Ingredient" id="search" name="search" autocomplete="off" autofocus="autofocus" value=(search_value);
                @if user.is_some() {
                    label class="flex items-center gap-2 whitespace-nowrap" {
                        input type="checkbox" id="mine-only" name="mine_only" value="1" checked[mine_checked];
                        "Mine only"
                    }
                }
            }
            span class="htmx-indicator" { "Searching..." }

            table class="table-fixed responsive-card"{
                @let is_admin = user.map_or(false, |u| u.is_admin);
                thead { tr class="p-2" {
                    @if is_admin { th class="w-1/12" { "ID" } }
                    th class="w-1/4" { "Name" }
                    th class="w-1/8" { "Energy (kJ/g)" }
                    th class="w-1/8" { "Comment" }
                    th {}
                    th {}
                    th class="w-1/6" {}
                    th class="w-1/6" {}
                } }
                tbody id="search-results" {
                    @if user.is_some() {
                        (ingredient_add_row())
                    }
                    (ingredient_rows(ingredients, user, user_group_ids))
                }
            }
        }
    }
}

/// First-row inline add. Rendered when the user is logged in; the server
/// picks the user's personal group on submit, so no group field needed here.
/// After a successful POST the entire page is re-rendered (server retargets
/// to #content) which restores a fresh empty add-row, and the after-request
/// handler refocuses the name input so the user can keep adding.
fn ingredient_add_row() -> Markup {
    html! {
        tr id="ingredient--1" {
            input type="hidden" name="id" value="-1";
            input type="hidden" name="group_id" value="-1";
            td class="text-center opacity-70 no-label" { "+" }
            td data-label="Name" { input class="text" type="text" name="name" placeholder="Name" required="required"; }
            td data-label="Energy (kJ/g)" { input class="text" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="kJ/g" required="required"; }
            td data-label="Comment" { input class="text" type="text" name="comment" placeholder="Comment"; }
            td class="no-label" {
                button class="btn btn-primary"
                    hx-post="/ingredients"
                    hx-include="closest tr"
                    hx-target="#ingredient--1"
                    hx-swap="outerHTML"
                    hx-on::after-request="if(event.detail.successful){const i=document.querySelector('#ingredient--1 input[name=name]');if(i)i.focus();}"
                    { "Add" }
            }
            td class="no-label" {}
            td class="no-label" {}
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

pub async fn edit_ingredient_form(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(ingredient): Form<IngredientWithSource>,
) -> MResponse {
    let id = ingredient.id;
    let owner_select = if id > 0 {
        Some(move_group::owner_select(&foodlib, &ctx, ingredient.group_id).await?)
    } else {
        None
    };
    Ok(html! {
        td colspan="6" {
            div class="flex flex-col items-center justify-center w-full" {
                div class="flex flex-col sm:flex-row gap-2 w-full sm:items-center" {
                    @if let Some(owner_select) = owner_select {
                        (owner_select)
                    } @else {
                        input type="hidden" name="group_id" value=(ingredient.group_id);
                    }
                    input class="text" type="text" name="name" placeholder="Name" value=(ingredient.name) required="required";
                    input class="text shrink" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="Energy (kJ/g)" required="required" value=(ingredient.energy);
                    input class="text grow" type="text" name="comment" placeholder="Comment" value=(ingredient.comment.as_ref().unwrap_or(&String::new()));
                    input class="text" type="hidden" name="id" value=(ingredient.id);
                    button class="btn btn-primary" hx-include=(format!("closest #ingredient-{}", id)) hx-post="/ingredients" hx-target=(format!("#ingredient-{}", id)) hx-swap="outerHTML" { "Submit" }
                    button class="btn btn-cancel" hx-get="/ingredients" hx-target="#content" { "Cancel" }
                }
            }
        }
    })
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
        dialog open="true" class="dialog z-50" id="popup" {
            div class="flex justify-between items-center w-full mb-3 gap-2" {
                p class="text-xl font-semibold" { "Sources: " (ingredient.name) }
                button class="btn btn-cancel" hx-swap="delete" hx-target="#popup" hx-get="/" { "Close" }
            }
            table class="w-full responsive-card" {
                thead {
                    tr {
                        th { "Store" }
                        th { "Weight (kg)" }
                        th { "Price (€)" }
                        th { "URL" }
                        th class="no-label" {}
                        th class="no-label" {}
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
            td data-label="Store" {
                label for="stores" style="display:none" { "Pick a Store" }
                select name="store_id" id="stores" required="true" class="text w-full" {
                    option value="" { "Pick a Store" }
                    @for store in stores {
                        option value=(store.id) { (store.name) }
                    }
                }
            }
            td data-label="Weight (kg)" { input class="text" name="weight" placeholder="Weight (kg)"; }
            td data-label="Price (€)" { input class="text" name="price" placeholder="Price (€)"; }
            td data-label="URL" { input class="text" name="url" placeholder="URL"; }
            td class="no-label" { button class="btn btn-primary" hx-post=(format!("/ingredients/sources/{}/-1", ingredient_id)) hx-include="closest tr" hx-target="#popup" hx-swap="outerHTML" { "Add" } }
            td class="no-label" {}
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
            td data-label="Store" {
                label for="stores" style="display:none" { "Pick a Store" }
                select name="store_id" id="stores" required="true" class="text w-full" disabled[!can_edit] {
                    @for store in stores {
                        (option(store, source.store_id))
                    }
                }
            }
            td data-label="Weight (kg)" { input class="text" name="weight" value=(source.package_size) disabled[!can_edit]; }
            td data-label="Price (€)" { input class="text" name="price" value=(source.price.to_f64().unwrap()) disabled[!can_edit]; }
            td data-label="URL" { input class="text" name="url" value=(source.url.clone().unwrap_or_default()) disabled[!can_edit]; }
            @if can_edit {
                td class="no-label" { button class="btn btn-primary" hx-post=(format!("/ingredients/sources/{}/{}", ingredient_id, source.id)) hx-include="closest tr" hx-target="#popup" hx-swap="outerHTML" { "Update" } }
                td class="no-label" { button class="btn btn-cancel" hx-get=(format!("/ingredients/sources/delete/{}/{}", ingredient_id, source.id)) hx-target="#popup" hx-swap="outerHTML" { "Delete" } }
            } @else {
                td class="no-label" {}
                td class="no-label" {}
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

    let comment = ingredient.comment.as_deref().unwrap_or("").trim().to_string();

    html! {
        tr id=(format!("ingredient-{}", ingredient.id)) {
            @if is_admin {
                td class="text-center opacity-70" data-label="ID" { (ingredient.id) }
            }
            td class="text-center" data-label="Name" { (ingredient.name) }
            td class="text-center" data-label="Energy (kJ/g)" { (ingredient.energy) }
            td class=(if comment.is_empty() { "text-center no-label" } else { "text-center" })
               data-label=(if comment.is_empty() { "" } else { "Comment" }) {
                (comment)
            }
            td class="no-label" {
                @if can_edit_this {
                    button class="btn btn-primary"
                    hx-get="/ingredients/edit"
                    hx-target=(format!("#ingredient-{}", ingredient.id))
                    hx-vals=(serde_json::to_string(ingredient).unwrap()) { "Edit" }
                }
            }
            td class="no-label" {
                @if is_admin {
                    button class="btn btn-cancel"
                    hx-get=(format!("/ingredients/delete/{}", ingredient.id))
                    hx-target="#content"
                    hx-swap="afterbegin" { "Delete" }
                }
            }
            td class="no-label" {
                button class="btn btn-primary relative"
                hx-get=(format!("/ingredients/sources/{}", ingredient.id))
                hx-target="#content"
                hx-swap="afterbegin"
                {
                    span class="absolute left-0 transform translate-x-6" { (sources_button_text) }
                    span class="block" { "Sources ▼" }
                }
            }
            td class="no-label" {
                button class="btn btn-primary"
                hx-get=(format!("/ingredients/properties/{}", ingredient.id))
                hx-target="#content"
                hx-swap="afterbegin"
                { "Allergens ▼" }
            }
        }
    }
}
