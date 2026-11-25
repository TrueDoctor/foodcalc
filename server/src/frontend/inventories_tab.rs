// Top imports:
use super::MResponse;
use crate::frontend::{icons, WebCtx};
use crate::AppState;
use axum::{
    extract::{Form, Path, Query, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use foodlib_new::entities::ingredient::Ingredient;
use foodlib_new::entities::inventory::{Inventory, InventoryItemWithName};
use foodlib_new::types::amount::Amount;
use foodlib_new::types::ids::{GroupId, IngredientId, InventoryId};
use maud::{html, Markup};
use serde::Deserialize;

pub(crate) fn inventories_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(list_view).post(create_inventory))
        .route("/new", get(new_inventory_form))
        .route("/{id}", put(rename_inventory).delete(delete_inventory))
        .route("/{id}/edit", get(edit_row))
        .route("/{id}/row", get(render_row))
        .route("/{id}/contents", get(contents_view))
        .route("/{id}/items", post(add_item))
        .route(
            "/{id}/items/{ingredient_id}",
            put(update_item_amount).delete(delete_item),
        )
        .route(
            "/{id}/items/{ingredient_id}/move-to/{to_id}",
            post(move_item),
        )
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some_and(|v| v == "true")
}

#[derive(Deserialize, Default, Clone)]
pub struct ListFilters {
    #[serde(default)]
    pub open: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
}

impl ListFilters {
    fn open_ids(&self) -> Vec<InventoryId> {
        self.open
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter_map(|s| s.trim().parse::<i32>().ok().map(InventoryId))
            .collect()
    }

    fn filter_str(&self) -> &str {
        self.filter.as_deref().unwrap_or("")
    }
}

async fn list_view(
    State(state): State<AppState>,
    _actor: WebCtx,
    headers: HeaderMap,
    Query(filters): Query<ListFilters>,
) -> MResponse {
    let inventories = foodlib_new::inventories::list_inventories(&state).await?;
    let open = filters.open_ids();
    let filter = filters.filter_str();

    let body = render_list(&inventories, &open, filter);

    if is_htmx(&headers) {
        Ok(body)
    } else {
        Ok(body)
    }
}

