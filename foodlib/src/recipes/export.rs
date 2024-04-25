use super::RecipeStep;

use bigdecimal::ToPrimitive;
use sqlx::postgres::types::PgInterval;

use sqlx;

use super::SubRecipe;

use sqlx::types::BigDecimal;

use crate::FoodBase;

#[cfg(not(feature = "tectonic"))]
pub async fn compile_pdf(_: String) -> Result<Vec<u8>, eyre::ErrReport> {
    Err(eyre::eyre!("tectonic feature not enabled"))
}

#[cfg(feature = "tectonic")]
pub async fn compile_pdf(text: String) -> Result<Vec<u8>, eyre::ErrReport> {
    use std::path::Path;
    use tectonic::driver::ProcessingSessionBuilder;
    use tectonic::status;
    use tokio::task::spawn_blocking;
    let mut status = status::NoopStatusBackend::default();
    let mut files = {
        spawn_blocking(move || {
            let auto_create_config_file = false;
            let config = tectonic::config::PersistentConfig::open(auto_create_config_file).unwrap();

            let only_cached = false;
            let bundle = config.default_bundle(only_cached, &mut status).unwrap();

            let format_cache_path = config.format_cache_path().unwrap();

            let mut sb = ProcessingSessionBuilder::default();
            sb.filesystem_root(Path::new("../recipes"))
                .primary_input_buffer(text.as_bytes())
                .tex_input_name("texput.tex")
                .format_name("latex")
                .keep_logs(false)
                .keep_intermediates(false)
                .format_cache_path(format_cache_path)
                .bundle(bundle)
                .do_not_write_output_files()
                .print_stdout(false);
            let mut sess = sb
                .create(&mut status)
                .expect("failed to initialize the LaTeX processing session");
            if let Err(e) = sess.run(&mut status) {
                log::error!("failed to run the LaTeX processing session: {}", e);
            }
            sess.into_file_data()
        })
        .await?
    };
    let Some(pdf) = files.remove("texput.pdf") else {
        return Err(eyre::eyre!(
            "LaTeX didn't report failure, but no PDF was created (??)"
        ));
    };
    let pdf_data = pdf.data;
    Ok(pdf_data)
}

