use crate::FoodLib;
use axum::{
    extract::{Form, Path},
    routing::{delete, put},
};
use bigdecimal::BigDecimal;
use foodlib_new::{
    auth_context::AuthCtx,
    recipe::{RecipeIngredient, RecipeMetaIngredient, RecipeStep},
    unit::Unit,
};
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgInterval;

use crate::{
    frontend::{move_group, MResponse},
    MyAppState,
};

pub(crate) fn recipes_edit_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/{recipe_id}", axum::routing::get(recipe_edit_view))
        .route("/commit-ingredient", put(handle_ingredient_add))
        .route("/commit-subrecipe", put(handle_subrecipe_add))
        .route("/commit-step", put(handle_step_add))
        .route("/reorder-step", put(handle_step_reorder))
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
}

#[derive(Deserialize, Debug)]
pub struct UpdateNameHeader {
    pub recipe_id: i32,
    pub name: String,
    pub comment: String,
    pub group_id: i32,
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
    ctx: AuthCtx,
    Form(data): Form<UpdateNameHeader>,
) -> Result<(), foodlib_new::Error> {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let current = foodlib.recipes().get(data.recipe_id).await?;
    if current.group_id != data.group_id {
        move_group::assert_can_move_to(&ctx, data.group_id)?;
    }
    foodlib
        .recipes()
        .update(foodlib_new::recipe::Recipe {
            id: data.recipe_id,
            name: data.name,
            comment: Some(data.comment),
            group_id: -1, // Placeholder; ops-level update ignores group_id
        })
        .await?;
    if current.group_id != data.group_id {
        foodlib.recipes().set_group(data.recipe_id, data.group_id).await?;
    }
    Ok(())
}

pub async fn handle_ingredient_change(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(data): Form<UpdateIngredientHeader>,
) -> MResponse {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let recipe_ingredient = foodlib.recipes().update_ingredient(data.into()).await?;

    Ok(recipe_ingredient.format_for_ingredient_table(
        foodlib.units().list().await.unwrap_or_default().clone(),
        recipe_ingredient.recipe_id,
    ))
}

pub async fn handle_subrecipe_change(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(data): Form<UpdateSubrecipeHeader>,
) -> MResponse {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let sub_recipe = foodlib
        .recipes()
        .update_meta_ingredient(data.into())
        .await?;

    Ok(sub_recipe.format_for_subrecipe_table())
}

pub async fn handle_step_change(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(data): Form<UpdateRecipeStepHeader>,
) -> MResponse {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let recipe_id = data.recipe_id;
    let step = foodlib.recipes().update_step(data.into()).await?;

    Ok(step.format_for_step_table(recipe_id))
}

/// DnD reorder. Reads only the new `step_order` (client-computed midpoint)
/// and persists it; leaves all other step fields untouched by loading the
/// step and mutating just the order before update.
#[derive(Deserialize)]
struct ReorderStepForm {
    recipe_id: i32,
    step_id: i32,
    step_order: f64,
}

async fn handle_step_reorder(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(form): Form<ReorderStepForm>,
) -> Result<(), foodlib_new::Error> {
    ctx.assert_can_edit_recipe(form.recipe_id).await?;
    let mut step = foodlib
        .recipes()
        .get_steps(form.recipe_id)
        .await?
        .into_iter()
        .find(|s| s.id == form.step_id)
        .ok_or_else(|| foodlib_new::Error::NotFound {
            entity: "RecipeStep",
            id: form.step_id.to_string(),
        })?;
    step.order = form.step_order;
    foodlib.recipes().update_step(step).await?;
    Ok(())
}