fn render_list(inventories: &[Inventory], open: &[InventoryId], filter: &str) -> Markup {
    html! {
        // Inline page glue: <details> URL-sync + Sortable.js wiring. Defined
        // once per list render; the htmx:load handler is idempotent so re-runs
        // on lazy-loaded contents reinitialise just the new tbodies.
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

            // Track which <details> were closed when a drag started so we can
            // restore them on dragend. While dragging, force all open so they
            // are valid drop targets.
            window.fcSavedOpen = window.fcSavedOpen || null;
            function fcOpenAllForDrag() {
                if (window.fcSavedOpen) return;
                window.fcSavedOpen = [];
                document.querySelectorAll('#inventories details').forEach(d => {
                    window.fcSavedOpen.push([d, d.open]);
                    d.open = true;
                });
            }
            function fcRestoreOpen() {
                if (!window.fcSavedOpen) return;
                window.fcSavedOpen.forEach(([d, was]) => { d.open = was; });
                window.fcSavedOpen = null;
            }

            window.fcInitSortable = function(tbody) {
                if (!tbody || tbody.dataset.sortableReady === '1') return;
                if (!window.Sortable) return;
                tbody.dataset.sortableReady = '1';
                Sortable.create(tbody, {
                    group: 'inventory-items',
                    draggable: '.item-row',
                    filter: '.add-row',
                    // Without this, Sortable preventDefaults mousedown on
                    // filtered rows, which stops the inline-add inputs from
                    // ever receiving focus.
                    preventOnFilter: false,
                    ghostClass: 'fc-ghost',
                    chosenClass: 'fc-chosen',
                    dragClass: 'fc-drag',
                    animation: 150,
                    delay: 150,
                    delayOnTouchOnly: true,
                    onStart: fcOpenAllForDrag,
                    onEnd: fcRestoreOpen,
                    onAdd: function(evt) {
                        const item = evt.item;
                        const fromId = evt.from.dataset.inventoryId;
                        const toId = evt.to.dataset.inventoryId;
                        const ingredientId = item.dataset.ingredientId;
                        if (!fromId || !toId || !ingredientId) return;
                        // Revert the visual move; the server response will
                        // OOB-swap both source and destination contents.
                        evt.from.appendChild(item);
                        htmx.ajax(
                            'POST',
                            `/inventories/${fromId}/items/${ingredientId}/move-to/${toId}`,
                            { target: `#inv-${toId}-contents`, swap: 'innerHTML' }
                        );
                    }
                });
            };

            document.body.addEventListener('htmx:load', function(evt) {
                const root = evt.detail.elt || document;
                root.querySelectorAll('tbody.inventory-items').forEach(window.fcInitSortable);
            });
            // Also run once now for any items already in the DOM.
            document.querySelectorAll('tbody.inventory-items').forEach(window.fcInitSortable);
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
        details id=(row_id) class="border rounded-lg bg-light-bg-light dark:bg-dark-bg-dark"
            open[open]
            hx-on:toggle="window.fcUpdateOpenParam && window.fcUpdateOpenParam(this.id, this.open)" {
            summary class="flex flex-row items-center justify-between cursor-pointer gap-3 p-2
                hover:bg-light-primary-normal hover:text-white dark:hover:bg-dark-primary-normal
                rounded-lg transition-colors duration-150 list-none" {
                div class="flex items-center gap-2" {
                    svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512"
                        class="h-3 w-3 text-current transition-transform duration-200 inv-chevron" {
                        path fill="currentColor" d="M137.4 374.6c12.5 12.5 32.8 12.5 45.3 0l128-128c9.2-9.2 11.9-22.9 6.9-34.9s-16.6-19.8-29.6-19.8L32 192c-12.9 0-24.6 7.8-29.6 19.8s-2.2 25.7 6.9 34.9l128 128z" {}
                    }
                    span class="text-sm opacity-70 tabular-nums" { "#" (inv.id) }
                    span class="font-medium" { (inv.name) }
                }
                div class="flex flex-row gap-2" onclick="event.stopPropagation()" {
                    button type="button" class="icon-btn" title="Edit" aria-label="Edit"
                        hx-get=(format!("/inventories/{}/edit", inv.id))
                        hx-target=(format!("#{}", row_id))
                        hx-swap="outerHTML" { (icons::edit()) }
                    button type="button" class="icon-btn" title="Delete" aria-label="Delete"
                        hx-delete=(format!("/inventories/{}", inv.id))
                        hx-target="#inventories"
                        hx-confirm="Delete this inventory?" { (icons::delete()) }
                }
            }
            div id=(contents_id) class="mt-2 px-2 pb-2"
                hx-get=(contents_url)
                hx-trigger=(hx_trigger)
                hx-swap="innerHTML" {
                p class="opacity-70" { "Loading..." }
            }
        }
    }
}

async fn render_row(State(state): State<AppState>, Path(id): Path<InventoryId>) -> MResponse {
    let inv = foodlib_new::inventories::get_inventory(&state, id).await?;
    Ok(render_row_markup(&inv, false, ""))
}

async fn contents_view(
    State(state): State<AppState>,
    Path(id): Path<InventoryId>,
    Query(filters): Query<ContentsFilters>,
) -> MResponse {
    render_contents(&state, id, filters.filter.as_deref().unwrap_or("")).await
}

#[derive(Deserialize, Default)]
pub struct ContentsFilters {
    #[serde(default)]
    pub filter: Option<String>,
}

async fn render_contents(
    state: &AppState,
    inventory_id: InventoryId,
    filter: &str,
) -> MResponse {
    let data = foodlib_new::inventories::gather_contents(state, inventory_id).await?;
    let mut items = data.items;
    if !filter.is_empty() {
        let needle = filter.to_lowercase();
        items.retain(|x| x.name.to_lowercase().contains(&needle));
    }
    let ingredients = data.ingredients;

    Ok(html! {
        div class="flex flex-col gap-2 w-full" {
            // Each option carries the ingredient id so the add-row's JS handler
            // can resolve the typed name to an id without a server round-trip.
            datalist id=(format!("ingredients-{}", inventory_id)) {
                @for ing in &ingredients {
                    option value=(ing.name) data-id=(ing.id) {}
                }
            }
            table class="w-full text-inherit table-auto responsive-card" {
                thead { tr { th class="py-2" { "Name" } th class="w-32 py-2 whitespace-nowrap" { "Amount (kg)" } th class="w-16 py-2" {} } }
                tbody id=(format!("inv-{}-items", inventory_id))
                    class="inventory-items"
                    data-inventory-id=(inventory_id) {
                    (add_item_row(inventory_id))
                    @for item in &items {
                        (item_row(inventory_id, item))
                    }
                }
            }
        }
    })
}

