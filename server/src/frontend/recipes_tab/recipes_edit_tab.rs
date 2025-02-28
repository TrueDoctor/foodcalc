use crate::FoodLib;
use axum::{
    extract::{Form, Path},
    routing::{delete, put},
};
use bigdecimal::BigDecimal;
use foodlib_new::{
    recipe::{RecipeIngredient, RecipeMetaIngredient, RecipeStep},
    unit::Unit,
};
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
        .route(
            "/delete-ingredient/{recipe_id}/{ingredient_id}",
            delete(handle_ingredient_delete),
        )
        .route(
            "/delete-subrecipe/{recipe_id}/{subrecipe_id}",
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

impl From<UpdateIngredientHeader> for RecipeIngredient {
    fn from(value: UpdateIngredientHeader) -> Self {
        Self {
            recipe_id: value.recipe_id,
            ingredient_id: value.ingredient_id,
            amount: value.ingredient_amount,
            unit_id: value.ingredient_unit_id,
            name: Some(value.ingredient_name),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateSubrecipeHeader {
    pub recipe_id: i32,
    pub subrecipe_id: i32,
    pub subrecipe_name: String,
    pub subrecipe_amount: BigDecimal,
}

impl From<UpdateSubrecipeHeader> for RecipeMetaIngredient {
    fn from(value: UpdateSubrecipeHeader) -> Self {
        Self {
            parent_id: value.recipe_id,
            child_id: value.subrecipe_id,
            weight: value.subrecipe_amount,
            name: Some(value.subrecipe_name),
        }
    }
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

impl From<UpdateRecipeStepHeader> for RecipeStep {
    fn from(value: UpdateRecipeStepHeader) -> Self {
        RecipeStep {
            id: value.step_id,
            order: value.step_order,
            name: value.step_name,
            description: value.step_description,
            fixed_duration: pg_interval_from_minutes(value.fixed_duration_minutes),
            duration_per_kg: pg_interval_from_minutes(value.duration_per_kg_minutes),
            recipe_id: value.recipe_id,
        }
    }
}

fn pg_interval_from_minutes(minutes: f64) -> PgInterval {
    PgInterval {
        months: 0,
        days: 0,
        microseconds: (60_000_000. * minutes) as i64,
    }
}

pub async fn handle_name_change(
    foodlib: FoodLib,
    Form(data): Form<UpdateNameHeader>,
) -> Result<(), foodlib_new::Error> {
    foodlib
        .recipes()
        .update(foodlib_new::recipe::Recipe {
            id: data.recipe_id,
            name: data.name,
            comment: Some(data.comment),
            owner_id: -1, // We don't change the owner
        })
        .await?;
    Ok(())
}

pub async fn handle_ingredient_change(
    foodlib: FoodLib,
    Form(data): Form<UpdateIngredientHeader>,
) -> MResponse {
    let recipe_ingredient = foodlib.recipes().update_ingredient(data.into()).await?;

    Ok(recipe_ingredient.format_for_ingredient_table(
        foodlib.units().list().await.unwrap_or_default().clone(),
        recipe_ingredient.recipe_id,
    ))
}

pub async fn handle_subrecipe_change(
    foodlib: FoodLib,
    Form(data): Form<UpdateSubrecipeHeader>,
) -> MResponse {
    let sub_recipe = foodlib
        .recipes()
        .update_meta_ingredient(data.into())
        .await?;

    Ok(sub_recipe.format_for_subrecipe_table())
}

pub async fn handle_step_change(
    foodlib: FoodLib,
    Form(data): Form<UpdateRecipeStepHeader>,
) -> MResponse {
    let recipe_id = data.recipe_id;
    let step = foodlib.recipes().update_step(data.into()).await?;

    Ok(step.format_for_step_table(recipe_id))
}

pub async fn handle_step_order_change(
    foodlib: FoodLib,
    Form(data): Form<UpdateRecipeStepHeader>,
) -> MResponse {
    let recipe_id = data.recipe_id;
    foodlib.recipes().update_step(data.into()).await?;

    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn handle_ingredient_delete(
    foodlib: FoodLib,
    Path((recipe_id, ingredient_id)): Path<(i32, i32)>,
) -> MResponse {
    foodlib
        .recipes()
        .delete_ingredient(recipe_id, ingredient_id)
        .await?;

    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn handle_subrecipe_delete(
    foodlib: FoodLib,
    Path((recipe_id, subrecipe_id)): Path<(i32, i32)>,
) -> MResponse {
    foodlib
        .recipes()
        .delete_meta_ingredient(recipe_id, subrecipe_id)
        .await?;

    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn handle_step_delete(
    foodlib: FoodLib,
    Path((recipe_id, step_id)): Path<(i32, i32)>,
) -> MResponse {
    foodlib.recipes().delete_step(recipe_id, step_id).await?;

    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn handle_ingredient_add(
    foodlib: FoodLib,
    Form(mut data): Form<UpdateIngredientHeader>,
) -> MResponse {
    let ingredient = foodlib
        .ingredients()
        .get_by_name(&data.ingredient_name)
        .await?;

    let recipe_id = data.recipe_id;
    data.ingredient_id = ingredient.id;
    foodlib.recipes().add_ingredient(data.into()).await?;

    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn handle_subrecipe_add(
    foodlib: FoodLib,
    Form(mut data): Form<UpdateSubrecipeHeader>,
) -> MResponse {
    let subrecipe = foodlib.recipes().get_by_name(&data.subrecipe_name).await?;
    let recipe_id = data.recipe_id;
    data.subrecipe_id = subrecipe.id;
    foodlib.recipes().add_meta_ingredient(data.into()).await?;

    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn handle_step_add(
    foodlib: FoodLib,
    Form(data): Form<UpdateRecipeStepHeader>,
) -> MResponse {
    let recipe_id = data.recipe_id;
    foodlib.recipes().add_step(data.into()).await?;
    recipe_edit_view(foodlib, Path(recipe_id)).await
}

pub async fn add_ingredient_form(foodlib: FoodLib, Path(recipe_id): Path<i32>) -> MResponse {
    let ingredients = foodlib.ingredients().list().await?;
    let unit_types = foodlib.units().list().await?;

    Ok(html! {
        form hx-put="/recipes/edit/commit-ingredient" hx-swap="outerHTML" hx-target="#contents" {
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
                        option value=(unit.id) { (unit.name) }
                    }
                }
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    })
}

pub async fn add_subrecipe_form(foodlib: FoodLib, Path(recipe_id): Path<i32>) -> MResponse {
    let subrecipes = foodlib.recipes().list().await?;

    Ok(html! {
        form hx-put="/recipes/edit/commit-subrecipe" hx-swap="outerHTML" hx-target="#contents" {
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

pub async fn add_step_form(Path(recipe_id): Path<i32>) -> Markup {
    html! {
        form id="test5" hx-put="/recipes/edit/commit-step" hx-swap="outerHTML" hx-target="#contents" {
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

pub async fn recipe_edit_view(foodlib: FoodLib, Path(recipe_id): Path<i32>) -> MResponse {
    let subrecipes = foodlib.recipes().get_meta_ingredients(recipe_id).await?;

    let ingredients = foodlib.recipes().get_ingredients(recipe_id).await?;
    let unit_types = foodlib.units().list().await?;
    let steps = foodlib.recipes().get_steps(recipe_id).await?;
    // The stats might not exist yet if we are creating a new recipe
    let stats = foodlib
        .recipes()
        .get_recipe_stats(recipe_id)
        .await
        .unwrap_or_default();
    let recipe = foodlib.recipes().get(recipe_id).await?;

    Ok(html! {
        div id=("contents") class="flex flex-col items-center justify-center mb-16 w-full"{
            div id=("recipe-information") class="w-3/4" {
                form hx-put="/recipes/edit/change-name" hx-indicator=".htmx-indicator" hx-swap="none" class="w-full flex flex-col mb-4 pb-4 gap-2" {
                    input type="hidden" name=("recipe_id") value=(recipe_id);
                    input class="text" type="text" name="name" value=(recipe.name) required="required";
                    textarea class="text" name="comment" { (recipe.comment.unwrap_or_default()) }
                    button type="submit" class="btn btn-primary"  { "Change Name and Comment" }}
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

            div id="styling-bullshit" class="mb-6 mt-6 w-1/4" {
                form hx-put=(format!("/recipes/edit/add-subrecipe/{}", recipe_id)) hx-swap="outerHTML" hx-target="#styling-bullshit" class="w-full flex flex-col items-center justify-center pb-4" {
                    input type="hidden" name=("recipe_id") value=(recipe_id);
                    button type="submit" class="btn btn-primary"  { "Add Subrecipe (+)" }
                }
            }


            span class="htmx-indicator" { "Saving\u{a0}…" }

            @if !subrecipes.is_empty() {
                div id=("meta-ingredients") class="w-3/4" {
                    table class="text-inherit table-auto object-center table-fixed" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                        tbody { @for item in subrecipes { (item.format_for_subrecipe_table()) } }
                    }
                }
            }

            form hx-put=(format!("/recipes/edit/add-ingredient/{}", recipe_id)) hx-swap="outerHTML" class="w-1/4 mt-6 flex flex-col items-center justify-center" {
                input type="hidden" name=("recipe_id") value=(recipe_id);
                button type="submit" class="btn btn-primary"  { "Add Ingredient (+)" }
            }

            @if !ingredients.is_empty() {
                div id=("ingredients") class="w-3/4" {
                    table class="text-inherit table-auto object-center table-fixed" padding="0 0.5em" display="block" max-height="60vh" overflow-y="scroll" {
                        thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                        tbody { @for item in ingredients { (item.format_for_ingredient_table(unit_types.clone(), recipe_id,)) } }
                    }
                }
            }

            form hx-put=(format!("/recipes/edit/add-step/{}", recipe_id)) hx-swap="outerHTML" class="w-1/4 mt-6 flex flex-col items-center justify-center" {
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
        let ingredient = self;
        let ingredient_id = ingredient.ingredient_id;
        let form_id = format!("ingredient-{}-form", ingredient_id);
        let unit = unit_types
            .iter()
            .find(|u| u.id == self.unit_id)
            .unwrap_or(&unit_types[0]);
        html! {
            tr id=(format!("ingredient-{}", ingredient_id)) style="text-align:center"{ // TODO: Put into form
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                    input class=(form_id) type="hidden" name=("ingredient_id") value=(ingredient_id);
                    input class=(form_id) type="hidden" name=("ingredient_name") value=(self.name.clone().unwrap_or_default());
                    div class=(format!("w-full {}",form_id)) name=("ingredient") { (self.name.clone().unwrap_or_default()) }
                }
                td { input class=(format!("text {}",form_id)) name="ingredient_amount" value=(self.amount) required="required" hx-put="/recipes/edit/change-ingredient" hx-indicator=".htmx-indicator" hx-target=(format!("#ingredient-{}", ingredient_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML"; }
                td { select class=(format!("unit {} fc-select",form_id)) name="ingredient_unit_id" selected=(unit.name) hx-target=(format!("#ingredient-{}", ingredient_id)) hx-swap="outerHTML" required="required" hx-put="/recipes/edit/change-ingredient" hx-indicator=".htmx-indicator" hx-include=(format!(".{}", form_id)) { @for unit in unit_types {
                    @if unit.id == self.unit_id { option value=(unit.id) selected { (unit.name) } } @else { option value=(unit.id) { (unit.name) } } } } }
                td { button class="btn btn-cancel" hx-target="#contents" hx-delete=(format!("/recipes/edit/delete-ingredient/{recipe_id}/{ingredient_id}"))  { "Delete" } }
            }
        }
    }
}

pub trait SubRecipeTableFormattable {
    fn format_for_subrecipe_table(&self) -> Markup;
}

impl SubRecipeTableFormattable for RecipeMetaIngredient {
    fn format_for_subrecipe_table(&self) -> Markup {
        let subrecipe = self;
        let subrecipe_id = subrecipe.child_id;
        let recipe_id = self.parent_id;
        let form_id = format!("subrecipe-{}-form", subrecipe_id);
        let amount = &self.weight;
        let name = self.name.clone().unwrap_or_default();
        html! {
            tr id=(format!("subrecipe-{}", subrecipe_id)) style="text-align:center"{ // TODO: Put into form
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                    input class=(form_id) type="hidden" name=("subrecipe_id") value=(subrecipe_id);
                    input class=(form_id) type="hidden" name=("subrecipe_name") value=(name);
                    input class=(form_id) type="hidden" name=("subrecipe_unit_id") value=(0);
                    div class=(format!("w-full {}",form_id)) name=("subrecipe") { a hx-target="#content" hx-get=(format!("/recipes/edit/{}", subrecipe_id)) { (name) } }
                }
                td { input class=(format!("text {}",form_id)) name="subrecipe_amount" value=(amount) required="required" hx-put="/recipes/edit/change-subrecipe" hx-target=(format!("#subrecipe-{}", subrecipe_id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-indicator=".htmx-indicator" hx-swap="outerHTML"; }
                td { "kg" }
                td { button class="btn btn-cancel" hx-target="#contents" type="button" hx-delete=(format!("/recipes/edit/delete-subrecipe/{}/{}", recipe_id, subrecipe_id)) hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}

pub trait StepTableFormattable {
    fn format_for_step_table(&self, recipe_id: i32) -> Markup;
}

impl StepTableFormattable for RecipeStep {
    fn format_for_step_table(&self, recipe_id: i32) -> Markup {
        let form_id = format!("step-{}-form", self.id);
        html! {
            tr id=(format!("step-{}", self.id)) style="text-align:center"{
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                    input class=(form_id) type="hidden" name=("step_id") value=(self.id);
                    input class=(format!("text {}",form_id)) name="step_order" value=(self.order) required="required" hx-put="/recipes/edit/change-step-order" hx-target="#contents" hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator";
                }
                td { input class=(format!("text {}",form_id)) name=("step_name") value=(self.name) required="required" hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="step_description" value=(self.description) hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="fixed_duration_minutes" value=(format!("{}", (self.fixed_duration.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="duration_per_kg_minutes" value=(format!("{}", (self.duration_per_kg.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { button class="btn btn-cancel" hx-target="#contents" type="button" hx-delete=(format!("/recipes/edit/delete-step/{}/{}", recipe_id, self.id)) hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}
