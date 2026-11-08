mod typst_event_info;
mod typst_recipe;

use crate::ops::export::{EventAllergenInfo, RecipeInfo};
use crate::typst::typst_event_info::{DishLabelLayoutInput, TypstDishLabel};
use crate::typst::typst_recipe::{RecipePagesInput, TypstRecipePage};
use typst_as_lib::TypstEngine;

static RECIPE_TEMPLATE: &str = include_str!("../templates/recipe.typ");
static DISH_LABEL_TEMPLATE: &str = include_str!("../templates/dish_label.typ");
static EVENT_ALLERGENS_TEMPLATE: &str = include_str!("../templates/event_allergens.typ");
static FONT_R: &[u8] = include_bytes!("../fonts/LinLibertine_R.ttf");
static FONT_RI: &[u8] = include_bytes!("../fonts/LinLibertine_RI.ttf");
static FONT_RB: &[u8] = include_bytes!("../fonts/LinLibertine_RB.ttf");
static FONT_RBI: &[u8] = include_bytes!("../fonts/LinLibertine_RBI.ttf");

#[derive(Debug, Clone, Copy, Default)]
pub enum DishLabelLayout {
    /// 6-up grid of small labels on A4.
    #[default]
    Flat,
    /// One label per A5-landscape page, top and bottom halves mirrored so when folded
    /// along the centerline both sides display correctly (tent card).
    Tent,
}

pub async fn export_recipes(info: RecipeInfo) -> eyre::Result<Vec<u8>> {
    let recipe_template = TypstEngine::builder()
        .main_file(RECIPE_TEMPLATE)
        .fonts([FONT_R, FONT_RI, FONT_RB, FONT_RBI])
        .build();

    // Map the database model to the Typst view models
    let pages = TypstRecipePage::from_recipe_info(&info);
    let input_data = RecipePagesInput { v: pages };

    let doc = recipe_template
        .compile_with_input(input_data)
        .output
        .expect("typst::compile() returned an error!");

    Ok(typst_pdf::pdf(&doc, &Default::default()).expect("Could not generate pdf."))
}

/// Render every recipe sheet for an event into one document, compiled once.
pub async fn export_combined(items: Vec<RecipeInfo>) -> eyre::Result<Vec<u8>> {
    let recipe_template = TypstEngine::builder()
        .main_file(RECIPE_TEMPLATE)
        .fonts([FONT_R, FONT_RI, FONT_RB, FONT_RBI])
        .build();

    // Map the database entities to the Typst view models
    let pages: Vec<TypstRecipePage> = items
        .iter()
        .flat_map(TypstRecipePage::from_recipe_info)
        .collect();

    // Wrap in the top-level struct to create the `inputs.v` dictionary
    let input_data = RecipePagesInput { v: pages };

    let doc = recipe_template
        .compile_with_input(input_data)
        .output
        .expect("typst::compile() returned an error!");

    Ok(typst_pdf::pdf(&doc, &Default::default()).expect("Could not generate pdf."))
}

pub async fn export_dish_labels(
    info: EventAllergenInfo,
    layout: DishLabelLayout,
) -> eyre::Result<Vec<u8>> {
    let dish_label_template = TypstEngine::builder()
        .main_file(DISH_LABEL_TEMPLATE)
        .fonts([FONT_R, FONT_RI, FONT_RB, FONT_RBI])
        .build();

    // Map the complex data
    let overview = TypstDishLabel::from_event_info(&info);

    let input_data = DishLabelLayoutInput {
        v: vec![overview],
        layout: match layout {
            DishLabelLayout::Flat => "flat".into(),
            DishLabelLayout::Tent => "tent".into(),
        },
    };

    let doc = dish_label_template
        .compile_with_input(input_data) // Passed directly as Into<Dict>
        .output
        .expect("typst::compile() returned an error!");

    Ok(typst_pdf::pdf(&doc, &Default::default()).expect("Could not generate pdf."))
}

pub async fn export_event_allergens(info: EventAllergenInfo) -> eyre::Result<Vec<u8>> {
    let event_allergens_template = TypstEngine::builder()
        .main_file(EVENT_ALLERGENS_TEMPLATE)
        .fonts([FONT_R, FONT_RI, FONT_RB, FONT_RBI])
        .build();

    let dish_label = TypstDishLabel::from_event_info(&info);
    let input_data = DishLabelLayoutInput {
        v: vec![dish_label],
        layout: "flat".into(),
    };

    let doc = event_allergens_template
        .compile_with_input(input_data)
        .output
        .expect("typst::compile() returned an error!");

    Ok(typst_pdf::pdf(&doc, &Default::default()).expect("Could not generate pdf."))
}
