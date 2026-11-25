pub mod mapping;
pub mod renderer;

pub use renderer::DocumentRenderer;

use foodlib_new::exports::{
    DishLabelLayout, EventAllergenInfo, EventOverviewInfo, PdfExporter, RecipeInfo,
    TourExportData,
};
use foodlib_new::{Error, Result};

pub struct TypstExporter {
    renderer: DocumentRenderer,
}

impl Default for TypstExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypstExporter {
    pub fn new() -> Self {
        Self {
            renderer: DocumentRenderer::new(),
        }
    }
}

fn map_err(e: eyre::Report) -> Error {
    Error::Misc(e.to_string())
}

impl PdfExporter for TypstExporter {
    fn export_recipes(&self, infos: Vec<RecipeInfo>) -> Result<Vec<u8>> {
        self.renderer.export_recipes(infos).map_err(map_err)
    }

    fn export_event_allergens(&self, info: EventAllergenInfo) -> Result<Vec<u8>> {
        self.renderer.export_event_allergens(info).map_err(map_err)
    }

    fn export_dish_labels(
        &self,
        info: EventAllergenInfo,
        layout: DishLabelLayout,
    ) -> Result<Vec<u8>> {
        let layout = match layout {
            DishLabelLayout::Flat => renderer::DishLabelLayout::Flat,
            DishLabelLayout::Tent => renderer::DishLabelLayout::Tent,
        };
        self.renderer
            .export_dish_labels(info, layout)
            .map_err(map_err)
    }

    fn export_event_overview(&self, _info: EventOverviewInfo) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "Event overview PDF export is not implemented in TypstExporter yet".into(),
        ))
    }

    fn export_shopping_tour(&self, _tour: &TourExportData) -> Result<Vec<u8>> {
        Err(Error::Misc(
            "Shopping tour PDF export is not implemented in TypstExporter yet".into(),
        ))
    }
}