use axum::extract::{Form, Path, State};
use bigdecimal::BigDecimal;
use foodlib::{Ingredient, Recipe, RecipeIngredient, RecipeMetaIngredient, RecipeStep, Unit};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgInterval;

use crate::MyAppState;

pub(crate) fn recipes_edit_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:recipe_id", axum::routing::get(recipe_edit_view))
        //.route("/add-step", axum::routing::put(add_step_form))
        //.route("/edit-step", axum::routing::put(handle_edit_step))
        //.route("/delete-step", axum::routing::delete(handle_step_delete))
        .route(
            "/add-ingredient/:recipe_id",
            axum::routing::put(add_ingredient_form),
        )
        .route(
            "/add-subrecipe/:recipe_id",
            axum::routing::put(add_subrecipe_form),
        )
        .route("/add-step/:recipe_id", axum::routing::put(add_step_form))
        .route(
            "/commit-ingredient",
            axum::routing::put(handle_ingredient_add),
        )
        .route(
            "/commit-subrecipe",
            axum::routing::put(handle_subrecipe_add),
        )
        .route("/commit-step", axum::routing::put(handle_step_add))
        .route(
            "/delete-ingredient",
            axum::routing::delete(handle_ingredient_delete),
        )
        .route(
            "/delete-subrecipe",
            axum::routing::delete(handle_subrecipe_delete),
        )
        .route("/delete-step", axum::routing::delete(handle_step_delete))
        .route(
            "/change-ingredient",
            axum::routing::put(handle_ingredient_change),
        )
        .route(
            "/change-subrecipe",
            axum::routing::put(handle_subrecipe_change),
        )
        .route("/change-step", axum::routing::put(handle_step_change))
        .route(
            "/change-step-order",
            axum::routing::put(handle_step_order_change),
        )
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

fn return_to_recipe_edit_error(recipe_id: i32) -> Markup {
    html! {
        div id="error" class="flex flex-col items-center justify-center text-red-500" {
            div {
                h1 { "Error" }
                p { "Failed to edit Recipe" }
            }
            button class="btn btn-primary" hx-get=(format!("/recipes/edit/{}", recipe_id)) hx-swap="outerHTML" hx-target="#error" { "Return to Recipe Edit" }
        }
    }
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
            unit,
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
        unit: state.db_connection.get_unit(0).await.unwrap_or_default(),
    };
    sub_recipe.format_for_subrecipe_table(data.recipe_id)
}

pub async fn handle_step_change(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateRecipeStepHeader>,
) -> Markup {
    state
        .db_connection
        .delete_step(data.recipe_id, data.step_id)
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to update subrecipe {} in recipe {}",
                data.step_id,
                data.recipe_id
            )
        });
    let step = RecipeStep {
        step_id: data.step_id,
        step_order: data.step_order,
        step_name: data.step_name,
        step_description: data.step_description,
        fixed_duration: pg_interval_from_minutes(data.fixed_duration_minutes),
        duration_per_kg: pg_interval_from_minutes(data.duration_per_kg_minutes),
        recipe_id: data.recipe_id,
    };
    step.format_for_step_table(data.recipe_id)
}