/// Permanent first row of each inventory's items table — an inline form for
/// adding a new item. Uses the standardized id-based sentinel:
/// - The visible name input is matched against the datalist's `data-id`
///   attribute to populate a hidden `ingredient_id`.
/// - `setCustomValidity` marks the name input invalid whenever it doesn't
///   match a datalist option, so the browser blocks submit with a native
///   tooltip; the button additionally calls `reportValidity()` on click so
///   the message is shown immediately rather than only on blur.
fn add_item_row(inventory_id: InventoryId) -> Markup {
    let list_id = format!("ingredients-{}", inventory_id);
    // Inline handler: resolves the typed name to an ingredient id by looking
    // up the matching `<option data-id>` in the associated datalist. Runs on
    // input AND change so paste/autocomplete-pick keep the hidden id in sync.
    // Also toggles the input's custom validity so the browser blocks submit
    // when the name doesn't match any known ingredient.
    let resolve_id = format!(
        "const o=document.querySelector('#{list} option[value='+JSON.stringify(this.value)+']'); const tr=this.closest('tr'); tr.querySelector('input[name=ingredient_id]').value = o ? o.dataset.id : '-1'; this.setCustomValidity(o ? '' : 'Pick an ingredient from the list');",
        list = list_id
    );
    // Gate the htmx request on the name input's validity. Without this,
    // hx-post fires regardless of the form's validation state.
    let validate_on_click = "const tr=this.closest('tr'); const ni=tr.querySelector('input[name=ingredient_name]'); if(!ni.reportValidity()){event.preventDefault(); event.stopPropagation(); return;}";
    html! {
        tr class="add-row" data-sortable-ignore="true" data-inv-id=(inventory_id) {
            input type="hidden" name="ingredient_id" value="-1";
            td data-label="Name" {
                input type="text" name="ingredient_name"
                    list=(list_id)
                    placeholder="Add ingredient..." class="text w-full" required="required"
                    hx-on:input=(resolve_id)
                    hx-on:change=(resolve_id);
            }
            td data-label="Amount (kg)" {
                input type="number" name="amount"
                    step="0.001" min="0" placeholder="kg"
                    class="text w-full" required="required";
            }
            td class="no-label" {
                button type="button" class="btn btn-primary w-full"
                    hx-post=(format!("/inventories/{}/items", inventory_id))
                    hx-include="closest tr"
                    hx-target=(format!("#inv-{}-contents", inventory_id))
                    hx-swap="innerHTML"
                    hx-on:click=(validate_on_click)
                    hx-on::after-request=(format!("if(event.detail.successful){{const inp=document.querySelector('#inv-{}-items input[name=ingredient_name]'); inp && inp.focus();}}", inventory_id))
                    { "Add" }
            }
        }
    }
}

fn item_row(inventory_id: InventoryId, item: &InventoryItemWithName) -> Markup {
    let row_id = format!("inv-{}-item-{}", inventory_id, item.ingredient_id);
    html! {
        tr id=(row_id)
            class="item-row"
            data-ingredient-id=(item.ingredient_id) {
            td data-label="Name" { (item.name) }
            td data-label="Amount (kg)" {
                input type="number" step="0.001" min="0" value=(item.amount.round(3))
                    class="text w-full"
                    name="amount"
                    hx-put=(format!("/inventories/{}/items/{}", inventory_id, item.ingredient_id))
                    hx-trigger="change, keyup[keyCode==13] changed"
                    hx-target=(format!("#{}", row_id))
                    hx-swap="outerHTML";
            }
            td class="no-label" {
                button type="button" class="icon-btn" title="Remove" aria-label="Remove"
                    hx-delete=(format!("/inventories/{}/items/{}", inventory_id, item.ingredient_id))
                    hx-target=(format!("#inv-{}-contents", inventory_id))
                    hx-swap="innerHTML" { (icons::delete()) }
            }
        }
    }
}

