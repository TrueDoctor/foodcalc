// shopping_tours.rs
use axum::{
    extract::{Form, Path, State},
    response::IntoResponse,
    routing::{delete, get, post},
};
use bigdecimal::BigDecimal;
use foodlib_new::error::{Error, Result};
use foodlib_new::{
    event::{EventInventory, ShoppingListItem, ShoppingTour},
    inventory::InventoryItem,
};
use maud::{html, Markup};
use num::ToPrimitive;
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

use crate::{frontend::events_tab::event_detail_tab, MyAppState};

pub(crate) fn shopping_tour_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/add/:event_id", get(add_shopping_tour))
        .route("/edit/:event_id/:tour_id", get(shopping_tour_form))
        .route("/save", post(save_tour))
        // .route("/delete/:event_id/:tour_id", delete(delete_tour))
        .route("/export/:tour_id/plain", get(export_plain))
        .route("/export/:tour_id/metro", get(export_metro))
        .route("/update_inventory/:tour_id", post(update_inventory))
        .route("/confirm_update/:tour_id", get(confirm_update))
}

#[derive(Deserialize)]
struct TourForm {
    event_id: i32,
    tour_id: Option<i32>,
    store_id: i32,
    date: String,
    inventories: Vec<i32>,
}
async fn add_shopping_tour(state: State<MyAppState>, Path(event_id): Path<i32>) -> Markup {
    shopping_tour_form(state, Path((event_id, -1))).await
}

