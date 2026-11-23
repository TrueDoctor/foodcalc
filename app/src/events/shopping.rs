//! Shopping-tour use-cases.

use crate::app_ctx::AppCtx;
use crate::{MyAppState, Result};
use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{
    EventId, IngredientId, InventoryId, ShoppingTourId, StoreId,
};
use foodlib_new::event::ShoppingTour;
use foodlib_new::inventory::InventoryItem;
use foodlib_new::meal::Meal;
use foodlib_new::types::ids::MealId;
use foodlib_new::types::money::Money;
use time::OffsetDateTime;

/// Creates one shopping tour for an event.
/// Requires `EditEvent` permission.
pub async fn create_tour(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    store_id: StoreId,
    tour_date: OffsetDateTime,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    state
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
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: Option<ShoppingTourId>,
    store_id: StoreId,
    tour_date: OffsetDateTime,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    if let Some(tour_id) = tour_id {
        actor.ctx.assert_can_edit_shopping_tour(tour_id).await?;
    }

    match tour_id {
        Some(id) => {
            state
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
pub async fn delete_tour(
    state: &MyAppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<()> {
    actor.ctx.assert_can_edit_shopping_tour(tour_id).await?;
    state.events().delete_shopping_tour(tour_id).await?;
    Ok(())
}

/// Attaches (`checked`) or detaches an inventory from an event's shopping.
/// Requires `EditEvent` permission.
pub async fn set_event_inventory(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    inventory_id: InventoryId,
    checked: bool,
) -> Result<()> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    if checked {
        state.events().add_inventory(event_id, inventory_id).await?;
    } else {
        state
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
pub async fn calculate_inventory_changes(
    state: &MyAppState,
    tour_id: ShoppingTourId,
) -> Result<Vec<InventoryChange>> {
    let tour = state.events().get_shopping_tour(tour_id).await?;
    let shopping_list = state.events().get_shopping_list(tour_id).await?;
    let inventories = state.events().get_inventories(tour.event_id).await?;
    let inventory_ops = state.inventories();
    let inventories = inventories
        .iter()
        .map(|inv| inv.inventory_id)
        .map(|id| inventory_ops.get(id));
    let inventories = ::futures::future::join_all(inventories).await;

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
            let inv = inv.as_ref().unwrap().clone();
            if remaining <= Amount::default() {
                break;
            }

            if let Ok(items) = state.inventories().get_items(inv.id).await {
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
    state: &MyAppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<()> {
    actor.ctx.assert_can_edit_shopping_tour(tour_id).await?;
    let changes = calculate_inventory_changes(state, tour_id).await?;
    for (_, inventory_id, _, ingredient_id, _, _, amount) in changes {
        state
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

/// Everything the per-meal price OOB swap renders.
/// Requires `EditEvent` permission.
pub struct MealPricesData {
    pub meals: Vec<Meal>,
    pub price_by_meal: std::collections::HashMap<MealId, Amount>,
    pub budget: Option<Money>,
}

/// Collects all data needed to render the per-meal price cells.
pub async fn gather_meal_prices(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<MealPricesData> {
    actor.ctx.assert_can_edit_event(event_id).await?;
    let meals = state.meals().get_event_meals(event_id).await?;
    let prices = state.meals().get_event_meal_prices(event_id).await?;
    let price_by_meal: std::collections::HashMap<MealId, Amount> =
        prices.into_iter().collect();
    let budget = state.events().get(event_id).await?.budget;

    Ok(MealPricesData {
        meals,
        price_by_meal,
        budget,
    })
}