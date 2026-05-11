use crate::FoodLib;
use axum::{
    extract::{Form, Path, Query},
    http::HeaderMap,
    routing::{get, post, put},
};
use bigdecimal::BigDecimal;
use foodlib_new::{
    auth_context::AuthCtx,
    ingredient::Ingredient,
    inventory::{Inventory, InventoryItem, InventoryItemWithName},
};
use maud::{html, Markup};
use serde::Deserialize;

use super::MResponse;
use crate::frontend::move_group;
use crate::MyAppState;

pub(crate) fn inventories_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/", get(list_view).post(create_inventory))
        .route("/new", get(new_inventory_form))
        .route("/{id}", put(rename_inventory).delete(delete_inventory))
        .route("/{id}/edit", get(edit_row))
        .route("/{id}/row", get(render_row))
        .route("/{id}/contents", get(contents_view))
        .route("/{id}/move", get(move_dialog).post(execute_move))
        .route("/{id}/items", post(add_item))
        .route(
            "/{id}/items/{ingredient_id}",
            put(update_item_amount).delete(delete_item),
        )
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .map_or(false, |v| v == "true")
}

#[derive(Deserialize, Default, Clone)]
pub struct ListFilters {
    #[serde(default)]
    pub open: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
}

impl ListFilters {
    fn open_ids(&self) -> Vec<i32> {
        self.open
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.trim().parse::<i32>().ok())
            .collect()
    }

    fn filter_str(&self) -> &str {
        self.filter.as_deref().unwrap_or("")
    }
}

async fn list_view(
    foodlib: FoodLib,
    _ctx: AuthCtx,
    headers: HeaderMap,
    Query(filters): Query<ListFilters>,
) -> MResponse {
    let inventories = foodlib.inventories().list().await?;
    let open = filters.open_ids();
    let filter = filters.filter_str();

    let body = render_list(&inventories, &open, filter);

    if is_htmx(&headers) {
        Ok(body)
    } else {
        Ok(body)
    }
}

