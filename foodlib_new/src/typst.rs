use crate::entities::allergen::DietaryFlags;
use crate::entities::property::Property;
use crate::entities::recipe::{RecipeStep, SubRecipe};
use crate::ops::export::{EventAllergenInfo, RecipeInfo};
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::postgres::types::PgInterval;
use typst::foundations::Bytes;
use typst::text::Font;
use typst_as_lib::TypstTemplate;

static TEMPLATE_FILE: &str = include_str!("../templates/recipe.typ");
static DISH_LABEL_TEMPLATE: &str = include_str!("../templates/dish_label.typ");
static EVENT_ALLERGENS_TEMPLATE: &str = include_str!("../templates/event_allergens.typ");
static FONT_R: &[u8] = include_bytes!("../fonts/LinLibertine_R.ttf");
static FONT_RI: &[u8] = include_bytes!("../fonts/LinLibertine_RI.ttf");
static FONT_RB: &[u8] = include_bytes!("../fonts/LinLibertine_RB.ttf");
static FONT_RBI: &[u8] = include_bytes!("../fonts/LinLibertine_RBI.ttf");

fn create_pdf(text: String) -> eyre::Result<Vec<u8>> {
    let fonts: Vec<Font> = [FONT_R, FONT_RI, FONT_RB, FONT_RBI]
        .iter()
        .filter_map(|bytes| Font::new(Bytes::from(*bytes), 0))
        .collect();
    if fonts.is_empty() {
        return Err(eyre::eyre!("Could not parse any font"));
    }
    let template = TypstTemplate::new(fonts, text);
    let doc = template
        .compile()
        .output
        .map_err(|e| eyre::eyre!("typst compile error: {e:?}"))?;
    let options = Default::default();
    let pdf = typst_pdf::pdf(&doc, &options)
        .map_err(|e| eyre::eyre!("Could not generate pdf: {e:?}"))?;
    Ok(pdf)
}

