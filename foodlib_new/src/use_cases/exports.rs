//! Shared document rendering engines (Typst PDF singleton, Plain Text, Metro CSV).

use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use crate::ctx::AppCtx;
use crate::entities::event::{ShoppingListItemWithUsage, ShoppingTour};
use crate::entities::ingredient::IngredientSource;
use crate::entities::recipe::SubRecipe;
use crate::types::amount::Amount;
use crate::types::ids::{
    EventId, FoodPrepId, IngredientId, MealId, RecipeId, ShoppingTourId, StoreId,
};
use crate::{AppState, Error, Result};

// Re-export so `PdfExporter` and `fetch_*` signatures don't leak `pub(crate) mod ops`.
pub use crate::ops::export::{EventAllergenInfo, EventOverviewInfo, RecipeInfo};

pub mod csv;
pub mod text;
mod no_renderer;
/// Finished export artifact returned by domain export use-cases.
pub struct ExportedFile {
    pub filename: String,
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

/// Output format requested by the API caller (`?format=pdf|txt|csv`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    #[default]
    #[serde(alias = "text")]
    Txt,
    Pdf,
    Csv,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DishLabelLayout {
    #[default]
    Flat,
    Tent,
}

/// Prepared shopping-tour dataset passed into `text`, `csv`, or `PdfExporter`.
pub struct TourExportData {
    pub tour: ShoppingTour,
    pub items: Vec<ShoppingListItemWithUsage>,
    /// `Some(map)` when `deduct_inventory` is enabled; `None` for gross export.
    pub inventory_stock: Option<HashMap<IngredientId, Amount>>,
    pub show_prices: bool,
    /// Metro-specific metadata (populated when `format == ExportFormat::Csv`).
    pub metro: Option<MetroExportContext>,
}

pub struct MetroExportContext {
    pub sources: Vec<IngredientSource>,
    pub article_numbers: HashMap<IngredientId, String>,
    /// The store the sources belong to (currently always the Metro store).
    pub store_id: StoreId,
}

/// Polymorphic PDF rendering backend stored once on [`AppState`].
pub trait PdfExporter: Send + Sync {
    fn export_recipes(&self, _infos: Vec<RecipeInfo>) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "PDF export for recipes is not implemented".into(),
        ))
    }

    fn export_event_allergens(&self, _info: EventAllergenInfo) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "PDF export for event allergens is not implemented".into(),
        ))
    }

    fn export_dish_labels(
        &self,
        _info: EventAllergenInfo,
        _layout: DishLabelLayout,
    ) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "PDF export for dish labels is not implemented".into(),
        ))
    }

    fn export_event_overview(&self, _info: EventOverviewInfo) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "PDF export for event overview is not implemented".into(),
        ))
    }

    fn export_shopping_tour(&self, _tour: &TourExportData) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "PDF export for shopping tours is not implemented".into(),
        ))
    }
}

/// Instantiates the singleton PDF renderer once at server startup.
pub fn default_pdf_exporter() -> Arc<dyn PdfExporter> {
    Arc::new(no_renderer::NoRenderer)
}

/// Runs a Typst PDF render job on Tokio's blocking pool using the singleton engine.
pub async fn render_pdf<F>(state: &AppState, filename: String, job: F) -> Result<ExportedFile>
where
    F: FnOnce(&dyn PdfExporter) -> Result<Vec<u8>> + Send + 'static,
{
    let pdf = state.pdf();
    let bytes = tokio::task::spawn_blocking(move || job(&*pdf))
        .await
        .map_err(|e| Error::Misc(format!("PDF render task failed: {e}")))??;

    Ok(ExportedFile {
        filename,
        mime: "application/pdf",
        bytes,
    })
}

/// Filesystem-safe filename fragment.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

// Thin read wrappers over the foodlib export ops so transports never touch
// the lib directly; all of these feed the `PdfExporter` or text/CSV writers.

pub async fn fetch_meal_recipe(
    state: &AppState,
    actor: &AppCtx,
    meal_id: MealId,
) -> Result<RecipeInfo> {
    actor.assert_can_edit_meal(meal_id).await?;
    state.db().export().fetch_meal_recipe(meal_id).await
}

