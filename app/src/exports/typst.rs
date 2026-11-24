use foodlib_new::ops::export::{EventAllergenInfo, RecipeInfo};
use typst_renderer::renderer::{DishLabelLayout as PdfLayout, DocumentRenderer};

use super::{DishLabelLayout, PdfExporter};
use crate::{Error, Result};

pub struct TypstExporter {
    inner: DocumentRenderer,
}

impl TypstExporter {
    pub fn new() -> Self {
        Self {
            inner: DocumentRenderer::new(),
        }
    }
}

impl PdfExporter for TypstExporter {
    fn export_recipes(&self, infos: Vec<RecipeInfo>) -> Result<Vec<u8>> {
        self.inner
            .export_recipes(infos)
            .map_err(|e| Error::Misc(e.to_string()))
    }

    fn export_event_allergens(&self, info: EventAllergenInfo) -> Result<Vec<u8>> {
        self.inner
            .export_event_allergens(info)
            .map_err(|e| Error::Misc(e.to_string()))
    }

    fn export_dish_labels(
        &self,
        info: EventAllergenInfo,
        layout: DishLabelLayout,
    ) -> Result<Vec<u8>> {
        let layout = match layout {
            DishLabelLayout::Flat => PdfLayout::Flat,
            DishLabelLayout::Tent => PdfLayout::Tent,
        };
        self.inner
            .export_dish_labels(info, layout)
            .map_err(|e| Error::Misc(e.to_string()))
    }
}