pub async fn handle_ingredient_delete(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((recipe_id, ingredient_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_recipe(recipe_id).await?;
    foodlib
        .recipes()
        .delete_ingredient(recipe_id, ingredient_id)
        .await?;

    recipe_edit_view(foodlib, ctx, Path(recipe_id)).await
}

pub async fn handle_subrecipe_delete(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((recipe_id, subrecipe_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_recipe(recipe_id).await?;
    foodlib
        .recipes()
        .delete_meta_ingredient(recipe_id, subrecipe_id)
        .await?;

    recipe_edit_view(foodlib, ctx, Path(recipe_id)).await
}

pub async fn handle_step_delete(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path((recipe_id, step_id)): Path<(i32, i32)>,
) -> MResponse {
    ctx.assert_can_edit_recipe(recipe_id).await?;
    foodlib.recipes().delete_step(recipe_id, step_id).await?;

    recipe_edit_view(foodlib, ctx, Path(recipe_id)).await
}

pub async fn handle_ingredient_add(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(mut data): Form<UpdateIngredientHeader>,
) -> MResponse {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let ingredient = foodlib
        .ingredients()
        .get_by_name(&data.ingredient_name)
        .await?;

    let recipe_id = data.recipe_id;
    data.ingredient_id = ingredient.id;
    foodlib.recipes().add_ingredient(data.into()).await?;

    recipe_edit_view(foodlib, ctx, Path(recipe_id)).await
}

pub async fn handle_subrecipe_add(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(mut data): Form<UpdateSubrecipeHeader>,
) -> MResponse {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let subrecipe = foodlib.recipes().get_by_name(&data.subrecipe_name).await?;
    let recipe_id = data.recipe_id;
    data.subrecipe_id = subrecipe.id;
    foodlib.recipes().add_meta_ingredient(data.into()).await?;

    recipe_edit_view(foodlib, ctx, Path(recipe_id)).await
}

pub async fn handle_step_add(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Form(mut data): Form<UpdateRecipeStepHeader>,
) -> MResponse {
    ctx.assert_can_edit_recipe(data.recipe_id).await?;
    let recipe_id = data.recipe_id;
    // Order is no longer user-supplied — append after the current max.
    let existing = foodlib.recipes().get_steps(recipe_id).await?;
    data.step_order = existing
        .iter()
        .map(|s| s.order)
        .fold(0.0_f64, f64::max)
        + 1.0;
    foodlib.recipes().add_step(data.into()).await?;
    recipe_edit_view(foodlib, ctx, Path(recipe_id)).await
}

/// First-row inline add for the recipe-ingredients table. Sentinel id -1; the
/// commit handler looks the ingredient up by name (since users pick from the
/// datalist) and rewrites the id before insert.
fn recipe_ingredient_add_row(
    recipe_id: i32,
    all_ingredients: &[foodlib_new::ingredient::Ingredient],
    unit_types: &[Unit],
) -> Markup {
    let list_id = format!("ingredient-add-datalist-{}", recipe_id);
    html! {
        tr id="recipe-ingredient--1" {
            input type="hidden" name="recipe_id" value=(recipe_id);
            input type="hidden" name="ingredient_id" value="-1";
            datalist id=(list_id) {
                @for ing in all_ingredients { option value=(ing.name) {} }
            }
            td {
                input class="text w-full" type="text" list=(list_id)
                    name="ingredient_name" placeholder="Ingredient" required="required";
            }
            td {
                input class="text w-full" type="text" name="ingredient_amount"
                    placeholder="Amount" required="required";
            }
            td {
                select class="unit fc-select w-full" name="ingredient_unit_id" required="required" {
                    @for unit in unit_types { option value=(unit.id) { (unit.name) } }
                }
            }
            td {
                button class="btn btn-primary"
                    hx-put="/recipes/edit/commit-ingredient"
                    hx-include="closest tr"
                    hx-target="#contents"
                    hx-swap="outerHTML" { "Add" }
            }
        }
    }
}

/// First-row inline add for the recipe-subrecipes table. Sentinel id -1; the
/// commit handler resolves the subrecipe id by name lookup.
fn recipe_subrecipe_add_row(
    recipe_id: i32,
    all_recipes: &[foodlib_new::recipe::Recipe],
) -> Markup {
    let list_id = format!("subrecipe-add-datalist-{}", recipe_id);
    html! {
        tr id="recipe-subrecipe--1" {
            input type="hidden" name="recipe_id" value=(recipe_id);
            input type="hidden" name="subrecipe_id" value="-1";
            datalist id=(list_id) {
                @for r in all_recipes {
                    @if r.id != recipe_id { option value=(r.name) {} }
                }
            }
            td {
                input class="text w-full" type="text" list=(list_id)
                    name="subrecipe_name" placeholder="Subrecipe" required="required";
            }
            td {
                input class="text w-full" type="text" name="subrecipe_amount"
                    placeholder="Amount (kg)" required="required";
            }
            td { "kg" }
            td {
                button class="btn btn-primary"
                    hx-put="/recipes/edit/commit-subrecipe"
                    hx-include="closest tr"
                    hx-target="#contents"
                    hx-swap="outerHTML" { "Add" }
            }
        }
    }
}

pub async fn recipe_edit_view(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(recipe_id): Path<i32>,
) -> MResponse {
    ctx.assert_can_edit_recipe(recipe_id).await?;
    let subrecipes = foodlib.recipes().get_meta_ingredients(recipe_id).await?;

    let ingredients = foodlib.recipes().get_ingredients(recipe_id).await?;
    let all_ingredients = foodlib.ingredients().list().await?;
    let all_recipes = foodlib.recipes().list().await?;
    let unit_types = foodlib.units().list().await?;
    let steps = foodlib.recipes().get_steps(recipe_id).await?;
    // The stats might not exist yet if we are creating a new recipe
    let stats = foodlib
        .recipes()
        .get_recipe_stats(recipe_id)
        .await
        .unwrap_or_default();
    let recipe = foodlib.recipes().get(recipe_id).await?;
    let owner_select = move_group::owner_select(&foodlib, &ctx, recipe.group_id).await?;

    Ok(html! {
        div id=("contents") class="flex flex-col items-center justify-center mb-16 w-full"{
            div id=("recipe-information") class="w-3/4" {
                form hx-put="/recipes/edit/change-name" hx-indicator=".htmx-indicator" hx-swap="none" class="w-full flex flex-col mb-4 pb-4 gap-2" {
                    input type="hidden" name=("recipe_id") value=(recipe_id);
                    input class="text" type="text" name="name" value=(recipe.name) required="required";
                    textarea class="text" name="comment" { (recipe.comment.unwrap_or_default()) }
                    div class="flex flex-row items-center gap-2" { (owner_select) }
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

            span class="htmx-indicator" { "Saving\u{a0}…" }

            div id="meta-ingredients" class="w-3/4 mt-6" {
                h3 class="text-lg font-semibold mb-2" { "Subrecipes" }
                table class="text-inherit table-auto object-center table-fixed w-full" {
                    thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                    tbody {
                        (recipe_subrecipe_add_row(recipe_id, &all_recipes))
                        @for item in subrecipes { (item.format_for_subrecipe_table()) }
                    }
                }
            }

            div id="ingredients" class="w-3/4 mt-6" {
                h3 class="text-lg font-semibold mb-2" { "Ingredients" }
                table class="text-inherit table-auto object-center table-fixed w-full" {
                    thead { tr { th { "Name" } th { "Amount" } th { "Unit" } th { "Delete" } } }
                    tbody {
                        (recipe_ingredient_add_row(recipe_id, &all_ingredients, &unit_types))
                        @for item in ingredients { (item.format_for_ingredient_table(unit_types.clone(), recipe_id,)) }
                    }
                }
            }

            div id="steps" class="w-3/4 mt-6" {
                h3 class="text-lg font-semibold mb-2" { "Steps" }
                table class="text-inherit table-auto object-center table-fixed w-full" {
                    thead { tr {
                        th class="w-8" {}
                        th { "Name" }
                        th { "Description" }
                        th { "Duration" }
                        th { "Duration per kg" }
                        th { "Delete" }
                    } }
                    tbody id="steps-tbody" class="recipe-steps" data-recipe-id=(recipe_id) {
                        (recipe_step_add_row(recipe_id))
                        @for item in &steps { (item.format_for_step_table(recipe_id)) }
                    }
                }
            }
            (steps_dnd_script(recipe_id))
            span class="htmx-indicator" { "Saving..." }
        }
    })
}

/// Inline JS that wires SortableJS to the steps tbody. On drop, computes the
/// midpoint of the new neighbours' `data-step-order` values (or +1 at the
/// end / -1 at the beginning) and PUTs it to /reorder-step. Uses Sortable's
/// own drag handle (the `.step-drag-handle` cell) so the row inputs stay
/// editable.
fn steps_dnd_script(recipe_id: i32) -> Markup {
    html! {
        script {
            (maud::PreEscaped(format!(r#"
            (function() {{
                const tbody = document.getElementById('steps-tbody');
                if (!tbody || !window.Sortable || tbody.dataset.sortableReady === '1') return;
                tbody.dataset.sortableReady = '1';
                Sortable.create(tbody, {{
                    draggable: '.step-row',
                    handle: '.step-drag-handle',
                    filter: '.add-row',
                    preventOnFilter: false,
                    ghostClass: 'fc-ghost',
                    chosenClass: 'fc-chosen',
                    dragClass: 'fc-drag',
                    animation: 150,
                    onEnd: function(evt) {{
                        const row = evt.item;
                        const stepId = row.dataset.stepId;
                        if (!stepId) return;
                        const siblings = Array.from(tbody.querySelectorAll('.step-row'));
                        const idx = siblings.indexOf(row);
                        const prev = siblings[idx - 1];
                        const next = siblings[idx + 1];
                        const prevOrder = prev ? parseFloat(prev.dataset.stepOrder) : null;
                        const nextOrder = next ? parseFloat(next.dataset.stepOrder) : null;
                        let newOrder;
                        if (prevOrder === null && nextOrder === null) newOrder = 1.0;
                        else if (prevOrder === null) newOrder = nextOrder - 1.0;
                        else if (nextOrder === null) newOrder = prevOrder + 1.0;
                        else newOrder = (prevOrder + nextOrder) / 2.0;
                        row.dataset.stepOrder = newOrder;
                        const hidden = row.querySelector('input[name=step_order]');
                        if (hidden) hidden.value = newOrder;
                        htmx.ajax('PUT', '/recipes/edit/reorder-step', {{
                            target: '#contents',
                            swap: 'none',
                            values: {{ recipe_id: {recipe_id}, step_id: stepId, step_order: newOrder }}
                        }});
                    }}
                }});
            }})();
            "#)))
        }
    }
}

/// First-row inline add for the steps table. Order is computed server-side
/// (max existing + 1.0); the form-supplied step_order=0 is ignored.
fn recipe_step_add_row(recipe_id: i32) -> Markup {
    html! {
        tr id="recipe-step--1" class="add-row" {
            input type="hidden" name="recipe_id" value=(recipe_id);
            input type="hidden" name="step_id" value="-1";
            input type="hidden" name="step_order" value="0";
            td {}
            td { input class="text w-full" type="text" name="step_name" placeholder="Name" required="required"; }
            td { input class="text w-full" type="text" name="step_description" placeholder="Description"; }
            td { input class="text w-full" type="text" name="fixed_duration_minutes" placeholder="Fixed (min)" required="required"; }
            td { input class="text w-full" type="text" name="duration_per_kg_minutes" placeholder="Per kg (min)" required="required"; }
            td {
                button class="btn btn-primary"
                    hx-put="/recipes/edit/commit-step"
                    hx-include="closest tr"
                    hx-target="#contents"
                    hx-swap="outerHTML" { "Add" }
            }
        }
    }
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
            tr id=(format!("step-{}", self.id)) class="step-row" data-step-id=(self.id) data-step-order=(self.order) style="text-align:center"{
                input class=(form_id) type="hidden" name=("recipe_id") value=(recipe_id);
                input class=(form_id) type="hidden" name=("step_id") value=(self.id);
                input class=(form_id) type="hidden" name=("step_order") value=(self.order);
                td class="step-drag-handle text-center cursor-grab opacity-60 select-none" title="Drag to reorder" { "⋮⋮" }
                td { input class=(format!("text {}",form_id)) name=("step_name") value=(self.name) required="required" hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="step_description" value=(self.description) hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="fixed_duration_minutes" value=(format!("{}", (self.fixed_duration.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { input class=(format!("text {}",form_id)) name="duration_per_kg_minutes" value=(format!("{}", (self.duration_per_kg.microseconds / 1_000_000) as f64 / 60.)) required="required" hx-put="/recipes/edit/change-step" hx-target=(format!("#step-{}", self.id)) hx-include=(format!(".{}", form_id)) hx-trigger="change" hx-swap="outerHTML" hx-indicator=".htmx-indicator"; }
                td { button class="btn btn-cancel" hx-target="#contents" type="button" hx-delete=(format!("/recipes/edit/delete-step/{}/{}", recipe_id, self.id)) hx-include=(format!(".{}", form_id)) { "Delete" } }
            }
        }
    }
}
