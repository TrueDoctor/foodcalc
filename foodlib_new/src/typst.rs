use crate::entities::allergen::DietaryFlags;
use crate::entities::property::Property;
use crate::entities::recipe::{RecipeStep, SubRecipe};
use crate::ops::export::{EventAllergenInfo, RecipeInfo, SubrecipeSection};
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


/// Subrecipe ids that should render inline on the previous page instead of
/// forcing their own page. These are the "boiling water" helper recipes whose
/// step timing (calibrated for a 3.5 kW induction stove) still matters but
/// which are too small to warrant a dedicated sheet.
/// NOTE: production recipe ids — Kochendes Wasser (282), Kochendes Salzwasser (408).
const WATER_RECIPE_IDS: [i32; 2] = [282, 408];

async fn format_recipe_info_typst(info: RecipeInfo) -> eyre::Result<String> {
    let mut out = TEMPLATE_FILE.to_string();
    format_recipe_info_body(&mut out, info)?;
    Ok(out)
}

/// Render every recipe sheet for an event into one document, compiled once.
pub async fn export_combined(items: Vec<RecipeInfo>) -> eyre::Result<Vec<u8>> {
    let mut out = TEMPLATE_FILE.to_string();
    for info in items {
        format_recipe_info_body(&mut out, info)?;
    }
    create_pdf(out)
}

/// Append one recipe's typst body (all its subrecipe pages, with water helpers
/// folded inline) onto `text`. Shared by single-recipe and combined exports.
fn format_recipe_info_body(text: &mut String, info: RecipeInfo) -> eyre::Result<()> {
    let prep_for = if info.is_prep {
        Some(info.prep_for.as_slice())
    } else {
        None
    };
    // For each water subrecipe, the extra (non-water) ingredients it carries —
    // e.g. "Kochendes Salzwasser" (408) has salt — expressed as a fraction of
    // that subrecipe's total weight, so the inline note can scale them to the
    // usage-site water amount. Keyed by water recipe id.
    let water_extras = water_extra_ingredients(&info.subrecipes);
    // The prep banner only rides on the first (root) recipe of the sheet.
    let mut first = true;
    for recipe in &info.subrecipes {
        let subrecipe_id = recipe.ingredients.first().map(|sr| sr.subrecipe_id);
        // Water helper recipes are not given their own page; they are rendered
        // inline below each recipe that uses them (see below).
        if subrecipe_id.is_some_and(|id| WATER_RECIPE_IDS.contains(&id)) {
            continue;
        }
        // Subrecipes that are prepped ahead for the event are assumed already
        // made: skip their section entirely (the meta-ingredient still lists
        // them, marked "aus Food Prep").
        if subrecipe_id.is_some_and(|id| info.prepped_subrecipe_ids.contains(&id)) {
            continue;
        }
        // Each page's dietary tag reflects *this* subrecipe's own ingredient tree,
        // so a vegan sub reads vegan even when a non-vegan sibling shares the sheet.
        let allergen_block = format_allergen_block(&recipe.properties, recipe.dietary);
        format_subrecipe(
            text,
            &recipe.ingredients,
            &recipe.steps,
            &info.date,
            &allergen_block,
            &recipe.root_recipe,
            if first { prep_for } else { None },
            // The eingewogen badge only rides on the first (root) recipe sheet.
            first && info.weighed_in,
            &info.prepped_subrecipe_ids,
            &water_extras,
        )?;
        first = false;
    }
    Ok(())
}

