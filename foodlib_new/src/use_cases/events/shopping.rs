//! Shopping-tour use-cases.

use crate::entities::event::{EventInventory, ShoppingListItem, ShoppingTour};
use crate::entities::inventory::{InventoryItem, InventoryItemWithName};
use crate::entities::meal::Meal;
use crate::entities::{event, inventory, store};
use crate::types::amount::Amount;
use crate::types::ids::{EventId, IngredientId, InventoryId, MealId, ShoppingTourId, StoreId};
use crate::types::money::Money;
use crate::{entities, AppCtx, AppState, Result};
use time::OffsetDateTime;

/// Creates one shopping tour for an event.
/// Requires `EditEvent` permission.
pub async fn create_tour(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    store_id: StoreId,
    tour_date: OffsetDateTime,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    state
        .db()
        .events()
        .add_shopping_tour(ShoppingTour {
            id: ShoppingTourId(-1),
            event_id,
            tour_date,
            store_id,
            store_name: None,
        })
        .await?;
    Ok(())
}

/// Saves one shopping tour: updates it when `tour_id` is given, otherwise
/// creates it. The tour's own `EditShoppingTour` permission is asserted for
/// existing tours.
/// Requires `EditEvent` permission.
pub async fn save_tour(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: Option<ShoppingTourId>,
    store_id: StoreId,
    tour_date: OffsetDateTime,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    if let Some(tour_id) = tour_id {
        actor.assert_can_edit_shopping_tour(tour_id).await?;
    }

    match tour_id {
        Some(id) => {
            state
                .db()
                .events()
                .update_shopping_tour(ShoppingTour {
                    id,
                    event_id,
                    tour_date,
                    store_id,
                    store_name: None,
                })
                .await?;
        }
        None => {
            state
                .db()
                .events()
                .add_shopping_tour(ShoppingTour {
                    id: ShoppingTourId(-1),
                    event_id,
                    tour_date,
                    store_id,
                    store_name: None,
                })
                .await?;
        }
    }
    Ok(())
}

/// Deletes one shopping tour.
/// Requires `EditShoppingTour` permission.
pub async fn delete_tour(state: &AppState, actor: &AppCtx, tour_id: ShoppingTourId) -> Result<()> {
    actor.assert_can_edit_shopping_tour(tour_id).await?;
    state.db().events().delete_shopping_tour(tour_id).await?;
    Ok(())
}

/// Attaches (`checked`) or detaches an inventory from an event's shopping.
/// Requires `EditEvent` permission.
pub async fn set_event_inventory(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    inventory_id: InventoryId,
    checked: bool,
) -> Result<()> {
    actor.assert_can_edit_event(event_id).await?;
    if checked {
        state.db().events().add_inventory(event_id, inventory_id).await?;
    } else {
        state
            .db()
            .events()
            .remove_inventory(event_id, inventory_id)
            .await?;
    }
    Ok(())
}

/// A planned deduction of one ingredient from one inventory, computed from a
/// tour's shopping list and the event's attached inventories:
/// `(inventory_name, inventory_id, ingredient_name, ingredient_id, current_amount, to_subtract, new_amount)`.
pub type InventoryChange = (
    String,
    InventoryId,
    String,
    IngredientId,
    Amount,
    Amount,
    Amount,
);

/// How much can actually be subtracted: never more than is available, and
/// never more than the remaining need.
fn calculate_subtraction(available: &Amount, remaining: &Amount) -> Amount {
    if available >= remaining {
        remaining.clone()
    } else {
        available.clone()
    }
}