async fn shopping_tour_form(
    State(state): State<MyAppState>,
    Path((event_id, tour_id)): Path<(i32, i32)>,
) -> Markup {
    let stores = state.get_stores().await.unwrap_or_default();
    let inventories = state
        .new_lib()
        .inventories()
        .get_all_inventories()
        .await
        .unwrap_or_default();

    let default_tour = ShoppingTour {
        event_id,
        id: -1,
        tour_date: OffsetDateTime::now_utc(),
        store_id: 0,
    };

    // If tour_id is provided, get existing tour data
    let (tour, event_inventories, shopping_list) = if tour_id > 0 {
        let tours = state
            .new_lib()
            .events()
            .get_shopping_tours(tour_id)
            .await
            .unwrap_or_default();
        let tour = tours.first().cloned().unwrap_or(default_tour);

        (
            tour,
            state
                .new_lib()
                .events()
                .get_inventories(event_id)
                .await
                .unwrap_or_default(),
            state
                .new_lib()
                .events()
                .get_shopping_list(tour_id)
                .await
                .unwrap_or_default(),
        )
    } else {
        (default_tour, vec![], vec![])
    };
    dbg!(&shopping_list);
    let tour_id = if tour_id < 0 { None } else { Some(tour_id) };

    let inventory_items = if tour_id.is_some() {
        // Get all inventory items for the event
        let mut all_items = vec![];
        for inv in &event_inventories {
            if let Ok(items) = state
                .new_lib()
                .inventories()
                .get_inventory_items(inv.inventory_id)
                .await
            {
                all_items.extend(items);
            }
        }
        all_items
    } else {
        vec![]
    };

    html! {
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
                    input type="hidden" name="tour_id" value=(tour_id.unwrap_or(-1));

                    div class="flex flex-col space-y-2" {
                        label for="store_id" { "Store" }
                        select name="store_id" class="text" required {
                            @for store in stores {
                                option value=(store.store_id)
                                    selected[store.store_id == tour.store_id] {
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
                            value=(tour_id.map(|_| tour.tour_date
                                .format(format_description!("[year]-[month]-[day]T[hour]:[minute]"))
                                .unwrap())
                                .unwrap_or_default())
                            required;
                    }

                    div class="flex flex-col space-y-2" {
                        label { "Use Inventories" }
                        div class="space-y-1" {
                            @for inventory in inventories {
                                div class="flex items-center gap-2" {
                                    input type="checkbox"
                                        class="w-4 h-4"
                                        name="inventories"
                                        value=(inventory.id)
                                        checked[event_inventories.iter().any(|inv| inv.inventory_id == inventory.id)];
                                    span { (inventory.name) }
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
                @if !shopping_list.is_empty() {
                    div class="w-full lg:w-1/2 space-y-4" {
                        // Action buttons in a grid on mobile, row on desktop
                        div class="grid grid-cols-2 lg:flex gap-2 mb-4" {
                            a class="btn btn-primary text-center"
                                href=(format!("/events/edit/shopping_tours/export/{}/plain", tour_id.unwrap()))
                                target="_blank" {
                                "Export Plain"
                            }
                            a class="btn btn-primary text-center"
                                href=(format!("/events/edit/shopping_tours/export/{}/metro", tour_id.unwrap()))
                                target="_blank" {
                                "Export Metro"
                            }
                            button class="btn btn-cancel col-span-2 lg:col-span-1"
                                hx-get=(format!("/events/edit/shopping_tours/confirm_update/{}", tour_id.unwrap()))
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
                                            td class="text-right" { (format!("{:.2} kg", item.weight)) }
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
    }
}

async fn save_tour(
    State(state): State<MyAppState>,
    Form(form): Form<TourForm>,
) -> impl IntoResponse {
    // Parse the date
    let tour_date = time::OffsetDateTime::parse(
        &format!("{}:00Z", form.date),
        format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:second[offset_hour sign:mandatory]"
        ),
    )
    .unwrap();

    // Create or update tour
    match form.tour_id {
        Some(tour_id) if tour_id > 0 => {
            state
                .new_lib()
                .events()
                .update_shopping_tour(foodlib_new::event::ShoppingTour {
                    id: tour_id,
                    event_id: form.event_id,
                    tour_date,
                    store_id: form.store_id,
                })
                .await
                .ok();
        }
        _ => {
            state
                .new_lib()
                .events()
                .add_shopping_tour(foodlib_new::event::ShoppingTour {
                    id: -1,
                    event_id: form.event_id,
                    tour_date,
                    store_id: form.store_id,
                })
                .await
                .ok();
        }
    }

    // Redirect back to event edit page
    html! {
        (event_detail_tab::event_form(State(state), Path(form.event_id)).await.unwrap_or_default())
    }
}

fn get_row_class(item: &ShoppingListItem, inventory: &[InventoryItem]) -> &'static str {
    let inv_amount = inventory
        .iter()
        .find(|i| i.ingredient_id == item.ingredient_id)
        .map(|i| i.amount.clone())
        .unwrap_or_default();

    if inv_amount >= item.weight {
        "bg-green-100 dark:bg-green-900"
    } else if inv_amount > BigDecimal::from(0) {
        "bg-red-100 dark:bg-red-900"
    } else {
        ""
    }
}

fn get_inventory_status(item: &ShoppingListItem, inventory: &[InventoryItem]) -> String {
    let inv_amount = inventory
        .iter()
        .find(|i| i.ingredient_id == item.ingredient_id)
        .map(|i| i.amount.clone())
        .unwrap_or_default();

    if inv_amount >= item.weight {
        "Available".to_string()
    } else if inv_amount > BigDecimal::from(0) {
        format!("Insufficient ({:.2} kg available)", inv_amount)
    } else {
        "Not in inventory".to_string()
    }
}

async fn confirm_update(Path(tour_id): Path<i32>) -> Markup {
    html! {
        div class="flex flex-col space-y-4 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg" {
            h3 class="text-xl text-red-600 dark:text-red-400" { "Warning!" }
            p class="text-gray-700 dark:text-gray-300" {
                "This will subtract the listed amounts from your inventories. This action cannot be undone."
            }
            div class="flex space-x-2 justify-end" {
                button class="btn btn-primary"
                    hx-get=(format!("/events/edit/shopping_tours/export/{}/plain", tour_id))
                    hx-target="this"
                    target="_blank" {
                    "Review List First"
                }
                button class="btn btn-cancel"
                    hx-post=(format!("/events/edit/shopping_tours/update_inventory/{}", tour_id))
                    hx-target="#content" {
                    "Apply Changes"
                }
                button class="btn btn-abort"
                    onclick="this.closest('div.flex.flex-col').remove()" {
                    "Cancel"
                }
            }
        }
    }
}

async fn export_plain(
    State(state): State<MyAppState>,
    Path(tour_id): Path<i32>,
) -> impl IntoResponse {
    let shopping_list = state
        .new_lib()
        .events()
        .get_shopping_list(tour_id)
        .await
        .unwrap_or_default();

    let mut output = String::new();
    let mut current_category = String::new();

    for item in shopping_list {
        let category = item.category.unwrap_or_default();
        if category != current_category {
            if !current_category.is_empty() {
                output.push('\n');
            }
            output.push_str(&format!("=== {} ===\n", category));
            current_category = category;
        }

        output.push_str(&format!(
            "[ ] {} - {:.2} kg{}",
            item.ingredient_name,
            item.weight,
            if let Some(price) = item.price {
                format!(" (€{:.2})", price)
            } else {
                String::new()
            }
        ));
        output.push('\n');
    }
    (
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
    )
}

async fn export_metro(
    State(state): State<MyAppState>,
    Path(tour_id): Path<i32>,
) -> impl IntoResponse {
    let shopping_list = state
        .new_lib()
        .events()
        .get_shopping_list(tour_id)
        .await
        .unwrap_or_default();
    let sources = state.get_ingredient_sources(None).await.unwrap_or_default();

    let mut output = String::new();
    output.push_str("Name,Amount,URL\n");

    for item in shopping_list {
        // Find source with matching ingredient_id
        if let Some(source) = sources
            .iter()
            .find(|s| s.ingredient_id == item.ingredient_id)
        {
            // Round up to next package size
            let packages =
                (item.weight.to_f64().unwrap() / source.package_size.to_f64().unwrap()).ceil();
            let amount = packages * source.package_size.to_f64().unwrap();

            output.push_str(&format!(
                "{},{:.2},{}\n",
                item.ingredient_name,
                amount,
                source.url.as_deref().unwrap_or("")
            ));
        }
    }

    (
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
    )
}

/// Calculates how much to subtract from a specific inventory for a given ingredient
fn calculate_subtraction(available: &BigDecimal, remaining: &BigDecimal) -> BigDecimal {
    if available >= remaining {
        remaining.clone()
    } else {
        available.clone()
    }
}

/// Updates a single inventory item and returns how much was actually subtracted
async fn update_single_inventory(
    ops: &foodlib_new::FoodLib,
    inventory_id: i32,
    ingredient_id: i32,
    to_subtract: BigDecimal,
) -> Result<BigDecimal> {
    let items = ops.inventories().get_inventory_items(inventory_id).await?;

    if let Some(current_item) = items.iter().find(|i| i.ingredient_id == ingredient_id) {
        let new_amount = current_item.amount.clone() - to_subtract.clone();

        ops.inventories()
            .update_inventory_item(InventoryItem {
                inventory_id,
                ingredient_id,
                amount: new_amount,
            })
            .await?;

        Ok(to_subtract)
    } else {
        Ok(BigDecimal::from(0))
    }
}

/// Processes a single shopping list item across multiple inventories
async fn process_shopping_list_item(
    ops: &foodlib_new::FoodLib,
    item: &ShoppingListItem,
    inventories: &[EventInventory],
) -> Result<()> {
    let mut remaining = item.weight.clone();

    for inv in inventories {
        // Skip if we've already covered the full amount
        if remaining <= BigDecimal::from(0) {
            break;
        }

        // Try to update this inventory and track what was subtracted
        if let Ok(items) = ops
            .inventories()
            .get_inventory_items(inv.inventory_id)
            .await
        {
            if let Some(current_item) = items.iter().find(|i| i.ingredient_id == item.ingredient_id)
            {
                let to_subtract = calculate_subtraction(&current_item.amount, &remaining);

                if to_subtract > BigDecimal::from(0) {
                    if let Ok(subtracted) = update_single_inventory(
                        ops,
                        inv.inventory_id,
                        item.ingredient_id,
                        to_subtract,
                    )
                    .await
                    {
                        remaining -= subtracted;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn update_inventory(State(state): State<MyAppState>, Path(tour_id): Path<i32>) -> Markup {
    let lib = state.new_lib();

    // Get tour and associated data
    let result = async {
        let tours = lib.events().get_shopping_tours(tour_id).await?;
        let tour = tours.first().ok_or(Error::NotFound {
            entity: "ShoppingTour",
            id: tour_id.to_string(),
        })?;

        let shopping_list = lib.events().get_shopping_list(tour_id).await?;
        let inventories = lib.events().get_inventories(tour.event_id).await?;

        // Process each shopping list item
        for item in shopping_list {
            process_shopping_list_item(lib, &item, &inventories).await?;
        }

        Ok::<_, Error>(tour.event_id)
    }
    .await;

    match result {
        Ok(event_id) => {
            // Redirect back to event page
            event_detail_tab::event_form(State(state), Path(event_id))
                .await
                .unwrap_or_default()
        }
        Err(e) => {
            html! {
                div class="error" {
                    ("Failed to update inventory: ") (e.to_string())
                }
            }
        }
    }
}