/// Non-water extra ingredients per water subrecipe, as fractions of the
/// subrecipe's total weight. For "Kochendes Salzwasser" this yields the salt
/// fraction so the inline note can show salt scaled to the usage amount.
/// Keyed by water recipe id (e.g. 408). Water itself ("Wasser") is excluded.
fn water_extra_ingredients(
    subrecipes: &[SubrecipeSection],
) -> std::collections::HashMap<i32, Vec<(String, f64)>> {
    let mut map = std::collections::HashMap::new();
    for section in subrecipes {
        let rows = &section.ingredients;
        let Some(id) = rows.first().map(|sr| sr.subrecipe_id) else { continue };
        if !WATER_RECIPE_IDS.contains(&id) {
            continue;
        }
        let total: f64 = rows
            .iter()
            .filter(|sr| !sr.is_subrecipe)
            .map(|sr| sr.weight.to_f64().unwrap_or(0.0))
            .sum();
        if total <= 0.0 {
            continue;
        }
        let extras: Vec<(String, f64)> = rows
            .iter()
            .filter(|sr| !sr.is_subrecipe)
            .filter(|sr| !sr.ingredient.eq_ignore_ascii_case("Wasser"))
            .map(|sr| {
                (
                    escape_underscore(&sr.ingredient),
                    sr.weight.to_f64().unwrap_or(0.0) / total,
                )
            })
            .collect();
        if !extras.is_empty() {
            map.insert(id, extras);
        }
    }
    map
}

/// The child recipe id a meta-ingredient row refers to, parsed from the last
/// segment of its `subrecipe_hierarchy` (e.g. "58.282" -> 282).
fn meta_child_id(sr: &SubRecipe) -> Option<i32> {
    sr.subrecipe_hierarchy
        .as_deref()?
        .rsplit('.')
        .next()?
        .parse()
        .ok()
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

#[allow(clippy::too_many_arguments)]
fn format_subrecipe(
    text: &mut String,
    subrecipes: &[SubRecipe],
    steps: &[RecipeStep],
    date: &str,
    allergen_block: &str,
    root_recipe: &str,
    prep_for: Option<&[(String, f64)]>,
    weighed_in: bool,
    prepped_ids: &[i32],
    water_extras: &std::collections::HashMap<i32, Vec<(String, f64)>>,
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
    for ingredient in &meta_ingredients {
        // A meta-ingredient sourced from food prep is flagged so the template can
        // mark it and the cook knows not to make it fresh.
        let from_prep = meta_child_id(ingredient)
            .is_some_and(|id| prepped_ids.contains(&id));
        writeln!(
            text,
            r#"("{}",{},{}),"#,
            escape_underscore(&ingredient.ingredient),
            ingredient.weight,
            from_prep
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
    write!(text, "),{allergen_block}")?;
    if let Some(prep_for) = prep_for {
        write!(text, ", prep_for: (")?;
        for (name, pct) in prep_for {
            write!(
                text,
                r#"("{}",{:.1}),"#,
                escape_typst_string(name),
                pct
            )?;
        }
        write!(text, ")")?;
    }
    if weighed_in {
        write!(text, ", weighed_in: true")?;
    }
    write!(
        text,
        r#", root: "{}""#,
        escape_typst_string(&escape_underscore(root_recipe))
    )?;
    // Boiling-water helpers for each water subrecipe this recipe uses, rendered
    // *inside* #recipe so they stay on the same page (not floated onto the next).
    // The meta-ingredient weight is the scaled water amount for this usage site.
    write!(text, ", water: (")?;
    for meta in &meta_ingredients {
        let Some(water_id) = meta_child_id(meta).filter(|id| WATER_RECIPE_IDS.contains(id))
        else {
            continue;
        };
        let water_kg = meta.weight.to_f64().unwrap_or_default();
        // Extra ingredients (e.g. salt in Salzwasser), scaled to this usage.
        write!(
            text,
            r#"("{}",{},("#,
            escape_typst_string(&escape_underscore(&meta.ingredient)),
            meta.weight
        )?;
        if let Some(extras) = water_extras.get(&water_id) {
            for (name, frac) in extras {
                write!(
                    text,
                    r#"("{}",{:.6}),"#,
                    escape_typst_string(name),
                    frac * water_kg
                )?;
            }
        }
        write!(text, ")),")?;
    }
    write!(text, ")")?;
    writeln!(text, ")")?;
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