pub async fn handle_step_order_change(
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
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn handle_ingredient_delete(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateIngredientHeader>,
) -> Markup {
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
    Form(data): axum::extract::Form<UpdateSubrecipeHeader>,
) -> Markup {
    state
        .db_connection
        .delete_recipe_meta_ingredient(data.recipe_id, data.subrecipe_id)
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to delete subrecipe {} from recipe {}",
                data.subrecipe_id,
                data.recipe_id
            )
        });
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn handle_step_delete(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<RecipeStep>,
) -> Markup {
    state
        .db_connection
        .delete_step(data.recipe_id, data.step_id)
        .await
        .unwrap_or_else(|_| {
            log::warn!(
                "Failed to delete subrecipe {} from recipe {}",
                data.step_id,
                data.recipe_id
            )
        });
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn handle_ingredient_add(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateIngredientHeader>,
) -> Markup {
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
    dbg!(format!(
        "requested name {} yielded ingredient id {}",
        data.ingredient_name, ingredient_id
    ));
    if ingredient_id < 0 {
        add_ingredient_form(State(state), Path(data.recipe_id)).await
    } else {
        let Ok(res) = state
            .db_connection
            .add_recipe_ingredient(
                data.recipe_id,
                ingredient_id,
                data.ingredient_amount,
                data.ingredient_unit_id,
            )
            .await
        else {
            return return_to_recipe_edit_error(data.recipe_id);
        };
        dbg!(res);
        (recipe_edit_view(State(state), Path(data.recipe_id))).await
    }
}

pub async fn handle_subrecipe_add(
    State(state): State<MyAppState>,
    Form(data): axum::extract::Form<UpdateSubrecipeHeader>,
) -> Markup {
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
        let Ok(res) = state
            .db_connection
            .add_recipe_meta_ingredient(data.recipe_id, subrecipe_id, data.subrecipe_amount)
            .await
        else {
            return return_to_recipe_edit_error(data.recipe_id);
        };
        dbg!(res);
        (recipe_edit_view(State(state), Path(data.recipe_id))).await
    }
}

pub async fn handle_step_add(
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
    let Ok(res) = state.db_connection.add_recipe_step(&step).await else {
        return return_to_recipe_edit_error(data.recipe_id);
    };
    dbg!(res);
    (recipe_edit_view(State(state), Path(data.recipe_id))).await
}

