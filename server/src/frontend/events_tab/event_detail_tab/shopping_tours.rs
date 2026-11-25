use crate::{
    frontend::{events_tab::event_detail_tab, web_error::WebError, IResponse, MResponse, WebCtx},
    AppState,
};
use axum::{
    extract::{Form, Path, State},
    response::IntoResponse,
    routing::{delete, get, post},
};
use foodlib_new::entities::event::ShoppingListItem;
use foodlib_new::entities::inventory::InventoryItemWithName;
use foodlib_new::error::Result;
use foodlib_new::types::amount::Amount;
use foodlib_new::types::ids::{EventId, IngredientId, InventoryId, ShoppingTourId, StoreId};
use foodlib_new::AppCtx;
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

pub(crate) fn shopping_tour_router() -> axum::Router<AppState> {
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
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
) -> MResponse {
    render_shopping_tour_form(&state, &actor, event_id, None).await
}

async fn edit_shopping_tour(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((event_id, tour_id)): Path<(EventId, ShoppingTourId)>,
) -> MResponse {
    render_shopping_tour_form(&state, &actor, event_id, Some(tour_id)).await
}

#[derive(Deserialize)]
struct InlineTourForm {
    store_id: StoreId,
    date: String,
}

async fn create_inline(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(event_id): Path<EventId>,
    Form(form): Form<InlineTourForm>,
) -> MResponse {
    let tour_date = parse_datetime_local(&form.date)?;
    foodlib_new::events::shopping::create_tour(&state, &actor, event_id, form.store_id, tour_date)
        .await?;
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
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> MResponse {
    foodlib_new::events::shopping::delete_tour(&state, &actor, tour_id).await?;
    Ok(html! {})
}

async fn render_shopping_tour_form(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: Option<ShoppingTourId>,
) -> MResponse {
    let foodlib_new::events::shopping::TourFormData {
        stores,
        inventories,
        existing,
        event_inventories,
        shopping_list,
    } = foodlib_new::events::shopping::gather_tour_form(state, actor, event_id, tour_id).await?;

    let selected_store_id = existing.as_ref().map(|t| t.store_id);
    let initial_date = existing
        .as_ref()
        .map(|t| t.tour_date)
        .unwrap_or_else(OffsetDateTime::now_utc);

    let inventory_items =
        foodlib_new::events::shopping::gather_inventory_items(state, &event_inventories).await;

    Ok(html! {
        div class="flex-col space-y-4 w-full" {
            h2 class="text-xl" {
                @if tour_id.is_some() {
                    "Edit Shopping Tour"
                } @else {
                    "Add Shopping Tour"
                }
            }

            div class="flex flex-col lg:flex-row gap-4" {
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

                @if let (Some(id), false) = (tour_id, shopping_list.is_empty()) {
                    div class="w-full lg:w-1/2 space-y-4" {
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

async fn save_tour(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Form(form): Form<TourForm>,
) -> MResponse {
    let tour_date = parse_datetime_local(&form.date)?;

    let result = foodlib_new::events::shopping::save_tour(
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
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> IResponse {
    let file = foodlib_new::exports::render_shopping_tour_txt(&state, &actor, tour_id).await?;
    Ok(IntoResponse::into_response((
        [
            (
                axum::http::header::CONTENT_TYPE,
                format!("{}; charset=utf-8", file.mime),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", file.filename),
            ),
        ],
        file.bytes,
    )))
}

async fn export_metro(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> IResponse {
    let file = foodlib_new::exports::render_metro_shopping_tour_csv(&state, &actor, tour_id).await?;

    Ok(IntoResponse::into_response((
        [
            (
                axum::http::header::CONTENT_TYPE,
                format!("{}; charset=utf-8", file.mime),
            ),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", file.filename),
            ),
        ],
        file.bytes,
    )))
}

async fn toggle_inventory(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path((event_id, inventory_id)): Path<(EventId, InventoryId)>,
    Form(form): Form<ToggleForm>,
) -> MResponse {
    foodlib_new::events::shopping::set_event_inventory(
        &state,
        &actor,
        event_id,
        inventory_id,
        form.checked,
    )
        .await?;

    render_shopping_tour_form(&state, &actor, event_id, Some(form.tour_id)).await
}

async fn update_inventory(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> MResponse {
    foodlib_new::events::shopping::apply_inventory_changes(&state, &actor, tour_id).await?;
    let tour = foodlib_new::events::shopping::get_shopping_tour(&state, tour_id).await?;

    event_detail_tab::event_form(State(state), WebCtx(actor), Path(tour.event_id)).await
}

async fn confirm_update(
    State(state): State<AppState>,
    WebCtx(actor): WebCtx,
    Path(tour_id): Path<ShoppingTourId>,
) -> std::result::Result<Markup, WebError> {
    let changes =
        foodlib_new::events::shopping::calculate_inventory_changes(&state, &actor, tour_id)
            .await?;

    let tour = foodlib_new::events::shopping::get_shopping_tour(&state, tour_id).await?;

    Ok(html! {
        div class="flex flex-col space-y-4 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg" {
            h3 class="text-xl text-red-600 dark:text-red-400" { "Warning!" }
            p class="text-gray-700 dark:text-gray-300 mb-4" {
                "This will subtract the listed amounts from your inventories. This action cannot be undone."
            }

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
