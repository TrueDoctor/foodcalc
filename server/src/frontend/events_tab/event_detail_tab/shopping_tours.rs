use crate::{
    frontend::{events_tab::event_detail_tab, IResponse, MResponse, WebCtx},
    MyAppState,
};
use axum::{
    extract::{Form, Path, State},
    response::IntoResponse,
    routing::{delete, get, post},
};
use bigdecimal::ToPrimitive;
use foodlib_new::{
    auth_context::AuthCtx,
    entities::types::amount::Amount,
    entities::types::ids::{EventId, IngredientId, InventoryId, ShoppingTourId, StoreId},
    error::Result,
    event::ShoppingListItem,
    inventory::InventoryItemWithName,
};
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

pub(crate) fn shopping_tour_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/add/{event_id}", get(add_shopping_tour))
        .route("/create_inline/{event_id}", post(create_inline))
        .route("/edit/{event_id}/{tour_id}", get(edit_shopping_tour))
        .route("/save", post(save_tour))
        .route("/export/{tour_id}/plain", get(export_plain))
        .route("/export/{tour_id}/metro", get(export_metro))
        .route("/update_inventory/{tour_id}", post(update_inventory))
        .route("/confirm_update/{tour_id}", get(confirm_update))
        .route("/{tour_id}", delete(delete_shopping_tour))
        .route(
            "/toggle_inventory/{event_id}/{inventory_id}",
            post(toggle_inventory),
        )
}

#[derive(Deserialize)]
struct TourForm {
    event_id: EventId,
    tour_id: Option<ShoppingTourId>,
    store_id: StoreId,
    date: String,
}

#[derive(Deserialize)]
struct ToggleForm {
    checked: bool,
    tour_id: ShoppingTourId,
}

async fn add_shopping_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<EventId>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    render_shopping_tour_form(&state, &ctx, event_id, None).await
}

async fn edit_shopping_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, tour_id)): Path<(EventId, ShoppingTourId)>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    render_shopping_tour_form(&state, &ctx, event_id, Some(tour_id)).await
}


#[derive(Deserialize)]
struct InlineTourForm {
    store_id: StoreId,
    date: String,
}

async fn create_inline(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
    Form(form): Form<InlineTourForm>,
) -> MResponse {
    let tour_date = parse_datetime_local(&form.date)?;
    app::events::shopping::create_tour(&state, &actor, event_id, form.store_id, tour_date).await?;
    event_detail_tab::event_form(State(state), WebCtx(actor), Path(event_id)).await
}

/// Parse a `datetime-local` value, accepting a date-only string (e.g.
/// "2026-05-12") by defaulting the time to 00:00. Browsers may submit either
/// the full `YYYY-MM-DDTHH:MM` form or just `YYYY-MM-DD` depending on whether
/// the user filled in the time portion.
pub(crate) fn parse_datetime_local(s: &str) -> Result<OffsetDateTime> {
    let normalized = if s.len() == 10 {
        format!("{}T00:00", s)
    } else {
        s.to_string()
    };
    let dt = time::PrimitiveDateTime::parse(
        &normalized,
        format_description!("[year]-[month]-[day]T[hour]:[minute]"),
    )
    .map_err(|e| foodlib_new::Error::Misc(e.to_string()))?;
    Ok(dt.assume_utc())
}

