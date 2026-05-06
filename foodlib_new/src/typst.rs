use crate::entities::recipe::{RecipeStep, SubRecipe};
use crate::ops::export::RecipeInfo;
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::postgres::types::PgInterval;
use typst::foundations::Bytes;
use typst::text::Font;
use typst_as_lib::TypstTemplate;

static TEMPLATE_FILE: &str = include_str!("../templates/recipe.typ");
static FONT: &[u8] = include_bytes!("../fonts/LinLibertine_R.ttf");

fn create_pdf(text: String) -> eyre::Result<Vec<u8>> {
    let font = Font::new(Bytes::from(FONT), 0)
        .ok_or_else(|| eyre::eyre!("Could not parse font"))?;
    let template = TypstTemplate::new(vec![font], text);
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

async fn format_recipe_info_typst(info: RecipeInfo) -> eyre::Result<String> {
    let mut preamble = TEMPLATE_FILE.to_string();
    for recipe in info.subrecipes {
        format_subrecipe(&mut preamble, &recipe.0, &recipe.1, &info.date)?;
    }
    Ok(preamble)
}

fn format_subrecipe(
    text: &mut String,
    subrecipes: &[SubRecipe],
    steps: &[RecipeStep],
    date: &str,
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
    writeln!(text, "))")?;
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