pub async fn fetch_food_prep_recipe(
    state: &AppState,
    actor: &AppCtx,
    prep_id: FoodPrepId,
) -> Result<RecipeInfo> {
    actor.assert_can_edit_food_prep(prep_id).await?;
    state.db().export().fetch_food_prep_recipe(prep_id).await
}

pub async fn fetch_user_input_meal(
    state: &AppState,
    recipe_id: RecipeId,
    servings: f64,
    energy_kj: u32,
) -> Result<RecipeInfo> {
    state
        .db()
        .export()
        .fetch_user_input_meal(recipe_id, servings, energy_kj, String::new())
        .await
}

pub async fn fetch_subrecipes_from_user_input(
    state: &AppState,
    recipe_id: RecipeId,
    servings: f64,
    energy_kj: u32,
) -> Result<Vec<SubRecipe>> {
    state
        .db()
        .export()
        .fetch_subrecipes_from_user_input(recipe_id, servings, energy_kj)
        .await
}

pub async fn fetch_event_allergens(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<EventAllergenInfo> {
    actor.assert_can_edit_event(event_id).await?;
    state.db().export().fetch_event_allergens(event_id).await
}

pub async fn fetch_event_combined(
    state: &AppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<Vec<RecipeInfo>> {
    actor.assert_can_edit_event(event_id).await?;
    state.db().export().fetch_event_combined(event_id).await
}

/// Renders a shopping tour as a Metro order CSV: whole package counts per
/// article (resolved via the Metro sync cache), minus what the event's
/// attached inventories already cover. Requires `EditShoppingTour` permission.
pub async fn render_metro_shopping_tour_csv(
    state: &AppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<ExportedFile> {
    use crate::events::shopping::{gather_inventory_items, gather_metro_export};

    let metro_reads = gather_metro_export(state, actor, tour_id).await?;
    let inventory_items = gather_inventory_items(state, &metro_reads.event_inventories).await;

    // Sum the stock per ingredient across all attached inventories.
    let mut inventory_stock: HashMap<IngredientId, Amount> = HashMap::new();
    for item in inventory_items {
        *inventory_stock
            .entry(item.ingredient_id)
            .or_insert_with(Amount::zero) += item.amount;
    }

    let store_id = metro_reads.tour.store_id;
    let data = TourExportData {
        tour: metro_reads.tour,
        items: metro_reads.shopping_list,
        inventory_stock: Some(inventory_stock),
        show_prices: false,
        metro: Some(MetroExportContext {
            sources: metro_reads.sources,
            article_numbers: metro_reads.article_numbers,
            store_id,
        }),
    };
    let bytes = csv::render_shopping_tour(&data)?;

    Ok(ExportedFile {
        filename: "metro_list.csv".to_string(),
        mime: "text/csv",
        bytes,
    })
}

/// Renders a shopping tour as a plain-text shopping list, subtracting what
/// the event's attached inventories already cover and showing proportional
/// package prices. Requires `EditShoppingTour` permission.
pub async fn render_shopping_tour_txt(
    state: &AppState,
    actor: &AppCtx,
    tour_id: ShoppingTourId,
) -> Result<ExportedFile> {
    use crate::events::shopping::{gather_inventory_items, gather_tour_export};

    let reads = gather_tour_export(state, actor, tour_id).await?;
    let inventory_items = gather_inventory_items(state, &reads.event_inventories).await;

    let mut inventory_stock: HashMap<IngredientId, Amount> = HashMap::new();
    for item in inventory_items {
        *inventory_stock
            .entry(item.ingredient_id)
            .or_insert_with(Amount::zero) += item.amount;
    }

    let data = TourExportData {
        tour: reads.tour,
        items: reads.shopping_list,
        inventory_stock: Some(inventory_stock),
        show_prices: true,
        metro: None,
    };
    let bytes = text::render_shopping_tour(&data);

    Ok(ExportedFile {
        filename: format!("shopping_list_{tour_id}.txt"),
        mime: "text/plain",
        bytes,
    })
}