async fn delete_shopping_tour(
    state: State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> MResponse {
    app::events::shopping::delete_tour(&state, &actor, tour_id).await?;
    Ok(html! {})
}


async fn render_shopping_tour_form(
    state: &MyAppState,
    ctx: &AuthCtx,
    event_id: EventId,
    tour_id: Option<ShoppingTourId>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;
    if let Some(id) = tour_id {
        ctx.assert_can_edit_shopping_tour(id).await?;
    }

    let stores = state.stores().list().await?;
    let inventories = state.inventories().list().await?;

    let (existing, event_inventories, shopping_list) = match tour_id {
        Some(id) => (
            Some(state.events().get_shopping_tour(id).await?),
            state.events().get_inventories(event_id).await?,
            state.events().get_shopping_list(id).await?,
        ),
        None => (None, vec![], vec![]),
    };

    let selected_store_id = existing.as_ref().map(|t| t.store_id);
    let initial_date = existing
        .as_ref()
        .map(|t| t.tour_date)
        .unwrap_or_else(OffsetDateTime::now_utc);

    let inventory_items = inventory_items(tour_id, &event_inventories, state).await;

    Ok(html! {
        div class="flex-col space-y-4 w-full" {
            h2 class="text-xl" {
                @if tour_id.is_some() {
                    "Edit Shopping Tour"
                } @else {
                    "Add Shopping Tour"
                }
            }

            // Stack form and shopping list vertically on mobile, side by side on desktop
            div class="flex flex-col lg:flex-row gap-4" {
                // Form section - takes full width on mobile, half on desktop
                form class="w-full lg:w-1/2 space-y-4" hx-post="/events/edit/shopping_tours/save" hx-target="#content" {
                    input type="hidden" name="event_id" value=(event_id);
                    @if let Some(id) = tour_id {
                        input type="hidden" name="tour_id" value=(id);
                    }

                    div class="flex flex-col space-y-2" {
                        label for="store_id" { "Store" }
                        select name="store_id" class="text" required {
                            @for store in stores {
                                option value=(store.id)
                                    selected[selected_store_id == Some(store.id)] {
                                    (store.name)
                                }
                            }
                        }
                    }

                    div class="flex flex-col space-y-2" {
                        label for="date" { "Date & Time" }
                        input type="datetime-local"
                            name="date"
                            class="text"
                            value=(initial_date
                                .format(format_description!("[year]-[month]-[day]T[hour]:[minute]"))
                                .unwrap())
                            required;
                    }

                    @if let Some(id) = tour_id {
                        div class="flex flex-col space-y-2" {
                            label { "Use Inventories" }
                            div class="space-y-1" {
                                @for inventory in inventories {
                                    @let is_checked = event_inventories.iter().any(|inv| inv.inventory_id == inventory.id);
                                    div class="flex items-center gap-2" {
                                        input type="checkbox"
                                            class="w-4 h-4"
                                            checked[is_checked]
                                            hx-post=(format!("/events/edit/shopping_tours/toggle_inventory/{}/{}", event_id, inventory.id))
                                            hx-target="#content"
                                            hx-vals=(format!("{{\"checked\": {}, \"tour_id\": {}}}", !is_checked, id));
                                        span { (inventory.name) }
                                    }
                                }
                            }
                        }
                    }

                    div class="flex gap-2 mt-4" {
                        button type="submit" class="btn btn-primary" {
                            @if tour_id.is_some() {
                                "Update"
                            } @else {
                                "Create"
                            }
                        }
                        button type="button"
                            class="btn btn-cancel"
                            hx-get=(format!("/events/edit/{}", event_id))
                            hx-target="#content" { "Cancel" }
                    }
                }

                // Shopping list section - takes full width on mobile, half on desktop
                @if let (Some(id), false) = (tour_id, shopping_list.is_empty()) {
                    div class="w-full lg:w-1/2 space-y-4" {
                        // Action buttons in a grid on mobile, row on desktop
                        div class="grid grid-cols-2 lg:flex gap-2 mb-4" {
                            a class="btn btn-primary text-center"
                                href=(format!("/events/edit/shopping_tours/export/{}/plain", id))
                                target="_blank" {
                                "Export Plain"
                            }
                            a class="btn btn-primary text-center"
                                href=(format!("/events/edit/shopping_tours/export/{}/metro", id))
                                target="_blank" {
                                "Export Metro"
                            }
                            button class="btn btn-cancel col-span-2 lg:col-span-1"
                                hx-get=(format!("/events/edit/shopping_tours/confirm_update/{}", id))
                                hx-target="closest div" {
                                "Update Inventory"
                            }
                        }

                        // Table with horizontal scroll on mobile if needed
                        div class="overflow-x-auto" {
                            table class="w-full text-inherit table-auto min-w-[500px]" {
                                thead {
                                    tr {
                                        th { "Name" }
                                        th class="w-20" { "Amount" }
                                        th class="w-20" { "Price" }
                                        th class="w-36" { "Category" }
                                        th class="w-20" { "Status" }
                                    }
                                }
                                tbody {
                                    @for item in shopping_list {
                                        tr class=(get_row_class(&item, &inventory_items)) {
                                            td { (item.ingredient_name) }
                                            td class="text-right" {
                                                @if let Some(ref weight) = item.weight {
                                                    (format!("{:.2} kg", weight))
                                                } @else {
                                                    "? kg (keine Umrechnung)"
                                                }
                                            }
                                            td class="text-right" {
                                                @if let Some(ref price) = item.price {
                                                    (format!("€{:.2}", price))
                                                }
                                            }
                                            td { (item.category.clone().unwrap_or_default()) }
                                            td { (get_inventory_status(&item, &inventory_items)) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

async fn inventory_items(
    tour_id: Option<ShoppingTourId>,
    event_inventories: &Vec<foodlib_new::event::EventInventory>,
    lib: &foodlib_new::FoodLib,
) -> Vec<InventoryItemWithName> {
    let inventory_items = if tour_id.is_some() {
        // Get all inventory items for the event
        let mut all_items = vec![];
        for inv in event_inventories {
            if let Ok(items) = lib.inventories().get_items(inv.inventory_id).await {
                all_items.extend(items);
            }
        }
        all_items
    } else {
        vec![]
    };
    inventory_items
}

async fn save_tour(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<TourForm>,
) -> MResponse {
    let tour_date = parse_datetime_local(&form.date)?;

    let result = app::events::shopping::save_tour(
        &state,
        &actor,
        form.event_id,
        form.tour_id,
        form.store_id,
        tour_date,
    )
    .await;
    if let Err(ref e) = result {
        log::error!("Encountered error while saving tour {e}");
    }

    Ok(html! {
        (event_detail_tab::event_form(State(state), WebCtx(actor), Path(form.event_id)).await?)
    })
}

fn get_row_class(item: &ShoppingListItem, inventory: &[InventoryItemWithName]) -> &'static str {
    let inv_amount = inventory
        .iter()
        .find(|i| i.ingredient_id == item.ingredient_id)
        .map(|i| i.amount.clone())
        .unwrap_or_default();

    let covered = item
        .weight
        .as_ref()
        .is_some_and(|weight| inv_amount >= *weight);

    if covered {
        "bg-green-100 dark:bg-green-900"
    } else if inv_amount > Amount::default() {
        "bg-red-100 dark:bg-red-900"
    } else {
        ""
    }
}
fn get_inventory_amount(item: &ShoppingListItem, inventory: &[InventoryItemWithName]) -> Amount {
    get_inventory_amount_by_id(item.ingredient_id, inventory)
}

fn get_inventory_amount_by_id(
    ingredient_id: IngredientId,
    inventory: &[InventoryItemWithName],
) -> Amount {
    inventory
        .iter()
        .filter(|i| i.ingredient_id == ingredient_id)
        .map(|i| i.amount.clone())
        .sum::<Amount>()
}

fn get_inventory_status(item: &ShoppingListItem, inventory: &[InventoryItemWithName]) -> String {
    let inv_amount = get_inventory_amount(item, inventory);

    match item.weight.as_ref() {
        Some(weight) if inv_amount >= *weight => "Available".to_string(),
        Some(_) if inv_amount > Amount::default() => {
            format!("Insufficient ({:.2} kg available)", inv_amount)
        }
        Some(_) => "Not in inventory".to_string(),
        None => "Unknown (no unit weight)".to_string(),
    }
}

async fn export_plain(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> IResponse {
    ctx.assert_can_edit_shopping_tour(tour_id).await?;
    // Single query: tour shopping list + per-recipe usage breakdown.
    let shopping_list = state
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await?;
    let tour = state.events().get_shopping_tour(tour_id).await?;

    let event_inventories = state.events().get_inventories(tour.event_id).await?;
    let inventory_items = inventory_items(Some(tour_id), &event_inventories, &state).await;

    let mut output = String::new();
    let mut current_category = String::new();

    for item in shopping_list {
        let category = item.category.clone().unwrap_or_default();
        if category != current_category {
            if !current_category.is_empty() {
                output.push('\n');
            }
            output.push_str(&format!("=== {} ===\n", category));
            current_category = category;
        }
        let inv_amount = get_inventory_amount_by_id(item.ingredient_id, &inventory_items);
        // Unknown weight → the buy amount can't be resolved; keep the row
        // visible with a "?" placeholder instead of inventing numbers.
        let (status, to_buy, weight_text, new_price) = match &item.weight {
            Some(weight) => {
                let status = if inv_amount > *weight { "x" } else { " " };
                // Clamp at zero: Amount has no `Ord`, so `max` isn't available.
                let diff = weight.clone() - inv_amount.clone();
                let to_buy = if diff < Amount::default() {
                    Amount::default()
                } else {
                    diff
                };

                // Scale package price proportionally to `to_buy` (`price / weight * to_buy`),
                // returning `None` if price/weight is missing or weight is zero.
                let mut new_price = None;
                if let (Some(price), Some(weight)) = (&item.price, &item.weight) {
                    if let Some(unit_price) = price.checked_div_amount(weight) {
                        new_price = Some(unit_price * &to_buy);
                    }
                }

                (status, to_buy, format!("{:.2} kg", weight), new_price)
            }
            None => ("?", Amount::default(), "? kg".to_string(), None),
        };

        output.push_str(&format!(
            "[{}] {} - {:.2} kg {} -> {:.2} kg",
            status,
            item.ingredient_name,
            to_buy,
            if let Some(price) = new_price {
                format!(" (€{:.2})", price)
            } else {
                String::new()
            },
            weight_text
        ));
        // Usage breakdown without the ingredient-name prefix the Metro export uses.
        let usage = format_recipe_usage(&item.recipe_usage);
        if !usage.is_empty() {
            output.push_str(&format!("  [{}]", usage));
        }
        output.push('\n');
    }
    Ok(IntoResponse::into_response((
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/plain; charset=utf-8".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("inline; filename=\"shopping_list_{}.txt\"", tour_id),
            ),
        ],
        output,
    )))
}

/// Format a per-recipe usage breakdown as "3.00kg Recipe 2.00kg Other".
/// `usages` is the `[(recipe, kg)]` list from `tour_shopping_list_with_usage`.
/// Returns an empty string when there are no usages.
fn format_recipe_usage(usages: &[(String, Option<Amount>)]) -> String {
    usages
        .iter()
        .map(|(recipe, kg)| match kg {
            Some(kg) => format!("{:.2}kg {}", kg.to_f64().unwrap_or_default(), recipe),
            None => format!("?kg {}", recipe),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn export_metro(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> IResponse {
    ctx.assert_can_edit_shopping_tour(tour_id).await?;
    // Single query: tour shopping list + per-recipe usage breakdown.
    let shopping_list = state
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await?;
    let sources = state.ingredients().list_sources(None).await?;

    let tour = state.events().get_shopping_tour(tour_id).await?;
    let event_inventories = state.events().get_inventories(tour.event_id).await?;
    let inventory_items = inventory_items(Some(tour_id), &event_inventories, &state).await;

    // ingredient_id -> Metro article number (e.g. "208486"), resolved from the
    // sync cache. Ingredients that can't be resolved are absent here and get
    // their name in column 1 + sorted to the bottom of the CSV.
    let article_numbers = state
        .events()
        .resolve_metro_article_numbers(tour_id)
        .await?;

    // CSV rows: (article_number_or_name, amount, comment, resolved?). Resolved
    // rows are emitted first; unresolved (no Metro article number) sink below.
    let mut rows: Vec<(String, f64, String, bool)> = Vec::new();

    for item in shopping_list {
        // Unresolvable weight (unknown unit weight) → not orderable via Metro.
        let Some(item_weight) = item.weight else {
            continue;
        };
        let inv_amount = get_inventory_amount_by_id(item.ingredient_id, &inventory_items);
        if inv_amount > item_weight {
            continue;
        }
        let weight = item_weight - inv_amount;

        // Find source with matching ingredient_id
        let Some(source) = sources
            .iter()
            .find(|s| s.ingredient_id == item.ingredient_id)
        else {
            continue;
        };

        // Round up to whole packages; column 2 is the package count Metro orders.
        let package_size = source.package_size.to_f64().unwrap();
        let packages = (weight.to_f64().unwrap() / package_size).ceil();
        let total_kg = packages * package_size;

        // Comment: "<name>(<count> * <size>kg = <total>kg): <usage breakdown>".
        // The prefix explains how the package count maps to kg; the usage list
        // is the per-recipe breakdown for this tour.
        let mut comment = format!(
            "{}({} * {:.2}kg = {:.2}kg)",
            item.ingredient_name, packages as i64, package_size, total_kg
        );
        let usage = format_recipe_usage(&item.recipe_usage);
        if !usage.is_empty() {
            comment.push_str(&format!(": {usage}"));
        }

        match article_numbers.get(&item.ingredient_id) {
            Some(article) => rows.push((article.clone(), packages, comment, true)),
            // Unresolved: fall back to the ingredient name and sink to the bottom.
            None => rows.push((item.ingredient_name.clone(), packages, comment, false)),
        }
    }

    // Resolved rows first (stable within each group preserves the shopping-list
    // category/name ordering).
    rows.sort_by_key(|(_, _, _, resolved)| !*resolved);

    let mut output = String::new();
    for (col1, packages, comment, _) in rows {
        // Quote the comment (may contain commas); CSV-escape embedded quotes.
        let comment_field = if comment.is_empty() {
            String::new()
        } else {
            format!("\"{}\"", comment.replace('"', "\"\""))
        };
        // Column 2 is a whole package count.
        output.push_str(&format!("{},{},{}\n", col1, packages as i64, comment_field));
    }

    Ok(IntoResponse::into_response((
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/csv; charset=utf-8".to_string(),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "inline; filename=\"metro_list.csv\"".to_string(),
            ),
        ],
        output,
    )))
}

async fn toggle_inventory(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path((event_id, inventory_id)): Path<(EventId, InventoryId)>,
    Form(form): Form<ToggleForm>,
) -> MResponse {
    app::events::shopping::set_event_inventory(&state, &actor, event_id, inventory_id, form.checked)
        .await?;

    render_shopping_tour_form(&state, &actor.ctx, event_id, Some(form.tour_id)).await
}

async fn update_inventory(
    State(state): State<MyAppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> MResponse {
    app::events::shopping::apply_inventory_changes(&state, &actor, tour_id).await?;
    let tour = state.events().get_shopping_tour(tour_id).await?;

    // Redirect back to event page
    event_detail_tab::event_form(State(state), WebCtx(actor), Path(tour.event_id)).await
}

async fn confirm_update(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> Result<Markup> {
    ctx.assert_can_edit_shopping_tour(tour_id).await?;
    let lib = &state.db;

    // Calculate expected changes
    let changes = app::events::shopping::calculate_inventory_changes(&state, tour_id)
        .await
        .unwrap();

    let tour = lib.events().get_shopping_tour(tour_id).await.unwrap();

    Ok(html! {
        div class="flex flex-col space-y-4 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg" {
            h3 class="text-xl text-red-600 dark:text-red-400" { "Warning!" }
            p class="text-gray-700 dark:text-gray-300 mb-4" {
                "This will subtract the listed amounts from your inventories. This action cannot be undone."
            }

            // Inventory changes table
            @if !changes.is_empty() {
                div class="overflow-x-auto mb-4" {
                    table class="w-full text-inherit table-auto min-w-[500px]" {
                        thead {
                            tr {
                                th { "Inventory" }
                                th { "Product" }
                                th class="text-right" { "Current" }
                                th class="text-right" { "Change" }
                                th class="text-right" { "New Value" }
                            }
                        }
                        tbody {
                            @for (inv_name, _inv_id, prod_name, _ingredient_id, current, change, new_val) in &changes {
                                tr {
                                    td { (inv_name) }
                                    td { (prod_name) }
                                    td class="text-right" { (format!("{:.2} kg", current)) }
                                    td class="text-right text-red-600" { (format!("-{:.2} kg", change)) }
                                    td class="text-right" { (format!("{:.2} kg", new_val)) }
                                }
                            }
                        }
                    }
                }
            }

            div class="flex flex-row-reverse space-x-4 space-x-reverse" {
                button class="btn btn-cancel"
                    hx-post=(format!("/events/edit/shopping_tours/update_inventory/{}", tour_id))
                    hx-target="#content" {
                    "Apply Changes"
                }
                a class="btn btn-primary"
                    href=(format!("/events/edit/shopping_tours/export/{}/plain", tour_id))
                    target="_blank" {
                    "Review List in New Tab"
                }
                button class="btn btn-abort"
                    hx-get=(format!("/events/edit/shopping_tours/edit/{}/{}", tour.event_id, tour_id))
                    hx-target="#content" {
                    "Cancel"
                }
            }
        }
    })
}
