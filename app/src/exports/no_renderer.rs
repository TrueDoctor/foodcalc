use foodlib_new::ops::export::{EventAllergenInfo, EventOverviewInfo, RecipeInfo};

use super::{DishLabelLayout, PdfExporter, TourExportData};
use crate::{Error, Result};

pub struct NoRenderer;

impl NoRenderer {
    fn unavailable() -> Error {
        Error::Misc("Server compiled without typst support".into())
    }
}

impl PdfExporter for NoRenderer {
    fn export_recipes(&self, _: Vec<RecipeInfo>) -> Result<Vec<u8>> {
        Err(Self::unavailable())
    }

    fn export_event_allergens(&self, _: EventAllergenInfo) -> Result<Vec<u8>> {
        Err(Self::unavailable())
    }

    fn export_dish_labels(&self, _: EventAllergenInfo, _: DishLabelLayout) -> Result<Vec<u8>> {
        Err(Self::unavailable())
    }

    fn export_event_overview(&self, _: EventOverviewInfo) -> Result<Vec<u8>> {
        Err(Self::unavailable())
    }

    fn export_shopping_tour(&self, _: &TourExportData) -> Result<Vec<u8>> {
        Err(Self::unavailable())
    }
}