/// Calculate inventory changes showing what will be deducted from each
/// inventory to cover the tour's shopping list. Inventories are drained in
/// the order the event attached them.
/// Requires `EditShoppingTour` permission.
pub async fn calculate_inventory_changes(
    state: &AppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<Vec<InventoryChange>> {
    actor.assert_can_edit_shopping_tour(tour_id).await?;
    let tour = state.db().events().get_shopping_tour(tour_id).await?;
    let shopping_list = state.db().events().get_shopping_list(tour_id).await?;
    let inventories = state.db().events().get_inventories(tour.event_id).await?;
    let inventory_ops = state.db().inventories();
    let inventories = inventories
        .iter()
        .map(|inv| inv.inventory_id)
        .map(|id| inventory_ops.get(id));
    let inventories = futures::future::try_join_all(inventories).await?;

    let mut changes = Vec::new();

    // For each item in the shopping list
    for item in &shopping_list {
        // Unresolvable weight (unknown unit weight) → nothing to deduct.
        let Some(initial_weight) = item.weight.clone() else {
            continue;
        };
        let mut remaining = initial_weight;

        // Try to satisfy from each inventory in order
        for inv in &inventories {
            let inv = inv.clone();
            if remaining <= Amount::default() {
                break;
            }

            if let Ok(items) = state.db().inventories().get_items(inv.id).await {
                if let Some(current_item) =
                    items.iter().find(|i| i.ingredient_id == item.ingredient_id)
                {
                    let to_subtract = calculate_subtraction(&current_item.amount, &remaining);

                    if to_subtract > Amount::default() {
                        let new_amount = current_item.amount.clone() - to_subtract.clone();
                        changes.push((
                            inv.name.clone(),
                            inv.id,
                            item.ingredient_name.clone(),
                            item.ingredient_id,
                            current_item.amount.clone(),
                            to_subtract.clone(),
                            new_amount,
                        ));
                        remaining -= to_subtract;
                    }
                }
            }
        }
    }

    Ok(changes)
}

/// Applies the planned inventory deductions of a tour: for every computed
/// change the inventory item is overwritten with its new amount.
/// Requires `EditShoppingTour` permission.
pub async fn apply_inventory_changes(
    state: &AppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<()> {
    actor.assert_can_edit_shopping_tour(tour_id).await?;
    let changes = calculate_inventory_changes(state, actor, tour_id).await?;
    for (_, inventory_id, _, ingredient_id, _, _, amount) in changes {
        state
            .db()
            .inventories()
            .update_item(InventoryItem {
                inventory_id,
                ingredient_id,
                amount,
            })
            .await?;
    }
    Ok(())
}

/// Everything the shopping-tour add/edit form renders, fetched in one use
/// case. The tour's own `EditShoppingTour` permission is asserted for
/// existing tours. `shopping_list` is empty for new tours.
/// Requires `EditEvent` permission.
pub struct TourFormData {
    pub stores: Vec<store::Store>,
    pub inventories: Vec<inventory::Inventory>,
    pub existing: Option<ShoppingTour>,
    pub event_inventories: Vec<EventInventory>,
    pub shopping_list: Vec<ShoppingListItem>,
}

pub async fn gather_tour_form(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: Option<ShoppingTourId>,
) -> Result<TourFormData> {
    actor.assert_can_edit_event(event_id).await?;
    if let Some(id) = tour_id {
        actor.assert_can_edit_shopping_tour(id).await?;
    }

    let stores = state.db().stores().list().await?;
    let inventories = state.db().inventories().list().await?;
    let (existing, event_inventories, shopping_list) = match tour_id {
        Some(id) => (
            Some(state.db().events().get_shopping_tour(id).await?),
            state.db().events().get_inventories(event_id).await?,
            state.db().events().get_shopping_list(id).await?,
        ),
        None => (None, vec![], vec![]),
    };

    Ok(TourFormData {
        stores,
        inventories,
        existing,
        event_inventories,
        shopping_list,
    })
}

/// All inventory items of the event's attached inventories (lenient: an
/// inventory that fails to read contributes no items). Used to annotate the
/// shopping list with availability.
pub async fn gather_inventory_items(
    state: &AppState,
    event_inventories: &[EventInventory],
) -> Vec<InventoryItemWithName> {
    let mut all_items = vec![];
    for inv in event_inventories {
        if let Ok(items) = state.db().inventories().get_items(inv.inventory_id).await {
            all_items.extend(items);
        }
    }
    all_items
}

/// Fetches one shopping tour.
pub async fn get_shopping_tour(state: &AppState, tour_id: ShoppingTourId) -> Result<ShoppingTour> {
    state.db().events().get_shopping_tour(tour_id).await
}

/// Reads for the tour text export: the shopping list with per-recipe usage
/// breakdown, the tour, and the event's attached inventories.
/// Requires `EditShoppingTour` permission.
pub struct TourExportReads {
    pub shopping_list: Vec<event::ShoppingListItemWithUsage>,
    pub tour: ShoppingTour,
    pub event_inventories: Vec<EventInventory>,
}

pub async fn gather_tour_export(
    state: &AppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<TourExportReads> {
    actor.assert_can_edit_shopping_tour(tour_id).await?;
    let shopping_list = state
        .db()
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await?;
    let tour = state.db().events().get_shopping_tour(tour_id).await?;
    let event_inventories = state.db().events().get_inventories(tour.event_id).await?;
    Ok(TourExportReads {
        shopping_list,
        tour,
        event_inventories,
    })
}

/// Reads for the Metro CSV export.
/// Requires `EditShoppingTour` permission.
pub struct MetroExportReads {
    pub shopping_list: Vec<event::ShoppingListItemWithUsage>,
    pub sources: Vec<entities::ingredient::IngredientSource>,
    pub tour: ShoppingTour,
    pub event_inventories: Vec<EventInventory>,
    pub article_numbers: std::collections::HashMap<IngredientId, String>,
}

pub async fn gather_metro_export(
    state: &AppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<MetroExportReads> {
    actor.assert_can_edit_shopping_tour(tour_id).await?;
    let shopping_list = state
        .db()
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await?;
    let sources = state.db().ingredients().list_sources(None).await?;
    let tour = state.db().events().get_shopping_tour(tour_id).await?;
    let event_inventories = state.db().events().get_inventories(tour.event_id).await?;
    let article_numbers = state
        .db()
        .events()
        .resolve_metro_article_numbers(tour_id)
        .await?;
    Ok(MetroExportReads {
        shopping_list,
        sources,
        tour,
        event_inventories,
        article_numbers,
    })
}

/// Everything the per-meal price OOB swap renders.
/// Requires `EditEvent` permission.
pub struct MealPricesData {
    pub meals: Vec<Meal>,
    pub price_by_meal: std::collections::HashMap<MealId, Amount>,
    pub budget: Option<Money>,
}

/// Collects all data needed to render the per-meal price cells.
pub async fn gather_meal_prices(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<MealPricesData> {
    actor.assert_can_edit_event(event_id).await?;
    let meals = state.db().meals().get_event_meals(event_id).await?;
    let prices = state.db().meals().get_event_meal_prices(event_id).await?;
    let price_by_meal: std::collections::HashMap<MealId, Amount> = prices.into_iter().collect();
    let budget = state.db().events().get(event_id).await?.budget;

    Ok(MealPricesData {
        meals,
        price_by_meal,
        budget,
    })
}
