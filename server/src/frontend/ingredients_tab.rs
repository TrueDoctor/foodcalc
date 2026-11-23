use axum::extract::{Form, Path, Query};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum_login::login_required;
use bigdecimal::BigDecimal;
use foodlib_new::auth::AuthBackend;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{
    GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId,
};
use foodlib_new::entities::types::money::Money;
use foodlib_new::ingredient::IngredientWithSource;
use crate::frontend::WebCtx;
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
use crate::frontend::{icons, move_group, LOGIN_URL};
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
        .route("/properties/{id}", post(add_property))
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
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Form(ingredient): Form<Ingredient>,
) -> IResponse {
    let result = if !ingredient.id.is_valid() {
        app::ingredients::create_ingredient(
            &state,
            &actor,
            ingredient.name.clone(),
            ingredient.energy,
            ingredient.comment.clone(),
        )
        .await?;
        Ok((
            [("HX-Retarget", "#content"), ("HX-Reswap", "innerHTML")],
            render_ingredients_view(State(state), Some(actor.ctx.user.clone())).await,
        )
            .into_response())
    } else {
        let target_group = ingredient.group_id;
        let refreshed =
            app::ingredients::update_ingredient(&state, &actor, ingredient, target_group).await?;
        Ok(format_ingredient(
            &refreshed.into(),
            Some(&actor.ctx.user),
            &actor.ctx.group_ids,
        )
        .into_response())
    };
    result
}

#[derive(Deserialize, Clone, PartialEq)]
struct SourceData {
    store_id: StoreId,
    weight: BigDecimal,
    price: f64,
    url: Option<String>,
    comment: Option<String>,
}

async fn update_source(
    State(state): State<MyAppState>,
    user: User,
    Path((ingredient_id, source_id)): Path<(IngredientId, IngredientSourceId)>,
    form: Form<SourceData>,
) -> MResponse {
    let data = form.0;
    app::ingredients::save_source(
        &state,
        &user,
        ingredient_id,
        source_id,
        data.store_id,
        Amount::from(data.weight),
        Money::from_f64(data.price).unwrap(),
        data.url,
        data.comment,
    )
    .await?;
    sources_table(State(state), Some(user), Path(ingredient_id)).await
}

async fn delete_source(
    State(state): State<MyAppState>,
    user: User,
    Path((ingredient_id, source_id)): Path<(IngredientId, IngredientSourceId)>,
) -> MResponse {
    app::ingredients::delete_source(&state, &user, ingredient_id, source_id).await?;
    sources_table(State(state), Some(user), Path(ingredient_id)).await
}

/// Route handler for `GET /ingredients`. Always returns the full page shell;
/// the filter form refreshes only the rows via `GET /ingredients/rows`. URL
/// is shareable — query params drive the initial filter state on full loads.
pub async fn ingredients_view(
    State(state): State<MyAppState>,
    user: Option<User>,
    Query(filters): Query<IngredientFilters>,
) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&state, user.as_ref())
        .await
        .unwrap_or_default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);
    render_ingredients_page(&filtered, user.as_ref(), &user_group_ids, &filters)
}

/// Returns just the filtered `<tr>` rows for the filter form to swap into
/// `#search-results`. Same query-param shape as `ingredients_view`.
pub async fn ingredient_rows_view(
    State(state): State<MyAppState>,
    user: Option<User>,
    Query(filters): Query<IngredientFilters>,
) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&state, user.as_ref())
        .await
        .unwrap_or_default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);
    ingredient_rows(&filtered, user.as_ref(), &user_group_ids)
}

/// Render the full ingredients page. Used both by the route handler (on
/// non-HX requests) and by mutation handlers that want to return the user
/// to a fresh page; mutations don't preserve filter state today.
pub async fn render_ingredients_view(
    State(state): State<MyAppState>,
    user: Option<User>,
) -> Markup {
    let (ingredients, user_group_ids) = fetch_ingredients_and_groups(&state, user.as_ref())
        .await
        .unwrap_or_default();
    let filters = IngredientFilters::default();
    let filtered = filter_ingredients(&ingredients, &filters, &user_group_ids);
    render_ingredients_page(&filtered, user.as_ref(), &user_group_ids, &filters)
}