pub async fn add_ingredient_form(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> Markup {
    let ingredients = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();
    let unit_types = state.db_connection.get_units().await.unwrap_or_default();
    html! {
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
    }
}

pub async fn add_subrecipe_form(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> Markup {
    let subrecipes = state.db_connection.get_recipes().await.unwrap_or_default();
    html! {
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
    }
}

pub async fn add_step_form(State(_): State<MyAppState>, Path(recipe_id): Path<i32>) -> Markup {
    html! {
        form hx-put="recipes/edit/commit-subrecipe" hx-swap="outerHTML" hx-target="#contents" {
            div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                h1 { "Add Subrecipe" }
                input type="hidden" name=("recipe_id") value=(recipe_id);
                input type="hidden" name=("subrecipe_id") value=(-1);
                input class="text" type="text" name=("step_name") placeholder="Name" value="" required="required";
                input class="text" type="text" name=("step_order") placeholder="Order" value="" required="required";
                input class="text" type="text" name=("step_description") placeholder="Description" value="" required="required";
                input class="text" type="text" name=("fixed_duration") placeholder="Fixed Duration in Minutes" value="" required="required";
                input class="text" type="text" name=("duration_per_kg") placeholder="Duration per kg in Minutes" value="" required="required";
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    }
}

pub async fn recipe_edit_view(
    State(state): State<MyAppState>,
    Path(recipe_id): Path<i32>,
) -> Markup {
    let subrecipes = state
        .db_connection
        .get_recipe_meta_ingredients(recipe_id)
        .await
        .unwrap_or_default();

    let ingredients = state
        .db_connection
        .get_recipe_ingredients(recipe_id)
        .await
        .unwrap_or_default();
    let unit_types = state.db_connection.get_units().await.unwrap_or_default();

    let steps = state
        .db_connection
        .get_recipe_steps(recipe_id)
        .await
        .unwrap_or_default();

    html! {
        div id=("contents") class="flex flex-col items-center justify-center mb-16 w-full"{
            put

            div id=("styling bullshit") class="mb-6" {
                form hx-put=(format!("recipes/edit/add-subrecipe/{}", recipe_id)) hx-swap="outerHTML" class="w-full flex flex-col items-center justify-center pb-4" {
                    input type="hidden" name=("recipe_id") value=(recipe_id);
                    button type="submit" class="btn btn-primary"  { "Add Subrecipe (+)" }
                }
            }

            @if !subrecipes.is_empty() {
                div id=("meta-ingredients") class="w-3/4" {
                    table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                        tbody { @for item in subrecipes { (item.format_for_subrecipe_table(recipe_id)) } }
                    }
                }
            }

            form hx-put=(format!("recipes/edit/add-ingredient/{}", recipe_id)) hx-swap="outerHTML" class="w-full mt-6 flex flex-col items-center justify-center" {
                input type="hidden" name=("recipe_id") value=(recipe_id);
                button type="submit" class="btn btn-primary"  { "Add Ingredient (+)" }
            }

            @if !ingredients.is_empty() {
                div id=("ingredients") class="w-3/4" {
                    table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                        tbody { @for item in ingredients { (item.format_for_ingredient_table(unit_types.clone(), recipe_id)) } }
                    }
                }
            }

            form hx-put=(format!("recipes/edit/add-step/{}", recipe_id)) hx-swap="outerHTML" class="w-full mt-6 flex flex-col items-center justify-center" {
                input type="hidden" name=("recipe_id") value=(recipe_id);
                button type="submit" class="btn btn-primary"  { "Add Step (+)" }
            }

            @if steps.len() > 0 {
                div id=("steps") class="w-3/4" {
                    table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Order" } th { "Name" } th { "Description" } th { "Duration"} th { "Duration per kg" } th { "Delete" } } }
                        tbody { @for item in steps { (item.format_for_step_table(recipe_id)) } }
                    }
                }
            }
        }
    }
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
                td { input class=(format!("text {}",form_id)) name="ingredient_amount" value=(self.amount) required="required" hx-put="recipes/edit/change-ingredient" hx-target=(format!("#ingredient-{}", ingredient_id)) hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML"; }
                td { select class=(format!("unit {} fc-select",form_id)) name="ingredient_unit_id" selected=(self.unit.name) hx-target=(format!("#ingredient-{}", ingredient_id)) hx-swap="outerHTML" required="required" hx-put="recipes/edit/change-ingredient" hx-include=(format!(".{}", form_id)) { @for unit in unit_types {
                    @if unit.unit_id == self.unit.unit_id { option value=(unit.unit_id) selected { (unit.name) } } else { option value=(unit.unit_id) { (unit.name) } } } } }
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
                    div class=(format!("w-full {}",form_id)) name=("subrecipe") { (subrecipe.name) }
                }
                td { input class=(format!("text {}",form_id)) name="subrecipe_amount" value=(self.amount) required="required" hx-put="recipes/edit/change-subrecipe" hx-target=(format!("#subrecipe-{}", subrecipe_id)) hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML"; }
                td { (self.unit.name) }
                td { button class="btn btn-cancel" hx-target="#contents" hx-delete=("recipes/edit/delete-subrecipe") hx-include=(format!(".{}", form_id)) { "Delete" } }
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
                    input class=(form_id) type="hidden" name=("step_name") value=(self.step_name);
                    div class=(format!("w-full {}",form_id)) name=("step_name") { (self.step_name) }
                }
                td { input class=(format!("text {}",form_id)) name="step_order" value=(self.step_order) required="required" hx-put="recipes/edit/change-step-order" hx-target="#contents" hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML"; }
                td { input class=(format!("text {}",form_id)) name="step_description" value=(self.step_description) required="required" hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML"; }
                td { input class=(format!("text {}",form_id)) name="fixed-duration" value=(format!("{} min", (self.fixed_duration.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML"; }
                td { input class=(format!("text {}",form_id)) name="duration_per_kg" value=(format!("{} min", (self.duration_per_kg.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="recipes/edit/change-step" hx-target=(format!("#step-{}", self.step_id)) hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML"; }
                td { button class="btn btn-cancel" hx-target="#contents" hx-delete=("recipes/edit/delete-subrecipe") hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}
