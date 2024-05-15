use std::collections::HashSet;

use crate::{recipes::SubRecipe, FoodBase, RecipeStep};
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::postgres::types::PgInterval;
use typst::{eval::Tracer, foundations::Smart};
use typst_as_library::TypstWrapperWorld;

fn create_pdf(text: String) -> Vec<u8> {
    let world = TypstWrapperWorld::new("./".to_owned(), text);

    let mut tracer = Tracer::default();
    let document = typst::compile(&world, &mut tracer).expect("Error compiling typst");
    let pdf = typst_pdf::pdf(&document, Smart::Auto, None);
    pdf
}

pub(crate) fn export_recipes(subrecipes: &[SubRecipe]) -> Vec<u8> {
    create_pdf("foo".into())
}

/* #recipe("Rezeptname", "22.04.2024 11:00",
  ("ingredients":("Pfeffer": 0.0111111111111, "Kartoffel": 2, "Test": 3, "test": 0.4444, ),"subrecipes":("Naan":1.0,"Dal":2,)), (
    ("title": "First", "desc":lorem(20), "duration": ("fix":10,"var":0)),
    ("title": "Second", "desc":lorem(10), "duration": ("fix":120,"var":20)),
  ))
*/

async fn format_recipe_typst(subrecipes: &[SubRecipe], db: &FoodBase) -> eyre::Result<String> {
    let mut preamble = include_str!("../templates/recipe.typ").to_string();
    let mut keys = subrecipes
        .iter()
        .map(|sr| sr.subrecipe_id)
        .collect::<HashSet<i32>>();

    for subrecipe_id in keys {
        let ingredients: Vec<_> = subrecipes
            .into_iter()
            .filter(|sr| sr.subrecipe_id == subrecipe_id)
            .collect();
        let steps = db.get_recipe_steps(subrecipe_id).await.unwrap_or_default();
        format_subrecipe(&mut preamble, &ingredients, &steps)?;
    }
    Ok(preamble)
}

fn format_subrecipe(
    text: &mut String,
    subrecipes: &[&SubRecipe],
    steps: &[RecipeStep],
) -> eyre::Result<()> {
    let title = escape_underscore(
        &subrecipes
            .first()
            .ok_or(eyre::eyre!("No subrecipe provided"))?
            .subrecipe,
    );
    let ingredients: Vec<_> = subrecipes.iter().filter(|sr| !sr.is_subrecipe).collect();
    let meta_ingredients: Vec<_> = subrecipes.iter().filter(|sr| sr.is_subrecipe).collect();
    let total_weight: BigDecimal = ingredients
        .iter()
        .map(|ingredient| ingredient.weight.clone())
        .sum();

    pub(crate) fn escape_underscore(s: &str) -> String {
        s.replace('_', " ")
    }
    use std::fmt::Write;
    writeln!(text, "#recipe({title},\"22.04.2024 11:00\",")?;
    writeln!(text, "(\"subrecipes\": ")?;
    for ingredient in meta_ingredients {
        writeln!(
            text,
            r#""{}":"{}","#,
            escape_underscore(&ingredient.ingredient),
            ingredient.weight
        )?;
    }
    writeln!(text, "), (\"ingredients\": ")?;
    for ingredient in ingredients {
        writeln!(
            text,
            r#""{}":"{}","#,
            escape_underscore(&ingredient.ingredient),
            ingredient.weight
        )?;
    }
    writeln!(text, ")),(")?;
    for step in steps {
        pub(crate) fn to_minutes(duration: &PgInterval) -> f64 {
            duration.microseconds as f64 / 1_000_000. / 60.
        }
        writeln!(
            text,
            r#"("title": "{}", "desc":{}, "duration": ("fix":{:.3},"var":{:.3})),"#,
            step.step_name,
            step.step_description,
            to_minutes(&step.fixed_duration),
            to_minutes(&step.duration_per_kg) * total_weight.to_f64().unwrap_or_default()
        )?;
    }
    writeln!(text, "))")?;
    Ok(())
}