fn filter_ingredients<'a>(
    ingredients: &'a [IngredientWithSource],
    filters: &IngredientFilters,
    user_group_ids: &[GroupId],
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
    user_group_ids: &[GroupId],
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
    user_group_ids: &[GroupId],
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
                @let is_admin = user.is_some_and(|u| u.is_admin);
                thead { tr class="p-2" {
                    @if is_admin { th class="w-1/12" { "ID" } }
                    th class="w-1/4" { "Name" }
                    th class="w-1/8" { "Energy (kJ/g)" }
                    th class="w-1/8" { "Comment" }
                    th {}
                    th class="w-1/6" {}
                } }
                tbody id="search-results" {
                    @if user.is_some() {
                        (ingredient_add_row(user, user_group_ids))
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
fn ingredient_add_row(user: Option<&User>, _user_group_ids: &[GroupId]) -> Markup {
    let is_admin = user.is_some_and(|u| u.is_admin);
    html! {
        tr id="ingredient--1" {
            input type="hidden" name="id" value="-1";
            input type="hidden" name="group_id" value="-1";
            @if is_admin {
                td class="text-center opacity-70 no-label" { "+" }
            }
            td data-label="Name" { input class="text" type="text" name="name" placeholder="Name" required="required"; }
            td data-label="Energy (kJ/g)" { input class="text" inputmode="numeric" pattern="\\d*(\\.\\d+)?" name="energy" placeholder="kJ/g" required="required"; }
            td data-label="Comment" { input class="text" type="text" name="comment" placeholder="Comment"; }
            td class="no-label" {}
            td class="no-label" {
                button class="btn btn-primary w-full"
                    hx-post="/ingredients"
                    hx-include="closest tr"
                    hx-target="#ingredient--1"
                    hx-swap="outerHTML"
                    hx-on::after-request="if(event.detail.successful){const i=document.querySelector('#ingredient--1 input[name=name]');if(i)i.focus();}"
                    { "Add" }
            }
        }
    }
}

async fn delete_ingredient(
    State(state): State<MyAppState>,
    user: User,
    id: Path<IngredientId>,
) -> MResponse {
    app::ingredients::delete_ingredient(&state, &user, id.0).await?;
    Ok(render_ingredients_view(State(state), Some(user)).await)
}

pub async fn edit_ingredient_form(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Form(ingredient): Form<IngredientWithSource>,
) -> MResponse {
    let id = ingredient.id;
    let owner_select = if id.is_valid() {
        Some(move_group::owner_select(&state, &ctx, ingredient.group_id).await?)
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

async fn delete_ingredient_form(
    State(state): State<MyAppState>,
    user: User,
    id: Path<IngredientId>,
) -> MResponse {
    let usages = app::ingredients::gather_usages(&state, &user, id.0).await?;

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

async fn sources_table(
    State(state): State<MyAppState>,
    user: Option<User>,
    id: Path<IngredientId>,
) -> MResponse {
    let app::ingredients::SourcesViewData {
        sources,
        stores,
        ingredient,
        user_group_ids,
    } = app::ingredients::gather_sources(&state, user.as_ref(), id.0).await?;
    let can_edit_sources = user
        .as_ref()
        .is_some_and(|u| app::ingredients::can_edit(ingredient.group_id, &user_group_ids, u));

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

async fn properties_dialog(
    State(state): State<MyAppState>,
    user: Option<User>,
    id: Path<IngredientId>,
) -> MResponse {
    let app::ingredients::PropertiesViewData {
        properties: props,
        ingredient,
        user_group_ids,
    } = app::ingredients::gather_properties(&state, user.as_ref(), id.0).await?;
    let can_edit = user
        .as_ref()
        .is_some_and(|u| app::ingredients::can_edit(ingredient.group_id, &user_group_ids, u));

    // Group properties by source kind for readability.
    let mut contains: Vec<&foodlib_new::property::Property> = Vec::new();
    let mut may_contain: Vec<&foodlib_new::property::Property> = Vec::new();
    for p in &props.properties {
        if p.name.starts_with("~spuren-") {
            may_contain.push(p);
        } else {
            // We don't have provenance stored, so we can't separate Category from Contains here.
            // Group everything non-trace into one "Direct properties" bucket.
            contains.push(p);
        }
    }

    fn pretty(name: &str) -> String {
        let stripped = name.strip_prefix("~spuren-").unwrap_or(name);
        let mut chars = stripped.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

    use foodlib_new::allergen::canonical;
    let contains_taken: std::collections::HashSet<&str> =
        contains.iter().map(|p| p.name.as_str()).collect();
    let may_contain_taken: std::collections::HashSet<String> = may_contain
        .iter()
        .map(|p| p.name.trim_start_matches("~spuren-").to_string())
        .collect();
    let contains_options: Vec<&'static str> = canonical::ALL
        .iter()
        .copied()
        .filter(|n| !contains_taken.contains(n))
        .collect();
    let may_contain_options: Vec<&'static str> = canonical::ALL
        .iter()
        .copied()
        .filter(|n| !may_contain_taken.contains(*n))
        .collect();

    let add_row = |kind: &'static str, options: &[&'static str]| -> Markup {
        let path = format!("/ingredients/properties/{}", ingredient.id);
        html! {
            div class="flex items-center gap-2 mt-2" {
                input type="hidden" name="kind" value=(kind);
                select name="canonical" required="true" class="fc-select flex-1" {
                    option value="" { "Select allergen…" }
                    @for c in options {
                        option value=(c) { (pretty(c)) }
                    }
                }
                button class="btn btn-primary"
                    hx-post=(path)
                    hx-include="closest div"
                    hx-target="#popup"
                    hx-swap="outerHTML" { "Add" }
            }
        }
    };

    Ok(html! {
        dialog open="true" class="dialog z-50" id="popup" {
            div class="flex justify-between items-center w-full mb-3 gap-2" {
                p class="text-xl font-semibold" { "Allergens: " (ingredient.name) }
                button class="btn btn-cancel" hx-swap="delete" hx-target="#popup" hx-get="/" { "Close" }
            }
            @if props.properties.is_empty() && !can_edit {
                p class="text-center opacity-70 py-4" { "No allergens tagged on this ingredient." }
            }

            div class="mb-4" {
                p class="font-semibold mb-2" { "Enthält / Direct properties" }
                @if contains.is_empty() && !can_edit {
                    p class="text-sm opacity-60" { "None." }
                } @else {
                    ul class="list-disc pl-6 flex flex-col gap-1" {
                        @for p in &contains {
                            li class="flex items-center justify-between gap-2" {
                                span { (pretty(&p.name)) }
                                @if can_edit {
                                    button class="btn btn-cancel"
                                        hx-delete=(format!("/ingredients/properties/{}/{}", ingredient.id, p.id))
                                        hx-target="#popup"
                                        hx-swap="outerHTML"
                                        hx-confirm=(format!("Remove `{}` from {}?", p.name, ingredient.name))
                                        { "Remove" }
                                }
                            }
                        }
                    }
                }
                @if can_edit && !contains_options.is_empty() {
                    (add_row("contains", &contains_options))
                }
            }

            div class="mb-4" {
                p class="font-semibold mb-2" { "Spuren / May contain" }
                @if may_contain.is_empty() && !can_edit {
                    p class="text-sm opacity-60" { "None." }
                } @else {
                    ul class="list-disc pl-6 flex flex-col gap-1" {
                        @for p in &may_contain {
                            li class="flex items-center justify-between gap-2" {
                                span { (pretty(&p.name)) }
                                @if can_edit {
                                    button class="btn btn-cancel"
                                        hx-delete=(format!("/ingredients/properties/{}/{}", ingredient.id, p.id))
                                        hx-target="#popup"
                                        hx-swap="outerHTML"
                                        hx-confirm=(format!("Remove `{}` from {}?", p.name, ingredient.name))
                                        { "Remove" }
                                }
                            }
                        }
                    }
                }
                @if can_edit && !may_contain_options.is_empty() {
                    (add_row("may_contain", &may_contain_options))
                }
            }

            p class="text-xs opacity-60 mt-3" {
                "Tags are populated by the Metro sync from product allergen disclosures. "
                "Manual edits here are NOT preserved across re-syncs unless you skip the wipe option."
            }
        }
    })
}

async fn delete_property(
    State(state): State<MyAppState>,
    user: User,
    Path((ingredient_id, property_id)): Path<(IngredientId, PropertyId)>,
) -> MResponse {
    app::ingredients::delete_property(&state, &user, ingredient_id, property_id).await?;
    properties_dialog(State(state), Some(user), Path(ingredient_id)).await
}

#[derive(Deserialize)]
struct AddPropertyForm {
    canonical: String,
    kind: String,
}

async fn add_property(
    State(state): State<MyAppState>,
    user: User,
    Path(ingredient_id): Path<IngredientId>,
    Form(form): Form<AddPropertyForm>,
) -> MResponse {
    app::ingredients::add_property(
        &state,
        &user,
        ingredient_id,
        form.canonical,
        form.kind,
    )
    .await?;
    properties_dialog(State(state), Some(user), Path(ingredient_id)).await
}

fn format_add_ingredient_source(stores: &[Store], ingredient_id: IngredientId) -> Markup {
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
    ingredient_id: IngredientId,
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
                td class="no-label" { button class="icon-btn" title="Delete" aria-label="Delete" hx-get=(format!("/ingredients/sources/delete/{}/{}", ingredient_id, source.id)) hx-target="#popup" hx-swap="outerHTML" { (icons::delete()) } }
            } @else {
                td class="no-label" {}
                td class="no-label" {}
            }
        }
    }
}

async fn fetch_ingredients_and_groups(
    state: &MyAppState,
    user: Option<&User>,
) -> Result<(Vec<IngredientWithSource>, Vec<GroupId>), Error> {
    let ingredients = app::ingredients::list_with_sources(state).await?;
    let user_group_ids = match user {
        Some(u) => app::users::user_group_ids(state, u.id).await?,
        None => vec![],
    };
    Ok((ingredients, user_group_ids))
}

fn format_ingredient(
    ingredient: &IngredientWithSource,
    user: Option<&User>,
    user_group_ids: &[GroupId],
) -> Markup {
    let sources_button_text = if ingredient.has_sources { "" } else { "⚠️" };
    let can_edit_this =
        user.is_some_and(|u| app::ingredients::can_edit(ingredient.group_id, user_group_ids, u));
    let is_admin = user.is_some_and(|u| u.is_admin);

    let comment = ingredient
        .comment
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();

    html! {
        tr id=(format!("ingredient-{}", ingredient.id)) {
            @if is_admin {
                td class="text-center opacity-70" data-label="ID" { (ingredient.id) }
            }
            td class="text-center" data-label="Name" {
                span class="inline-flex items-center gap-1" {
                    (ingredient.name)
                    button class="opacity-60 hover:opacity-100 cursor-pointer text-blue-600 dark:text-blue-400"
                        hx-get=(format!("/ingredients/properties/{}", ingredient.id))
                        hx-target="#content"
                        hx-swap="afterbegin"
                        title="View allergens"
                        type="button"
                        style="background: none; border: none; padding: 0; font-size: 0.85em;"
                        { "ⓘ" }
                }
            }
            td class="text-center" data-label="Energy (kJ/g)" { (ingredient.energy) }
            td class=(if comment.is_empty() { "text-center no-label" } else { "text-center" })
               data-label=(if comment.is_empty() { "" } else { "Comment" }) {
                (comment)
            }
            td class="no-label whitespace-nowrap" {
                div class="flex flex-row flex-nowrap gap-2 justify-end items-center" {
                    @if can_edit_this {
                        button class="icon-btn" title="Edit" aria-label="Edit"
                        hx-get="/ingredients/edit"
                        hx-target=(format!("#ingredient-{}", ingredient.id))
                        hx-vals=(serde_json::to_string(ingredient).unwrap()) { (icons::edit()) }
                    }
                    @if is_admin {
                        button class="icon-btn" title="Delete" aria-label="Delete"
                        hx-get=(format!("/ingredients/delete/{}", ingredient.id))
                        hx-target="#content"
                        hx-swap="afterbegin" { (icons::delete()) }
                    }
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
        }
    }
}