fn render_list(inventories: &[Inventory], open: &[i32], filter: &str) -> Markup {
    html! {
        // Tiny client helper: when a <details> on this page toggles, keep the
        // ?open= URL param in sync so refresh/share preserves expansion state.
        // Defined once per render; idempotent.
        script {
            (maud::PreEscaped(r#"
            window.fcUpdateOpenParam = function(rowId, isOpen) {
                const id = rowId.replace(/^inv-/, '');
                const url = new URL(window.location.href);
                const cur = (url.searchParams.get('open') || '')
                    .split(',').filter(x => x && x !== id);
                if (isOpen) cur.push(id);
                if (cur.length) url.searchParams.set('open', cur.join(','));
                else url.searchParams.delete('open');
                history.replaceState(null, '', url.toString());
            };
            "#))
        }
        div id="inventories" class="w-full flex flex-col gap-2" {
            div class="flex flex-row items-center justify-between mb-2 gap-3" {
                h1 class="text-xl" { "Inventories" }
                button class="btn btn-primary"
                    hx-get="/inventories/new"
                    hx-target="#inventories"
                    hx-swap="afterbegin" { "Add Inventory (+)" }
            }
            div class="flex flex-col gap-2" {
                @if inventories.is_empty() {
                    p class="opacity-70" { "No inventories yet." }
                } @else {
                    @for inv in inventories {
                        (render_row_markup(inv, open.contains(&inv.id), filter))
                    }
                }
            }
        }
    }
}

async fn new_inventory_form() -> Markup {
    html! {
        form class="border rounded-lg p-2 flex flex-row items-center gap-2"
            hx-post="/inventories"
            hx-target="#inventories"
            hx-swap="outerHTML" {
            input type="text" name="name" placeholder="New inventory name" required="required" class="text grow";
            button type="submit" class="btn btn-primary" { "Create" }
            button type="button" class="btn btn-cancel"
                hx-on:click="this.closest('form').remove()" { "Cancel" }
        }
    }
}

/// One inventory row, rendered as a <details>. Lazy-loads the contents on first
/// open via hx-trigger="toggle once". `open` is the initial state from the URL.
fn render_row_markup(inv: &Inventory, open: bool, filter: &str) -> Markup {
    let row_id = format!("inv-{}", inv.id);
    let contents_id = format!("inv-{}-contents", inv.id);
    let contents_url = if filter.is_empty() {
        format!("/inventories/{}/contents", inv.id)
    } else {
        format!(
            "/inventories/{}/contents?filter={}",
            inv.id,
            urlencoding::encode(filter)
        )
    };

    // Lazy load on first open. We register two triggers depending on whether
    // the row is already open at render time: `load` (fires immediately) or
    // `toggle once` on the parent <details> (fires the first time the user
    // expands it). After the GET, htmx swaps the loading placeholder for the
    // contents fragment, which is no longer wired to fetch.
    let hx_trigger = if open {
        "load"
    } else {
        "toggle once from:closest details"
    };

    html! {
        details id=(row_id) class="border rounded-lg p-2 bg-light-bg-light dark:bg-dark-bg-dark"
            open[open]
            hx-on:toggle="window.fcUpdateOpenParam && window.fcUpdateOpenParam(this.id, this.open)" {
            summary class="flex flex-row items-center justify-between cursor-pointer gap-3" {
                span class="font-medium" { (inv.name) }
                div class="flex flex-row gap-2" onclick="event.stopPropagation()" {
                    button type="button" class="btn btn-primary"
                        hx-get=(format!("/inventories/{}/edit", inv.id))
                        hx-target=(format!("#{}", row_id))
                        hx-swap="outerHTML" { "Rename" }
                    button type="button" class="btn btn-primary"
                        hx-get=(format!("/inventories/{}/move", inv.id))
                        hx-target="body"
                        hx-swap="beforeend" { "Move" }
                    button type="button" class="btn btn-cancel"
                        hx-delete=(format!("/inventories/{}", inv.id))
                        hx-target="#inventories"
                        hx-confirm="Delete this inventory?" { "Delete" }
                }
            }
            div id=(contents_id) class="mt-2"
                hx-get=(contents_url)
                hx-trigger=(hx_trigger)
                hx-swap="innerHTML" {
                p class="opacity-70" { "Loading..." }
            }
        }
    }
}

async fn render_row(foodlib: FoodLib, Path(id): Path<i32>) -> MResponse {
    let inv = foodlib.inventories().get(id).await?;
    Ok(render_row_markup(&inv, false, ""))
}

async fn contents_view(
    foodlib: FoodLib,
    Path(id): Path<i32>,
    Query(filters): Query<ContentsFilters>,
) -> MResponse {
    render_contents(&foodlib, id, filters.filter.as_deref().unwrap_or("")).await
}

#[derive(Deserialize, Default)]
pub struct ContentsFilters {
    #[serde(default)]
    pub filter: Option<String>,
}

async fn render_contents(foodlib: &FoodLib, inventory_id: i32, filter: &str) -> MResponse {
    let mut items = foodlib.inventories().get_items(inventory_id).await?;
    if !filter.is_empty() {
        let needle = filter.to_lowercase();
        items.retain(|x| x.name.to_lowercase().contains(&needle));
    }
    let ingredients = foodlib.ingredients().list().await?;

    Ok(html! {
        div class="flex flex-col gap-2 w-full" {
            (add_item_form(inventory_id))
            datalist id=(format!("ingredients-{}", inventory_id)) {
                @for ing in &ingredients {
                    option value=(ing.name) {}
                }
            }
            @if items.is_empty() {
                p class="opacity-70 text-sm" { "Empty." }
            } @else {
                table class="w-full text-inherit table-auto" {
                    thead { tr { th { "Name" } th class="w-32" { "Amount (kg)" } th class="w-16" {} } }
                    tbody id=(format!("inv-{}-items", inventory_id)) {
                        @for item in &items {
                            (item_row(inventory_id, item))
                        }
                    }
                }
            }
        }
    })
}

fn add_item_form(inventory_id: i32) -> Markup {
    html! {
        form class="flex flex-row items-center gap-2"
            hx-post=(format!("/inventories/{}/items", inventory_id))
            hx-target=(format!("#inv-{}-contents", inventory_id))
            hx-swap="innerHTML" {
            input type="text" name="ingredient_name"
                list=(format!("ingredients-{}", inventory_id))
                placeholder="Ingredient" class="text" required="required";
            input type="number" name="amount" step="0.001" min="0" placeholder="Amount (kg)"
                class="text w-32" required="required";
            button type="submit" class="btn btn-primary" { "Add" }
        }
    }
}

fn item_row(inventory_id: i32, item: &InventoryItemWithName) -> Markup {
    let row_id = format!("inv-{}-item-{}", inventory_id, item.ingredient_id);
    html! {
        tr id=(row_id) {
            td { (item.name) }
            td {
                input type="number" step="0.001" min="0" value=(item.amount.round(3))
                    class="text w-full"
                    name="amount"
                    hx-put=(format!("/inventories/{}/items/{}", inventory_id, item.ingredient_id))
                    hx-trigger="change, keyup[keyCode==13] changed"
                    hx-target=(format!("#{}", row_id))
                    hx-swap="outerHTML";
            }
            td {
                button type="button" class="btn btn-cancel"
                    hx-delete=(format!("/inventories/{}/items/{}", inventory_id, item.ingredient_id))
                    hx-target=(format!("#inv-{}-contents", inventory_id))
                    hx-swap="innerHTML" { "X" }
            }
        }
    }
}

#[derive(Deserialize)]
struct NewInventoryForm {
    name: String,
}

async fn create_inventory(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(form): Form<NewInventoryForm>,
) -> MResponse {
    let group = foodlib.users().get_personal_group(ctx.user.id).await?;
    foodlib
        .inventories()
        .create(Inventory {
            id: -1,
            name: form.name,
            group_id: group.id,
        })
        .await?;
    let inventories = foodlib.inventories().list().await?;
    Ok(render_list(&inventories, &[], ""))
}

#[derive(Deserialize)]
struct RenameForm {
    name: String,
}

async fn rename_inventory(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Form(form): Form<RenameForm>,
) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    let mut inv = foodlib.inventories().get(id).await?;
    inv.name = form.name;
    foodlib.inventories().update(inv.clone()).await?;
    Ok(render_row_markup(&inv, false, ""))
}

async fn edit_row(foodlib: FoodLib, ctx: AuthCtx, Path(id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    let inv = foodlib.inventories().get(id).await?;
    let row_id = format!("inv-{}", inv.id);
    Ok(html! {
        details id=(row_id) class="border rounded-lg p-2" open {
            summary class="cursor-pointer" {
                form class="flex flex-row items-center gap-2"
                    hx-put=(format!("/inventories/{}", inv.id))
                    hx-target=(format!("#{}", row_id))
                    hx-swap="outerHTML" {
                    input type="text" name="name" value=(inv.name) required="required" class="text grow";
                    button type="submit" class="btn btn-primary" { "Save" }
                    button type="button" class="btn btn-cancel"
                        hx-get=(format!("/inventories/{}/row", inv.id))
                        hx-target=(format!("#{}", row_id))
                        hx-swap="outerHTML" { "Cancel" }
                }
            }
        }
    })
}

async fn delete_inventory(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    foodlib.inventories().delete(id).await?;
    let inventories = foodlib.inventories().list().await?;
    Ok(render_list(&inventories, &[], ""))
}

async fn move_dialog(foodlib: FoodLib, ctx: AuthCtx, Path(id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    let inv = foodlib.inventories().get(id).await?;
    let panel = move_group::move_panel(
        &foodlib,
        &ctx,
        inv.group_id,
        &format!("/inventories/{}/move", id),
        "#content",
    )
    .await?;
    Ok(html! {
        dialog open="true" class="dialog fixed top-1/4 left-1/2 -translate-x-1/2 z-50 shadow-xl"
            id="move-dialog"
            hx-on::after-request="if(event.detail.successful) this.remove()" {
            div class="flex flex-col m-2 gap-2 min-w-80" {
                p class="text-lg font-semibold" { "Move \"" (inv.name) "\"" }
                (panel)
                button type="button" class="btn btn-abort"
                    hx-on:click="document.getElementById('move-dialog').remove()" { "Cancel" }
            }
        }
    })
}

#[derive(Deserialize)]
struct MoveInventoryForm {
    group_id: i32,
}

async fn execute_move(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Form(form): Form<MoveInventoryForm>,
) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    move_group::assert_can_move_to(&ctx, form.group_id)?;
    foodlib.inventories().set_group(id, form.group_id).await?;
    let inventories = foodlib.inventories().list().await?;
    Ok(render_list(&inventories, &[id], ""))
}

#[derive(Deserialize)]
struct AddItemForm {
    ingredient_name: String,
    amount: BigDecimal,
}

async fn add_item(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Form(form): Form<AddItemForm>,
) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    let Ok(ingredient) = foodlib
        .ingredients()
        .get_by_name(&form.ingredient_name)
        .await
    else {
        return render_contents(&foodlib, id, "").await;
    };

    let items = foodlib.inventories().get_items(id).await?;
    let item = InventoryItem {
        inventory_id: id,
        ingredient_id: ingredient.id,
        amount: form.amount,
    };
    if items.iter().any(|x| x.ingredient_id == ingredient.id) {
        foodlib.inventories().update_item(item).await?;
    } else {
        foodlib.inventories().add_item(item).await?;
    }
    render_contents(&foodlib, id, "").await
}

#[derive(Deserialize)]
struct UpdateAmountForm {
    amount: BigDecimal,
}

async fn update_item_amount(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    Form(form): Form<UpdateAmountForm>,
) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    let updated = foodlib
        .inventories()
        .update_item(InventoryItem {
            inventory_id: id,
            ingredient_id,
            amount: form.amount,
        })
        .await?;
    let ingredient = foodlib.ingredients().get(ingredient_id).await?;
    let item: InventoryItemWithName = InventoryItemWithName {
        name: ingredient.name,
        ..updated.into()
    };
    Ok(item_row(id, &item))
}

async fn delete_item(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((id, ingredient_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_inventory(id).await?;
    foodlib
        .inventories()
        .delete_item(id, ingredient_id)
        .await?;
    render_contents(&foodlib, id, "").await
}

// Kept for unrelated callers that re-use the datalist of ingredients.
#[allow(dead_code)]
trait DataListFormattable {
    fn format_for_datalist(&self) -> Markup;
}

impl DataListFormattable for Ingredient {
    fn format_for_datalist(&self) -> Markup {
        html! { option value=(self.name) {} }
    }
}