pub async fn export_recipes(info: RecipeInfo) -> eyre::Result<Vec<u8>> {
    let typst = format_recipe_info_typst(info).await?;
    create_pdf(typst)
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DishLabelLayout {
    /// 6-up grid of small labels on A4.
    #[default]
    Flat,
    /// One label per A5-landscape page, top and bottom halves mirrored so when folded
    /// along the centerline both sides display correctly (tent card).
    Tent,
}

pub async fn export_dish_labels(
    info: EventAllergenInfo,
    layout: DishLabelLayout,
) -> eyre::Result<Vec<u8>> {
    let typst = format_dish_labels_typst(&info, layout);
    create_pdf(typst)
}

pub async fn export_event_allergens(info: EventAllergenInfo) -> eyre::Result<Vec<u8>> {
    let typst = format_event_allergens_typst(&info);
    create_pdf(typst)
}

fn format_dish_labels_typst(info: &EventAllergenInfo, layout: DishLabelLayout) -> String {
    use std::fmt::Write;
    let mut out = DISH_LABEL_TEMPLATE.to_string();
    let layout_str = match layout {
        DishLabelLayout::Flat => "flat",
        DishLabelLayout::Tent => "tent",
    };
    writeln!(
        out,
        "#dish_labels(\"{}\",\"{}\",(",
        escape_typst_string(&info.event_name),
        layout_str
    )
    .unwrap();
    for meal in &info.meals {
        let (contains, may) = split_properties(&meal.properties);
        writeln!(
            out,
            "(name: \"{name}\", place: \"{place}\", servings: {servings}, \
             time: \"{time}\", contains: ({contains}), may_contain: ({may}), \
             dietary: (vegan: {v}, vegetarian: {vt}, lactose_free: {lf}, gluten_free: {gf})),",
            name = escape_typst_string(&meal.recipe_name),
            place = escape_typst_string(&meal.place),
            servings = meal.servings,
            time = format_time(meal.start_time),
            contains = format_array(&contains),
            may = format_array(&may),
            v = meal.dietary.vegan,
            vt = meal.dietary.vegetarian,
            lf = meal.dietary.lactose_free,
            gf = meal.dietary.gluten_free,
        )
        .unwrap();
    }
    writeln!(out, "))").unwrap();
    out
}

fn format_event_allergens_typst(info: &EventAllergenInfo) -> String {
    use std::fmt::Write;
    let mut out = EVENT_ALLERGENS_TEMPLATE.to_string();
    writeln!(
        out,
        "#event_allergen_overview(\"{}\",(",
        escape_typst_string(&info.event_name)
    )
    .unwrap();
    for meal in &info.meals {
        let (contains, may) = split_properties(&meal.properties);
        writeln!(
            out,
            "(name: \"{name}\", place: \"{place}\", servings: {servings}, \
             time: \"{time}\", contains: ({contains}), may_contain: ({may}), \
             dietary: (vegan: {v}, vegetarian: {vt}, lactose_free: {lf}, gluten_free: {gf})),",
            name = escape_typst_string(&meal.recipe_name),
            place = escape_typst_string(&meal.place),
            servings = meal.servings,
            time = format_time(meal.start_time),
            contains = format_array(&contains),
            may = format_array(&may),
            v = meal.dietary.vegan,
            vt = meal.dietary.vegetarian,
            lf = meal.dietary.lactose_free,
            gf = meal.dietary.gluten_free,
        )
        .unwrap();
    }
    writeln!(out, "))").unwrap();
    out
}

fn split_properties(properties: &[Property]) -> (Vec<String>, Vec<String>) {
    let mut contains: Vec<String> = Vec::new();
    let mut may: Vec<String> = Vec::new();
    for p in properties {
        let lower = p.name.to_lowercase();
        if let Some(rest) = lower.strip_prefix("~spuren-") {
            may.push(rest.to_string());
        } else {
            contains.push(lower);
        }
    }
    contains.sort();
    contains.dedup();
    may.sort();
    may.dedup();
    contains = crate::ops::allergens::compact_allergen_set(&contains);
    may = crate::ops::allergens::compact_allergen_set(&may);
    // Escape only after collapsing — the canonical names don't contain quotes,
    // but be safe in case future canonicals do.
    let contains = contains.into_iter().map(|s| escape_typst_string(&s)).collect();
    let may = may.into_iter().map(|s| escape_typst_string(&s)).collect();
    (contains, may)
}

fn format_array(items: &[String]) -> String {
    // Single-element arrays in Typst require a trailing comma (e.g. `("milk",)`),
    // but `()` is the empty array — adding a trailing comma there yields `(,)`,
    // which is a parse error. Emit the trailing comma only when there's content.
    if items.is_empty() {
        return String::new();
    }
    items
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>()
        .join(",")
        + ","
}

fn format_time(t: time::OffsetDateTime) -> String {
    use time::format_description;
    let f = format_description::parse("[day].[month]. [hour]:[minute]").unwrap();
    t.format(&f).unwrap_or_default()
}


async fn format_recipe_info_typst(info: RecipeInfo) -> eyre::Result<String> {
    let mut preamble = TEMPLATE_FILE.to_string();
    let allergen_block = format_allergen_block(&info.properties, info.dietary);
    for recipe in info.subrecipes {
        format_subrecipe(
            &mut preamble,
            &recipe.0,
            &recipe.1,
            &info.date,
            &allergen_block,
        )?;
    }
    Ok(preamble)
}

/// Produce a typst literal for the dictionary the template's `#recipe` function expects
/// under the `allergens` key: contains array, may_contain array, dietary record.
fn format_allergen_block(properties: &[Property], dietary: DietaryFlags) -> String {
    let (contains, may_contain) = split_properties(properties);
    format!(
        "(contains: ({c}), may_contain: ({m}), \
         dietary: (vegan: {v}, vegetarian: {vt}, lactose_free: {lf}, gluten_free: {gf}))",
        c = format_array(&contains),
        m = format_array(&may_contain),
        v = dietary.vegan,
        vt = dietary.vegetarian,
        lf = dietary.lactose_free,
        gf = dietary.gluten_free,
    )
}

fn format_subrecipe(
    text: &mut String,
    subrecipes: &[SubRecipe],
    steps: &[RecipeStep],
    date: &str,
    allergen_block: &str,
) -> eyre::Result<()> {
    let title = escape_underscore(
        &subrecipes
            .first()
            .ok_or_else(|| eyre::eyre!("No subrecipe provided"))?
            .subrecipe,
    );
    let ingredients: Vec<_> = subrecipes.iter().filter(|sr| !sr.is_subrecipe).collect();
    let meta_ingredients: Vec<_> = subrecipes.iter().filter(|sr| sr.is_subrecipe).collect();
    let total_weight: BigDecimal = ingredients
        .iter()
        .map(|ingredient| ingredient.weight.clone())
        .sum();

    use std::fmt::Write;
    writeln!(text, "#recipe(\"{title}\",\"{date}\",")?;
    writeln!(text, "(\"subrecipes\":( ")?;
    for ingredient in meta_ingredients {
        writeln!(
            text,
            r#"("{}",{}),"#,
            escape_underscore(&ingredient.ingredient),
            ingredient.weight
        )?;
    }
    writeln!(text, "), \"ingredients\": (")?;
    for ingredient in ingredients {
        writeln!(
            text,
            r#"("{}",{}),"#,
            escape_underscore(&ingredient.ingredient),
            ingredient.weight
        )?;
    }
    writeln!(text, ")),(")?;
    for step in steps {
        writeln!(
            text,
            r#"("title": "{}", "desc":"{}", "duration": ("fix":{:.3},"var":{:.3})),"#,
            escape_typst_string(&step.name),
            escape_typst_string(&step.description),
            to_minutes(&step.fixed_duration),
            to_minutes(&step.duration_per_kg) * total_weight.to_f64().unwrap_or_default()
        )?;
    }
    writeln!(text, "),{allergen_block})")?;
    Ok(())
}

fn escape_underscore(s: &str) -> String {
    s.replace('_', " ")
}

fn escape_typst_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn to_minutes(duration: &PgInterval) -> f64 {
    duration.microseconds as f64 / 1_000_000. / 60.
}
