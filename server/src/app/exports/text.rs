use std::collections::HashMap;

use bigdecimal::{BigDecimal, ToPrimitive};

use foodlib_new::entities::event::{ShoppingListItemWithUsage, ShoppingTour};
use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::IngredientId;

use super::TourExportData;

pub fn render_shopping_tour(data: &TourExportData) -> Vec<u8> {
    let text = match &data.inventory_stock {
        Some(stock) => {
            format_tour_text_with_inventory(&data.tour, &data.items, stock, data.show_prices)
        }
        None => format_tour_text(&data.tour, &data.items),
    };
    text.into_bytes()
}

fn format_tour_text(tour: &ShoppingTour, items: &[ShoppingListItemWithUsage]) -> String {
    let mut output = format!(
        "Einkaufsliste – {} – {}\n\n",
        tour.store_name.as_deref().unwrap_or("Tour"),
        format_tour_date(tour),
    );

    // `Option` tracks initialization separately from the (possibly empty)
    // category value, so the first uncategorized item still gets its
    // `=== Sonstiges ===` heading.
    let mut current_category: Option<String> = None;
    for item in items {
        let category = item.category.clone().unwrap_or_default();
        if current_category.as_deref() != Some(category.as_str()) {
            if current_category.is_some() {
                output.push('\n');
            }
            let label = if category.is_empty() {
                "Sonstiges"
            } else {
                &category
            };
            output.push_str(&format!("=== {label} ===\n"));
            current_category = Some(category);
        }
        let weight_str = item
            .weight
            .as_ref()
            .map(|w| format!("{w:.2} kg"))
            .unwrap_or_else(|| "? kg".to_string());
        output.push_str(&format!("[ ] {} – {weight_str}\n", item.ingredient_name));

        for (recipe, weight) in &item.recipe_usage {
            let part_str = weight
                .as_ref()
                .map(|w| format!("{w:.2} kg"))
                .unwrap_or_else(|| "? kg".to_string());
            output.push_str(&format!("      {recipe}: {part_str}\n"));
        }
    }
    output
}

fn format_tour_text_with_inventory(
    tour: &ShoppingTour,
    items: &[ShoppingListItemWithUsage],
    stock_by_ingredient: &HashMap<IngredientId, Amount>,
    show_prices: bool,
) -> String {
    let mut output = format!(
        "Einkaufsliste (inkl. Lagerabzug) – {} – {}\n\n",
        tour.store_name.as_deref().unwrap_or("Tour"),
        format_tour_date(tour),
    );
    let mut current_category = String::new();

    for item in items {
        let category = item.category.clone().unwrap_or_default();
        if category != current_category {
            if !current_category.is_empty() {
                output.push('\n');
            }
            let label = if category.is_empty() {
                "Sonstiges"
            } else {
                &category
            };
            output.push_str(&format!("=== {label} ===\n"));
            current_category = category;
        }

        let inv_amount = stock_by_ingredient
            .get(&item.ingredient_id)
            .cloned()
            .unwrap_or_else(Amount::zero);

        let (status, to_buy_text, weight_text, new_price) = match &item.weight {
            Some(weight) => {
                let (status, to_buy) = if inv_amount >= *weight {
                    ("x", Amount::zero())
                } else {
                    (" ", weight.clone() - inv_amount)
                };
                let weight_f64 = weight.to_f64().unwrap_or(0.0);
                let to_buy_f64 = to_buy.to_f64().unwrap_or(0.0);
                let new_price = if show_prices && weight_f64 > 0.0 {
                    item.price.as_ref().and_then(|p| {
                        let w_bd = BigDecimal::try_from(weight_f64).ok()?;
                        let b_bd = BigDecimal::try_from(to_buy_f64).ok()?;
                        Some((p.clone() / w_bd) * b_bd)
                    })
                } else {
                    None
                };
                (
                    status,
                    format!("{to_buy:.2} kg"),
                    format!("{weight:.2} kg"),
                    new_price,
                )
            }
            None => ("?", "? kg".to_string(), "? kg".to_string(), None),
        };

        let price_suffix = new_price
            .map(|price| format!(" (€{price:.2})"))
            .unwrap_or_default();

        output.push_str(&format!(
            "[{status}] {} - {to_buy_text}{price_suffix} -> {weight_text}",
            item.ingredient_name,
        ));

        let usage = format_recipe_usage(&item.recipe_usage);
        if !usage.is_empty() {
            output.push_str(&format!("  [{usage}]"));
        }
        output.push('\n');
    }

    output
}

/// Format a per-recipe usage breakdown as "3.00kg Recipe 2.00kg Other".
pub(crate) fn format_recipe_usage(usages: &[(String, Option<Amount>)]) -> String {
    usages
        .iter()
        .map(|(recipe, kg)| match kg {
            Some(amt) => format!("{amt:.2}kg {recipe}"),
            None => format!("?kg {recipe}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_tour_date(tour: &ShoppingTour) -> String {
    time::format_description::parse("[day].[month].[year]")
        .ok()
        .and_then(|fmt| tour.tour_date.format(&fmt).ok())
        .unwrap_or_else(|| tour.tour_date.to_string())
}