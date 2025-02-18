use axum::{
    extract::{Form, Path, State},
    routing::{delete, put},
};
use bigdecimal::BigDecimal;
use foodlib::{Ingredient, Recipe, RecipeIngredient, RecipeMetaIngredient, RecipeStep, Unit};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgInterval;

use crate::{frontend::MResponse, MyAppState};

pub(crate) fn recipes_edit_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/{recipe_id}", axum::routing::get(recipe_edit_view))
        .route("/add-ingredient/{recipe_id}", put(add_ingredient_form))
        .route("/add-subrecipe/{recipe_id}", put(add_subrecipe_form))
        .route("/add-step/{recipe_id}", put(add_step_form))
        .route("/commit-ingredient", put(handle_ingredient_add))
        .route("/commit-subrecipe", put(handle_subrecipe_add))
        .route("/commit-step", put(handle_step_add))
        .route("/delete-ingredient", delete(handle_ingredient_delete))
        .route(
            "/delete-subrecipe/{recipe_id}/{step_id}",
            delete(handle_subrecipe_delete),
        )
        .route(
            "/delete-step/{recipe_id}/{step_id}",
            delete(handle_step_delete),
        )
        .route("/change-ingredient", put(handle_ingredient_change))
        .route("/change-subrecipe", put(handle_subrecipe_change))
        .route("/change-name", put(handle_name_change))
        .route("/change-step", put(handle_step_change))
        .route("/change-step-order", put(handle_step_order_change))
}

#[derive(Deserialize, Debug)]
pub struct UpdateNameHeader {
    pub recipe_id: i32,
    pub name: String,
    pub comment: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateIngredientHeader {
    pub recipe_id: i32,
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub ingredient_amount: BigDecimal,
    pub ingredient_unit_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct UpdateSubrecipeHeader {
    pub recipe_id: i32,
    pub subrecipe_id: i32,
    pub subrecipe_name: String,
    pub subrecipe_amount: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecipeStepHeader {
    pub step_id: i32,
    pub step_order: f64,
    pub step_name: String,
    pub step_description: String,
    pub fixed_duration_minutes: f64,
    pub duration_per_kg_minutes: f64,
    pub recipe_id: i32,
}

fn pg_interval_from_minutes(minutes: f64) -> PgInterval {
    PgInterval {
        months: 0,
        days: 0,
        microseconds: (60_000_000. * minutes) as i64,
    }
}

pub async fn handle_name_change(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateNameHeader>,
) {
    state
        .db_connection
        .update_recipe(&foodlib::Recipe {
            recipe_id: data.recipe_id,
            name: data.name.clone(),
            comment: Some(data.comment.clone()),
        })
        .await
        .unwrap_or_else(|_| {
            log::warn!("Failed to update recipe {}", data.recipe_id);
            Recipe {
                recipe_id: -1,
                name: String::new(),
                comment: None,
            }
        });
}

pub async fn handle_ingredient_change(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateIngredientHeader>,
) -> Markup {
    let unit = state
        .db_connection
        .get_unit(data.ingredient_unit_id)
        .await
        .unwrap_or_default();
    state
        .db_connection
        .update_recipe_ingredient(
            data.recipe_id,
            data.ingredient_id,
            data.ingredient_amount.clone(),
            unit.unit_id,
        )
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to update ingredient {} in recipe {}",
                data.ingredient_id,
                data.recipe_id
            )
        });
    let recipe_ingredient = RecipeIngredient {
        ingredient: RecipeMetaIngredient::Ingredient(Ingredient {
            ingredient_id: data.ingredient_id,
            name: data.ingredient_name.clone(),
            energy: BigDecimal::from(-1),
            comment: None,
        }),
        amount: data.ingredient_amount,
        unit: state
            .db_connection
            .get_unit(data.ingredient_unit_id)
            .await
            .unwrap_or_default(),
    };
    dbg!(format!("unit: {:?}", recipe_ingredient.unit));
    recipe_ingredient.format_for_ingredient_table(
        state
            .db_connection
            .get_units()
            .await
            .unwrap_or_default()
            .clone(),
        data.recipe_id,
    )
}