impl FoodBase {
    pub async fn fetch_subrecipes(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> eyre::Result<Vec<SubRecipe>> {
        let subrecipes = sqlx::query_as!(
            SubRecipe,
            r#"
                SELECT
                    recipe as "recipe!",
                    ingredient as "ingredient!",
                    round(weight * $2, 10)  as "weight!",
                    subrecipe as "subrecipe!",
                    is_subrecipe as "is_subrecipe!",
                    subrecipe_id as "subrecipe_id!"
                FROM subrecipes
                WHERE recipe_id = $1
                ORDER BY recipe, subrecipe_id, ingredient

            "#,
            recipe_id,
            weight,
        )
        .fetch_all(&*self.pg_pool)
        .await?;
        Ok(subrecipes)
    }

    pub async fn fetch_subrecipes_from_user_input(
        &self,
        recipe_id: i32,
        people: f64,
        calories: u32,
    ) -> eyre::Result<Vec<SubRecipe>> {
        let total_calories = BigDecimal::from((calories as f64 * people) as u64);
        let weight = self
            .calc_energy_to_weight(recipe_id, total_calories)
            .await
            .unwrap_or_default();
        self.fetch_subrecipes(recipe_id, weight).await
    }

    //pub async fn fetch_subrecipes_from_meal(&self, meal_id: i32) -> eyre::Result<()> {
    //    let meal = self.get_meal
    //    let weight = meal.weight;
    //    let recipe_id = meal.recipe_id;
    //    self.fetch_subrecipes(recipe_id, weight).await
    //}

    pub async fn calc_energy_to_weight(
        &self,
        recipe_id: i32,
        energy: BigDecimal,
    ) -> eyre::Result<BigDecimal> {
        let recipe_stats = sqlx::query!(
            r#"
                SELECT
                    weight, energy
                    FROM recipe_stats
                WHERE recipe_id = $1
            "#,
            recipe_id,
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        let recipe_weight = recipe_stats.weight.unwrap();
        let recipe_energy = recipe_stats.energy.unwrap();
        Ok(recipe_weight / recipe_energy * energy)
    }

    pub async fn format_subrecipes_markdown(&self, subrecipes: Vec<SubRecipe>) -> String {
        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();
        keys.dedup();

        let mut subrecipe_markdown = Vec::new();
        for subrecipe_id in keys {
            let mut text = String::new();
            let ingredients: Vec<_> = subrecipes
                .iter()
                .filter(|sr| sr.subrecipe_id == subrecipe_id)
                .collect();
            let steps = self
                .get_recipe_steps(subrecipe_id)
                .await
                .unwrap_or_default();
            let title = ingredients
                .first()
                .ok_or(eyre::eyre!("No subrecipe provided"))
                .unwrap()
                .subrecipe
                .clone();
            text.push_str(&format!("# {}\n", title));
            let weight: BigDecimal = ingredients
                .iter()
                .map(|ingredient| ingredient.weight.clone())
                .sum();

            for ingredient in ingredients {
                text.push_str(&format!(
                    "* {:.3}kg {}\n",
                    ingredient.weight, ingredient.ingredient
                ));
            }

            if !steps.is_empty() {
                for (i, step) in steps.into_iter().enumerate() {
                    pub(crate) fn to_minutes(duration: PgInterval) -> f64 {
                        duration.microseconds as f64 / 1_000_000. / 60.
                    }
                    let fixed_duration = to_minutes(step.fixed_duration);
                    let duration_per_kg = to_minutes(step.duration_per_kg);
                    let scaled_duration = duration_per_kg * weight.to_f64().unwrap_or_default();
                    let duration = fixed_duration + scaled_duration;

                    text.push_str(&format!(
                        "## {}. {} ({:.3} + {:.3} = {:.3} min)\n{}\n",
                        i + 1,
                        step.step_name,
                        fixed_duration,
                        scaled_duration,
                        duration,
                        step.step_description
                    ));
                }
            }
            subrecipe_markdown.push(text);
        }
        subrecipe_markdown.join("\n")
    }

    pub async fn generate_recipes_typst(&self, subrecipes: &[SubRecipe]) -> Vec<u8> {
        crate::typst::export_recipes(subrecipes)
    }

    pub async fn format_subrecipes_latex(&self, subrecipes: Vec<SubRecipe>) -> String {
        let mut keys = subrecipes
            .iter()
            .map(|sr| sr.subrecipe_id)
            .collect::<Vec<i32>>();
        keys.dedup();

        let mut text = r"
            \documentclass[11pt,a4paper]{article}


            \usepackage[ngerman]{babel}
            \usepackage{ifxetex}

            \ifxetex
              \usepackage{fontspec}
            \else
              \usepackage[T1]{fontenc}
              \usepackage[utf8]{inputenc}
              \usepackage{lmodern}
            \fi

            \usepackage{gensymb}

            \usepackage{recipe}

            \begin{document}
            "
        .to_owned();

        for subrecipe_id in keys {
            let ingredients: Vec<_> = subrecipes
                .iter()
                .filter(|sr| sr.subrecipe_id == subrecipe_id)
                .collect();
            let steps = self
                .get_recipe_steps(subrecipe_id)
                .await
                .unwrap_or_default();
            self.format_subrecipe(&mut text, ingredients, steps)
                .unwrap_or_else(|e| log::error!("{e}"));
        }
        text.push_str("\\end{document}");
        text
    }

    pub async fn format_recipe_latex_from_user_input(
        &self,
        recipe_id: i32,
        people: f64,
        energy: u32,
    ) -> eyre::Result<String> {
        let subrecipes = self
            .fetch_subrecipes_from_user_input(recipe_id, people, energy)
            .await?;
        Ok(self.format_subrecipes_latex(subrecipes).await)
    }

    // TODO Should probabyl use fetch_subrecipes and format_subrecipes_latex
    pub async fn save_recipe_export(
        &self,
        recipe_id: i32,
        weight: BigDecimal,
    ) -> Result<(), eyre::Error> {
        use std::io::Write;
        let text = self
            .format_subrecipes_latex(self.fetch_subrecipes(recipe_id, weight).await?)
            .await;

        let title = self
            .get_recipe_from_string_reference(recipe_id.to_string())
            .await
            .unwrap()
            .name;

        #[cfg(feature = "tectonic")]
        {
            let pdf_data = compile_pdf(text).await?;
            println!("Output PDF size is {} bytes", pdf_data.len());

            let create_result = std::fs::create_dir("recipes/out");
            if let Err(e) = create_result {
                if e.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(eyre::eyre!("failed to create output directory: {}", e));
                }
            }
            let mut file = std::fs::File::create(format!("recipes/out/{}.pdf", title))?;
            file.write_all(&pdf_data)?;
        }
        #[cfg(not(feature = "tectonic"))]
        {
            let mut file = std::fs::File::create(format!(
                "recipes/{}.tex",
                title.replace(' ', "_").to_lowercase()
            ))
            .unwrap();
            file.write_all(text.as_bytes()).unwrap();
        }
        Ok(())
    }

    pub fn format_subrecipe(
        &self,
        text: &mut String,
        subrecipes: Vec<&SubRecipe>,
        steps: Vec<RecipeStep>,
    ) -> Result<(), eyre::Error> {
        let title = escape_underscore(
            &subrecipes
                .first()
                .ok_or(eyre::eyre!("No subrecipe provided"))?
                .subrecipe,
        );
        let ingredients: Vec<_> = subrecipes.iter().filter(|sr| !sr.is_subrecipe).collect();
        let meta_ingredients: Vec<_> = subrecipes.iter().filter(|sr| sr.is_subrecipe).collect();
        let weight: BigDecimal = ingredients
            .iter()
            .map(|ingredient| ingredient.weight.clone())
            .sum();

        pub(crate) fn escape_underscore(s: &str) -> String {
            s.replace('_', " ")
        }
        use std::fmt::Write;
        writeln!(text, "\\addrecipe{{{title}}}")?;
        for ingredient in meta_ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight.round(3)
            )?;
        }
        for ingredient in ingredients {
            writeln!(
                text,
                "\\addingredient{{{}}}{{{}}}{{{}kg}}",
                title,
                escape_underscore(&ingredient.ingredient),
                ingredient.weight.round(3)
            )?;
        }
        for step in steps {
            pub(crate) fn to_minutes(duration: PgInterval) -> f64 {
                duration.microseconds as f64 / 1_000_000. / 60.
            }
            let duration = to_minutes(step.duration_per_kg) * weight.to_f64().unwrap_or_default()
                + to_minutes(step.fixed_duration);
            writeln!(
                text,
                "\\addstep{{{}}}{{{}}}{{{}}}{{{:.3} min}}",
                title, step.step_name, step.step_description, duration
            )?;
        }
        writeln!(text, "\\printrecipe{{{title}}}")?;
        Ok(())
    }
}
