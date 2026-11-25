use super::{text::format_recipe_usage, TourExportData};
use bigdecimal::ToPrimitive;

use crate::types::amount::Amount;
use crate::{Error, Result};

pub fn render_shopping_tour(data: &TourExportData) -> Result<Vec<u8>> {
    let metro = data
        .metro
        .as_ref()
        .ok_or_else(|| Error::Misc("Missing Metro context for CSV export".into()))?;

    let empty_stock = std::collections::HashMap::new();
    let stock = data.inventory_stock.as_ref().unwrap_or(&empty_stock);

    let mut rows: Vec<(String, i64, String, bool)> = Vec::new();

    for item in &data.items {
        let Some(item_weight) = &item.weight else {
            continue;
        };

        let inv_amount = stock
            .get(&item.ingredient_id)
            .cloned()
            .unwrap_or_else(Amount::zero);

        if inv_amount >= *item_weight {
            continue;
        }
        let weight = item_weight.clone() - inv_amount.clone();

        // Prefer a source at the tour's store so its package size drives the
        // package count, but fall back to any source of the ingredient: some
        // ingredients are only sourced at other stores (e.g. Gemüse Schenk),
        // and a hard store filter would silently drop those rows.
        let source = metro
            .sources
            .iter()
            .find(|s| s.ingredient_id == item.ingredient_id && s.store_id == metro.store_id)
            .or_else(|| {
                metro
                    .sources
                    .iter()
                    .find(|s| s.ingredient_id == item.ingredient_id)
            });
        let Some(source) = source else {
            continue;
        };

        let Some(package_size) = source.package_size.to_f64().filter(|&s| s > 0.0) else {
            continue;
        };
        let weight_f64 = weight.to_f64().unwrap_or(0.0);
        let packages = (weight_f64 / package_size).ceil() as i64;
        let total_kg = packages as f64 * package_size;

        let mut comment = format!(
            "{}({} * {:.2}kg = {:.2}kg)",
            item.ingredient_name, packages, package_size, total_kg
        );
        let usage = format_recipe_usage(&item.recipe_usage);
        if !usage.is_empty() {
            comment.push_str(&format!(": {usage}"));
        }

        match metro.article_numbers.get(&item.ingredient_id) {
            Some(article) => rows.push((article.clone(), packages, comment, true)),
            None => rows.push((item.ingredient_name.clone(), packages, comment, false)),
        }
    }

    rows.sort_by_key(|(_, _, _, resolved)| !*resolved);

    let mut output = String::new();
    for (col1, packages, comment, _) in rows {
        let col1 = format!(
            "\"{}\"",
            col1.replace('"', "\"\"")
                .replace('\n', " ")
                .replace('\r', " ")
        );
        let comment_field = if comment.is_empty() {
            String::new()
        } else {
            format!("\"{}\"", comment.replace('"', "\"\""))
        };
        output.push_str(&format!("{},{},{}\n", col1, packages, comment_field));
    }

    Ok(output.into_bytes())
}