pub async fn handle_subrecipe_change(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateSubrecipeHeader>,
) -> Markup {
    state
        .db_connection
        .update_recipe_meta_ingredient(
            data.recipe_id,
            data.subrecipe_id,
            data.subrecipe_amount.clone(),
        )
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to update subrecipe {} in recipe {}",
                data.subrecipe_id,
                data.recipe_id
            )
        });
    let sub_recipe = RecipeIngredient {
        ingredient: RecipeMetaIngredient::MetaRecipe(Recipe {
            recipe_id: data.recipe_id,
            name: data.subrecipe_name,
            comment: None,
        }),
        amount: data.subrecipe_amount,
        unit: state.get_unit(0).await.unwrap_or_default(),
    };
    sub_recipe.format_for_subrecipe_table(data.recipe_id)
}

pub async fn handle_step_change(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateRecipeStepHeader>,
) -> Markup {
    let step = RecipeStep {
        step_id: data.step_id,
        step_order: data.step_order,
        step_name: data.step_name,
        step_description: data.step_description,
        fixed_duration: pg_interval_from_minutes(data.fixed_duration_minutes),
        duration_per_kg: pg_interval_from_minutes(data.duration_per_kg_minutes),
        recipe_id: data.recipe_id,
    };
    state
        .db_connection
        .update_recipe_step(&step)
        .await
        .unwrap_or_default();
    step.format_for_step_table(data.recipe_id)
}

pub async fn handle_step_order_change(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateRecipeStepHeader>,
) -> MResponse {
    let step = RecipeStep {
        step_id: data.step_id,
        step_order: data.step_order,
        step_name: data.step_name,
        step_description: data.step_description,
        fixed_duration: pg_interval_from_minutes(data.fixed_duration_minutes),
        duration_per_kg: pg_interval_from_minutes(data.duration_per_kg_minutes),
        recipe_id: data.recipe_id,
    };
    state
        .db_connection
        .update_recipe_step(&step)
        .await
        .unwrap_or_default();
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn handle_ingredient_delete(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateIngredientHeader>,
) -> MResponse {
    state
        .db_connection
        .delete_recipe_ingredient(data.recipe_id, data.ingredient_id)
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to delete ingredient {} from inventory {}",
                data.ingredient_id,
                data.recipe_id
            )
        });
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn handle_subrecipe_delete(
    State(state): State<MyAppState>,
    Path((recipe_id, subrecipe_id)): Path<(i32, i32)>,
) -> MResponse {
    state
        .db_connection
        .delete_recipe_meta_ingredient(recipe_id, subrecipe_id)
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to delete subrecipe {} from recipe {}",
                subrecipe_id,
                recipe_id
            )
        });
    (recipe_edit_view(State(state), Path(recipe_id))).await
}

pub async fn handle_step_delete(
    State(state): State<MyAppState>,
    Path((recipe_id, step_id)): Path<(i32, i32)>,
) -> MResponse {
    state
        .db_connection
        .delete_step(recipe_id, step_id)
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to delete step {} from recipe {}",
                step_id,
                recipe_id
            )
        });
    (recipe_edit_view(State(state), Path(recipe_id))).await
}

pub async fn handle_ingredient_add(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateIngredientHeader>,
) -> MResponse {
    let ingredient_id = state
        .db_connection
        .get_ingredient_from_string_reference(data.ingredient_name.clone())
        .await
        .unwrap_or(Ingredient {
            ingredient_id: -1,
            name: String::new(),
            energy: BigDecimal::from(-1),
            comment: None,
        })
        .ingredient_id;
    if ingredient_id < 0 {
        add_ingredient_form(State(state), Path(data.recipe_id)).await
    } else {
        state
            .db_connection
            .add_recipe_ingredient(
                data.recipe_id,
                ingredient_id,
                data.ingredient_amount,
                data.ingredient_unit_id,
            )
            .await?;
        recipe_edit_view(State(state), Path(data.recipe_id)).await
    }
}

pub async fn handle_subrecipe_add(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateSubrecipeHeader>,
) -> MResponse {
    let subrecipe_id = state
        .db_connection
        .get_recipe_from_string_reference(data.subrecipe_name.clone())
        .await
        .unwrap_or(Recipe {
            recipe_id: -1,
            name: String::new(),
            comment: None,
        })
        .recipe_id;

    dbg!(format!(
        "requested name {} yielded subrecipe id {}",
        data.subrecipe_name, subrecipe_id
    ));
    if subrecipe_id < 0 {
        add_subrecipe_form(State(state), Path(data.recipe_id)).await
    } else {
        state
            .db_connection
            .add_recipe_meta_ingredient(data.recipe_id, subrecipe_id, data.subrecipe_amount)
            .await?;
        recipe_edit_view(State(state), Path(data.recipe_id)).await
    }
}

