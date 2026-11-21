//! Shared document rendering engines (Typst PDF singleton, Plain Text, Metro CSV).

use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use crate::app::{Error, MyAppState, Result};
use foodlib_new::entities::event::{ShoppingListItemWithUsage, ShoppingTour};
use foodlib_new::entities::types::ids::IngredientId;
use foodlib_new::ops::export::{EventAllergenInfo, EventOverviewInfo, RecipeInfo};
use foodlib_new::types::amount::Amount;

pub mod csv;
pub mod text;
#[cfg(feature = "typst")]
mod typst;
#[cfg(not(feature = "typst"))]
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
    Pdf,
    #[serde(alias = "text")]
    Txt,
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
    pub sources: Vec<foodlib_new::entities::ingredient::IngredientSource>,
    pub article_numbers: HashMap<IngredientId, String>,
}

/// Polymorphic PDF rendering backend stored once on [`MyAppState`].
pub trait PdfExporter: Send + Sync {
    fn export_recipes(&self, _infos: Vec<RecipeInfo>) -> Result<Vec<u8>> {
        Err(Error::Misc("PDF export for recipes is not implemented".into()))
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
    #[cfg(feature = "typst")]
    {
        Arc::new(typst::TypstExporter::new())
    }
    #[cfg(not(feature = "typst"))]
    {
        Arc::new(no_renderer::NoRenderer)
    }
}

/// Runs a Typst PDF render job on Tokio's blocking pool using the singleton engine.
pub async fn render_pdf<F>(state: &MyAppState, filename: String, job: F) -> Result<ExportedFile>
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