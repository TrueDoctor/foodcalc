use typst::foundations::{Dict, IntoValue};
use typst_as_lib::{TypstEngine, TypstTemplateMainFile};
use foodlib_new::ops::export::{EventAllergenInfo, RecipeInfo};
use crate::mapping::{map_event_to_typst, map_recipes_to_typst};

// Load assets once at compile time
static RECIPE_TEMPLATE: &str = include_str!("../templates/recipe.typ");
static DISH_LABEL_TEMPLATE: &str = include_str!("../templates/dish_label.typ");
static EVENT_ALLERGENS_TEMPLATE: &str = include_str!("../templates/event_allergens.typ");
static FONT_R: &[u8] = include_bytes!("../fonts/LinLibertine_R.ttf");
static FONT_RI: &[u8] = include_bytes!("../fonts/LinLibertine_RI.ttf");
static FONT_RB: &[u8] = include_bytes!("../fonts/LinLibertine_RB.ttf");
static FONT_RBI: &[u8] = include_bytes!("../fonts/LinLibertine_RBI.ttf");


#[derive(Debug, Clone, Copy, Default)]
pub enum DishLabelLayout {
    #[default]
    Flat,
    Tent,
}

/// A thread-safe renderer that caches the initialized Typst engines.
pub struct DocumentRenderer {
    // Explicitly define the generic file type the engine expects
    recipe_engine: TypstEngine<TypstTemplateMainFile>,
    dish_label_engine: TypstEngine<TypstTemplateMainFile>,
    event_allergens_engine: TypstEngine<TypstTemplateMainFile>,
}

impl DocumentRenderer {
    /// Initialize this once when your server starts and share it via App State (Arc)
    pub fn new() -> Self {
        let fonts = [FONT_R, FONT_RI, FONT_RB, FONT_RBI];
        Self {
            recipe_engine: TypstEngine::builder()
                .main_file(RECIPE_TEMPLATE)
                .fonts(fonts)
                .build(),
            dish_label_engine: TypstEngine::builder()
                .main_file(DISH_LABEL_TEMPLATE)
                .fonts(fonts)
                .build(),
            event_allergens_engine: TypstEngine::builder()
                .main_file(EVENT_ALLERGENS_TEMPLATE)
                .fonts(fonts)
                .build(),
        }
    }

    /// DRY helper function to handle compilation and PDF conversion
    fn render_pdf(
        &self,
        engine: &TypstEngine<TypstTemplateMainFile>,
        inputs: impl Into<Dict>,
    ) -> eyre::Result<Vec<u8>> {
        let doc = engine
            .compile_with_input(inputs)
            .output
            .map_err(|e| eyre::eyre!("Typst compilation failed: {:?}", e))?;

        let options = Default::default();

        let pdf_bytes = typst_pdf::pdf(&doc, &options).unwrap();
        Ok(pdf_bytes)
    }

    // --- Public Endpoints ---

    pub fn export_recipes(&self, pages: Vec<RecipeInfo>) -> eyre::Result<Vec<u8>> {
        let input_data = map_recipes_to_typst(&pages);
        self.render_pdf(&self.recipe_engine, input_data)
    }

    pub fn export_dish_labels(
        &self,
        overview: EventAllergenInfo,
        layout: DishLabelLayout,
    ) -> eyre::Result<Vec<u8>> {
        let event_overview = map_event_to_typst(&overview);

        let mut export_data = Dict::new();
        export_data.insert("v".into(), event_overview.into_value());
        export_data.insert(
            "layout".into(),
            match layout {
                DishLabelLayout::Flat => "flat".into_value(),
                DishLabelLayout::Tent => "tent".into_value(),
            },
        );

        self.render_pdf(&self.dish_label_engine, export_data)
    }

    pub fn export_event_allergens(&self, overview: EventAllergenInfo) -> eyre::Result<Vec<u8>> {
        let event_overview = map_event_to_typst(&overview);

        let mut export_data = Dict::new();
        export_data.insert("v".into(), event_overview.into_value());
        self.render_pdf(&self.event_allergens_engine, export_data)
    }
}