pub async fn handle_step_add(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateRecipeStepHeader>,
) -> MResponse {
    let step = RecipeStep {
        step_id: data.step_id,
        step_order: data.step_order,
        step_name: data.step_name,
        step_description: data.step_description,
        fixed_duration: pg_interval_from_minutes(data.fixed_duration_minutes),
        duration_per_kg: pg_interval_from_minutes(data.duration_per_kg_minutes),
        recipe_id: data.recipe_id,
    };
    state.add_recipe_step(&step).await?;
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn add_ingredient_form(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> MResponse {
    let ingredients = state.db_connection.get_ingredients().await?;
    let unit_types = state.get_units().await?;
    Ok(html! {
        form hx-put="recipes/edit/commit-ingredient" hx-swap="outerHTML" hx-target="#contents" {
            datalist id=("ingredient_data_list") {
                @for ingredient in ingredients {
                    option value=(ingredient.name) { }
                }
            }
            div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                h1 { "Add ingredient" }
                input type="hidden" name=("recipe_id") value=(recipe_id);
                input type="hidden" name=("ingredient_id") value=(-1);
                input class="text" type="text" list=("ingredient_data_list") name=("ingredient_name") placeholder="Ingredient" required="required";
                input class="text" type="text" name=("ingredient_amount") placeholder="Amount" value="" required="required";
                select class=("unit fc-select") name="ingredient_unit_id" required="required" {
                    @for unit in unit_types {
                        option value=(unit.unit_id) { (unit.name) }
                    }
                }
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    })
}

pub async fn add_subrecipe_form(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> MResponse {
    let subrecipes = state.get_recipes().await?;
    Ok(html! {
        form hx-put="recipes/edit/commit-subrecipe" hx-swap="outerHTML" hx-target="#contents" {
            datalist id=("subrecipe_data_list") {
                @for subrecipe in subrecipes {
                    option value=(subrecipe.name) { }
                }
            }
            div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                h1 { "Add Subrecipe" }
                input type="hidden" name=("recipe_id") value=(recipe_id);
                input type="hidden" name=("subrecipe_id") value=(-1);
                input class="text" type="text" list=("subrecipe_data_list") name=("subrecipe_name") placeholder="Subrecipe" required="required";
                input class="text" type="text" name=("subrecipe_amount") placeholder="Amount (kg)" value="" required="required";
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    })
}

pub async fn add_step_form(State(_): State<MyAppState>, Path(recipe_id): Path<i32>) -> Markup {
    html! {
        form id="test5" hx-put="recipes/edit/commit-step" hx-swap="outerHTML" hx-target="#contents" {
            div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                h1 { "Add Step" }
                input type="hidden" name=("recipe_id") value=(recipe_id);
                input type="hidden" name=("step_id") value=(-1);
                input class="text" type="text" name=("step_order") placeholder="Order" value="" required="required";
                input class="text" type="text" name=("step_name") placeholder="Name" value="" required="required";
                input class="text" type="text" name=("step_description") placeholder="Description" value="";
                input class="text" type="text" name=("fixed_duration_minutes") placeholder="Fixed Duration in Minutes" value="" required="required";
                input class="text" type="text" name=("duration_per_kg_minutes") placeholder="Duration per kg in Minutes" value="" required="required";
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    }
}

pub async fn recipe_edit_view(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> MResponse {
    let subrecipes = state
        .db_connection
        .get_recipe_meta_ingredients(recipe_id)
        .await?;

    let ingredients = state
        .db_connection
        .get_recipe_ingredients(recipe_id)
        .await?;
    let unit_types = state.get_units().await?;

    let steps = state.db_connection.get_recipe_steps(recipe_id).await?;

    let stats = state
        .new_lib()
        .recipes()
        .get_recipe_stats(recipe_id)
        .await?;

    Ok(html! {
        div id=("contents") class="flex flex-col items-center justify-center mb-16 w-full"{
            div id=("recipe-information") class="w-3/4" {
                form hx-put="recipes/edit/change-name" hx-indicator=".htmx-indicator" hx-swap="none" class="w-full flex flex-col mb-4 pb-4 gap-2" {
                    input type="hidden" name=("recipe_id") value=(recipe_id);
                    input class="text" type="text" name="name" value=(state.get_recipe(recipe_id).await.unwrap_or_default().name) required="required";
                    textarea class="text" name="comment" { (state.get_recipe(recipe_id).await.unwrap_or_default().comment.unwrap_or_default()) }
                    button type="submit" class="btn btn-primary"  { "Change Name and Comment" }}
            }

            div id="styling-bullshit" class="mb-6 mt-6 w-1/4" {
                form hx-put=(format!("recipes/edit/add-subrecipe/{}", recipe_id)) hx-swap="outerHTML" hx-target="#styling-bullshit" class="w-full flex flex-col items-center justify-center pb-4" {
                    input type="hidden" name=("recipe_id") value=(recipe_id);
                    button type="submit" class="btn btn-primary"  { "Add Subrecipe (+)" }
                }
            }

            div class="w-3/4 bg-blue-100 dark:bg-blue-900 p-4 mb-4 rounded-lg" {
                div class="flex justify-between items-center" {
                    h3 class="text-lg font-semibold" { "Recipe Weight Diagnostics" }
                    div class="text-right" {
                        p { "Total Weight: " (format!("{:.3} kg", stats.weight)) }
                        p { "Total Energy: " (format!("{:.2} kJ", stats.energy / 1000.0)) }
                    }
                }
            }


            span class="htmx-indicator" { "Saving\u{a0}…" }

            @if !subrecipes.is_empty() {
                div id=("meta-ingredients") class="w-3/4" {
                    table class="text-inherit table-auto object-center table-fixed" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                        tbody { @for item in subrecipes { (item.format_for_subrecipe_table(recipe_id)) } }
                    }
                }
            }

            form hx-put=(format!("recipes/edit/add-ingredient/{}", recipe_id)) hx-swap="outerHTML" class="w-1/4 mt-6 flex flex-col items-center justify-center" {
                input type="hidden" name=("recipe_id") value=(recipe_id);
                button type="submit" class="btn btn-primary"  { "Add Ingredient (+)" }
            }

            @if !ingredients.is_empty() {
                div id=("ingredients") class="w-3/4" {
                    table class="text-inherit table-auto object-center table-fixed" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                        tbody { @for item in ingredients { (item.format_for_ingredient_table(unit_types.clone(), recipe_id)) } }
                    }
                }
            }

            form hx-put=(format!("recipes/edit/add-step/{}", recipe_id)) hx-swap="outerHTML" class="w-1/4 mt-6 flex flex-col items-center justify-center" {
                input type="hidden" name=("recipe_id") value=(recipe_id);
                button type="submit" class="btn btn-primary"  { "Add Step (+)" }
            }

            @if !steps.is_empty() {
                div id=("steps") class="w-3/4" {
                    table class="text-inherit table-auto object-center table-fixed" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Order" } th { "Name" } th { "Description" } th { "Duration"} th { "Duration per kg" } th { "Delete" } } }
                        tbody { @for item in steps { (item.format_for_step_table(recipe_id)) } }
                    }
                }
            }
            span class="htmx-indicator" { "Saving..." }
        }
    })
}

pub trait IngredientTableFormattable {
    fn format_for_ingredient_table(&self, unit_types: Vec<Unit>, recipe_id: i32) -> Markup;
}

impl IngredientTableFormattable for RecipeIngredient {
    fn format_for_ingredient_table(&self, unit_types: Vec<Unit>, recipe_id: i32) -> Markup {
        let ingredient = match &self.ingredient {
            foodlib::RecipeMetaIngredient::Ingredient(ingredient) => ingredient,
            foodlib::RecipeMetaIngredient::MetaRecipe(_) => {
                panic!("Expected ingredient, got subrecipe")
            }
        };
        let ingredient_id = ingredient.ingredient_id;
        let form_id = format!("ingredient-{}-form", ingredient_id);
        html! {
            tr id=(format!("ingredient-{}", ingredient_id)) style="text-align:center"{ // TODO: Put into form
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                    input class=(form_id) type="hidden" name=("ingredient_id") value=(ingredient_id);
                    input class=(form_id) type="hidden" name=("ingredient_name") value=(self.ingredient.name());
                    div class=(format!("w-full {}",form_id)) name=("ingredient") { (self.ingredient.name()) }
                }
                td { input class=(format!("text {}",form_id)) name="ingredient_amount" value=(self.amount) required="required" hx-put="recipes/edit/change-ingredient" hx-indicator=".htmx-indicator" hx-target=(format!("#ingredient-{}", ingredient_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML"; }
                td { select class=(format!("unit {} fc-select",form_id)) name="ingredient_unit_id" selected=(self.unit.name) hx-target=(format!("#ingredient-{}", ingredient_id)) hx-swap="outerHTML" required="required" hx-put="recipes/edit/change-ingredient" hx-indicator=".htmx-indicator" hx-include=(format!(".{}", form_id)) { @for unit in unit_types {
                    @if unit.unit_id == self.unit.unit_id { option value=(unit.unit_id) selected { (unit.name) } } @else { option value=(unit.unit_id) { (unit.name) } } } } }
                td { button class="btn btn-cancel" hx-target="#contents" hx-delete=("recipes/edit/delete-ingredient") hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}

pub trait SubRecipeTableFormattable {
    fn format_for_subrecipe_table(&self, recipe_id: i32) -> Markup;
}

impl SubRecipeTableFormattable for RecipeIngredient {
    fn format_for_subrecipe_table(&self, recipe_id: i32) -> Markup {
        let subrecipe = match &self.ingredient {
            foodlib::RecipeMetaIngredient::Ingredient(_) => {
                panic!("Expected subrecipe, got ingredient")
            }
            foodlib::RecipeMetaIngredient::MetaRecipe(subrecipe) => subrecipe,
        };
        let subrecipe_id = subrecipe.recipe_id;
        let form_id = format!("subrecipe-{}-form", subrecipe_id);
        html! {
            tr id=(format!("subrecipe-{}", subrecipe_id)) style="text-align:center"{ // TODO: Put into form
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                    input class=(form_id) type="hidden" name=("subrecipe_id") value=(subrecipe_id);
                    input class=(form_id) type="hidden" name=("subrecipe_name") value=(subrecipe.name);
                    input class=(form_id) type="hidden" name=("subrecipe_unit_id") value=(self.unit.unit_id);
                    div class=(format!("w-full {}",form_id)) name=("subrecipe") { a hx-target="#content" hx-get=(format!("/recipes/edit/{}", subrecipe.recipe_id)) { (subrecipe.name) } }
                }
                td { input class=(format!("text {}",form_id)) name="subrecipe_amount" value=(self.amount) required="required" hx-put="recipes/edit/change-subrecipe" hx-target=(format!("#subrecipe-{}", subrecipe_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-indicator=".htmx-indicator" hx-swap="outerHTML"; }
                td { (self.unit.name) }
                td { button class="btn btn-cancel" hx-target="#contents" type="button" hx-delete=(format!("recipes/edit/delete-subrecipe/{}/{}", recipe_id, subrecipe_id)) hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}

pub trait StepTableFormattable {
    fn format_for_step_table(&self, recipe_id: i32) -> Markup;
}

impl StepTableFormattable for RecipeStep {
    fn format_for_step_table(&self, recipe_id: i32) -> Markup {
        let form_id = format!("step-{}-form", self.step_id);
        html! {
            tr id=(format!("step-{}", self.step_id)) style="text-align:center"{
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                    input class=(form_id) type="hidden" name=("step_id") value=(self.step_id);
                    input class=(format!("text {}",form_id)) name="step_order" value=(self.step_order) required="required" hx-put="recipes/edit/change-step-order" hx-target="#contents" hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator";
                }
                td { input class=(format!("text {}",form_id)) name=("step_name") value=(self.step_name) required="required" hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="step_description" value=(self.step_description) hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="fixed_duration_minutes" value=(format!("{}", (self.fixed_duration.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="duration_per_kg_minutes" value=(format!("{}", (self.duration_per_kg.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { button class="btn btn-cancel" hx-target="#contents" type="button" hx-delete=(format!("recipes/edit/delete-step/{}/{}", recipe_id, self.step_id)) hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}