#[derive(Deserialize)]
struct NewInventoryForm {
    name: String,
}

async fn create_inventory(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<NewInventoryForm>,
) -> MResponse {
    foodlib_new::inventories::create_inventory(&state, &actor, form.name).await?;
    let inventories = foodlib_new::inventories::list_inventories(&state).await?;
    Ok(render_list(&inventories, &[], ""))
}

#[derive(Deserialize)]
struct RenameForm {
    name: String,
    group_id: GroupId,
}

async fn rename_inventory(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(id): Path<InventoryId>,
    Form(form): Form<RenameForm>,
) -> MResponse {
    let inv =
        foodlib_new::inventories::rename_inventory(&state, &actor, id, form.name, form.group_id).await?;
    Ok(render_row_markup(&inv, false, ""))
}

async fn edit_row(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(id): Path<InventoryId>,
) -> MResponse {
    let inv = foodlib_new::inventories::get_inventory_for_edit(&state, &actor, id).await?;
    let owner_select =
        crate::frontend::move_group::owner_select(&state, &actor, inv.group_id).await?;
    let row_id = format!("inv-{}", inv.id);
    Ok(html! {
        details id=(row_id) class="border rounded-lg p-2" open {
            summary class="cursor-pointer" {
                form class="flex flex-row items-center gap-2 flex-wrap"
                    hx-put=(format!("/inventories/{}", inv.id))
                    hx-target=(format!("#{}", row_id))
                    hx-swap="outerHTML" {
                    input type="text" name="name" value=(inv.name) required="required" class="text grow";
                    div class="flex flex-row items-center gap-2" { (owner_select) }
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
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(id): Path<InventoryId>,
) -> MResponse {
    foodlib_new::inventories::delete_inventory(&state, &actor, id).await?;
    let inventories = foodlib_new::inventories::list_inventories(&state).await?;
    Ok(render_list(&inventories, &[], ""))
}

#[derive(Deserialize)]
struct AddItemForm {
    ingredient_id: IngredientId,
    amount: Amount,
}

async fn add_item(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(id): Path<InventoryId>,
    Form(form): Form<AddItemForm>,
) -> MResponse {
    foodlib_new::inventories::set_item(&state, &actor, id, form.ingredient_id, form.amount).await?;
    render_contents(&state, id, "").await
}

#[derive(Deserialize)]
struct UpdateAmountForm {
    amount: Amount,
}

async fn update_item_amount(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((id, ingredient_id)): Path<(InventoryId, IngredientId)>,
    Form(form): Form<UpdateAmountForm>,
) -> MResponse {
    let item = foodlib_new::inventories::update_item_amount(
        &state,
        &actor,
        id,
        ingredient_id,
        form.amount,
    )
    .await?;
    Ok(item_row(id, &item))
}

async fn delete_item(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((id, ingredient_id)): Path<(InventoryId, IngredientId)>,
) -> MResponse {
    foodlib_new::inventories::delete_item(&state, &actor, id, ingredient_id).await?;
    render_contents(&state, id, "").await
}

/// Move (drag-and-drop) an item from one inventory to another. The whole
/// amount transfers. If the destination already has that ingredient the
/// amounts are summed. Response is the destination contents plus an
/// `hx-swap-oob` block that updates the source inventory's contents in the
/// same response — no second round trip.
async fn move_item(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((from_id, ingredient_id, to_id)): Path<(InventoryId, IngredientId, InventoryId)>,
) -> MResponse {
    let moved =
        foodlib_new::inventories::move_item(&state, &actor, from_id, to_id, ingredient_id).await?;
    if !moved {
        return render_contents(&state, to_id, "").await;
    }

    let source_contents = render_contents(&state, from_id, "").await?;
    let dest_contents = render_contents(&state, to_id, "").await?;

    Ok(html! {
        (dest_contents)
        // Out-of-band: replace the source inventory's contents in the same response.
        div id=(format!("inv-{}-contents", from_id))
            hx-swap-oob="innerHTML" {
            (source_contents)
        }
    })
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
