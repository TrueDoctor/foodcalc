//! Recipe use-cases.

use crate::ctx::{assert_can_move_to, AppCtx};
use crate::entities::allergen::DietaryFlags;
use crate::entities::ingredient::Ingredient;
use crate::entities::recipe::{
    Recipe, RecipeData, RecipeIngredient, RecipeKind, RecipeMetaIngredient, RecipeStats, RecipeStep,
};
use crate::entities::unit::Unit;
use crate::ops::allergens::compact_allergen_set;
use crate::types::amount::Amount;
use crate::types::ids::{GroupId, IngredientId, RecipeId, RecipeStepId, UnitId};
use crate::{AppState, Error, Result};
use time::Duration;
use crate::entities::property::RecipeProperties;

fn dedup_sorted(mut items: Vec<String>) -> Vec<String> {
    items.sort();
    items.dedup();
    items
}

fn build_step(
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    order: f64,
    name: String,
    description: String,
    fixed_duration_minutes: f64,
    duration_per_kg_minutes: f64,
) -> RecipeStep {
    RecipeStep {
        id: step_id,
        order,
        name,
        description,
        fixed_duration: Duration::seconds((fixed_duration_minutes * 60.0).round() as i64),
        duration_per_kg: Duration::seconds((duration_per_kg_minutes * 60.0).round() as i64),
        recipe_id,
    }
}

/// Everything the recipe overview/list renders: all recipes plus the group
/// ids of the acting user (empty for anonymous callers), fetched in one use
/// case so transports only have to deal with presentation.
pub async fn gather_overview(
    state: &AppState,
    actor: Option<&AppCtx>,
) -> Result<(Vec<Recipe>, Vec<GroupId>)> {
    let recipes = state.db().recipes().list().await?;
    let user_group_ids = actor.map(|a| a.group_ids.clone()).unwrap_or_default();
    Ok((recipes, user_group_ids))
}

/// All measurement units (used to render unit pickers).
pub async fn list_units(state: &AppState) -> Result<Vec<Unit>> {
    state.db().units().list().await
}

/// Lists all recipes.
pub async fn list_recipes(state: &AppState) -> Result<Vec<Recipe>> {
    state.db().recipes().list().await
}

/// Fetches one recipe.
pub async fn get_recipe(state: &AppState, recipe_id: RecipeId) -> Result<Recipe> {
    state.db().recipes().get(recipe_id).await
}

/// Everything the recipe allergen dialog renders. The raw property labels
/// are classified here (regulatory "contains" vs "~spuren-" may-contain vs
/// informational "~hinweis-" notes) so transports only render.
pub struct RecipeAllergenData {
    pub recipe_name: String,
    pub dietary: DietaryFlags,
    pub contains: Vec<String>,
    pub may_contain: Vec<String>,
    pub hinweise: Vec<String>,
}

pub async fn gather_recipe_allergens(
    state: &AppState,
    recipe_id: RecipeId,
) -> Result<RecipeAllergenData> {
    let rp = state
        .db()
        .properties()
        .get_recipe_properties(recipe_id)
        .await?;
    let dietary = DietaryFlags::from_properties(&rp.properties);

    let mut contains_lower = Vec::new();
    let mut may_lower = Vec::new();
    let mut hinweis_lower = Vec::new();
    for p in &rp.properties {
        let lower = p.name.to_lowercase();
        if let Some(rest) = lower.strip_prefix("~spuren-") {
            may_lower.push(rest.to_string());
        } else if let Some(rest) = lower.strip_prefix("~hinweis-") {
            hinweis_lower.push(rest.to_string());
        } else {
            contains_lower.push(lower);
        }
    }

    Ok(RecipeAllergenData {
        recipe_name: rp.recipe_name,
        dietary,
        contains: compact_allergen_set(&dedup_sorted(contains_lower)),
        may_contain: compact_allergen_set(&dedup_sorted(may_lower)),
        hinweise: dedup_sorted(hinweis_lower),
    })
}

/// Everything the recipe edit view renders, fetched in one use case so
/// transports only have to deal with presentation.
pub struct EditViewData {
    pub recipe: Recipe,
    pub sub_recipes: Vec<RecipeMetaIngredient>,
    pub ingredients: Vec<RecipeIngredient>,
    pub all_ingredients: Vec<Ingredient>,
    pub all_recipes: Vec<Recipe>,
    pub unit_types: Vec<Unit>,
    pub steps: Vec<RecipeStep>,
    pub stats: RecipeStats,
}

/// Creates a new empty meal recipe owned by the actor's personal group and
/// returns its id.
pub async fn create_recipe(
    state: &AppState,
    actor: &AppCtx,
    name: String,
    comment: Option<String>,
) -> Result<RecipeId> {
    let group = state.db().users().get_personal_group(actor.user.id).await?;
    let recipe = state
        .db()
        .recipes()
        .create(
            group.id,
            RecipeData {
                name,
                comment,
                weighed_in: false,
                kind: RecipeKind::Meal,
            },
        )
        .await?;
    Ok(recipe.id)
}

/// Collects all data needed to render the recipe edit view.
/// Requires `EditRecipe` permission on the recipe's owner group.
pub async fn gather_edit_view_data(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
) -> Result<EditViewData> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    let recipes_ops = state.db().recipes();

    let sub_recipes = recipes_ops.get_meta_ingredients(recipe_id).await?;
    let ingredients = recipes_ops.get_ingredients(recipe_id).await?;
    let all_ingredients = state.db().ingredients().list().await?;
    let all_recipes = recipes_ops.list().await?;
    let unit_types = state.db().units().list().await?;
    let steps = recipes_ops.get_steps(recipe_id).await?;
    // The stats might not exist yet if we are creating a new recipe
    let stats = recipes_ops
        .get_recipe_stats(recipe_id)
        .await
        .unwrap_or_default();
    let recipe = recipes_ops.get(recipe_id).await?;

    Ok(EditViewData {
        recipe,
        sub_recipes,
        ingredients,
        all_ingredients,
        all_recipes,
        unit_types,
        steps,
        stats,
    })
}

/// Renames a recipe, updates its comment and "weighed in" flag, and optionally
/// transfers ownership to another group. Requires `EditRecipe` permission on
/// the recipe's current owner group; moving to another group additionally
/// requires membership in (or admin over) the target group.
pub async fn update_name(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    name: String,
    comment: String,
    group_id: GroupId,
    weighed_in: bool,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;

    let current = state.db().recipes().get(recipe_id).await?;
    if current.group_id != group_id {
        assert_can_move_to(actor, group_id)?;
    }

    state
        .db()
        .recipes()
        .update(
            recipe_id,
            RecipeData {
                name,
                comment: (!comment.is_empty()).then_some(comment),
                weighed_in,
                kind: current.kind,
            },
        )
        .await?;
    if current.group_id != group_id {
        state.db().recipes().set_group(recipe_id, group_id).await?;
    }
    Ok(())
}

/// Updates one recipe ingredient row (amount and unit). Returns the updated
/// ingredient for the caller's presentation layer.
/// Requires `EditRecipe` permission.
pub async fn update_ingredient(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    ingredient_id: IngredientId,
    ingredient_name: String,
    amount: Amount,
    unit_id: UnitId,
) -> Result<RecipeIngredient> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    state
        .db()
        .recipes()
        .update_ingredient(RecipeIngredient {
            recipe_id,
            ingredient_id,
            amount,
            unit_id,
            name: Some(ingredient_name),
        })
        .await
}

/// Updates one subrecipe row (weight). Returns the updated meta ingredient
/// for the caller's presentation layer.
/// Requires `EditRecipe` permission.
pub async fn update_subrecipe(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    subrecipe_id: RecipeId,
    subrecipe_name: String,
    weight: Amount,
) -> Result<RecipeMetaIngredient> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    state
        .db()
        .recipes()
        .update_meta_ingredient(RecipeMetaIngredient {
            parent_id: recipe_id,
            child_id: subrecipe_id,
            weight,
            name: Some(subrecipe_name),
        })
        .await
}

/// Updates one recipe step. Minutes are rounded to whole seconds and stored
/// as durations. Returns the updated step for the caller's presentation layer.
/// Requires `EditRecipe` permission.
#[allow(clippy::too_many_arguments)]
pub async fn update_step(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    order: f64,
    name: String,
    description: String,
    fixed_duration_minutes: f64,
    duration_per_kg_minutes: f64,
) -> Result<RecipeStep> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    let step = build_step(
        recipe_id,
        step_id,
        order,
        name,
        description,
        fixed_duration_minutes,
        duration_per_kg_minutes,
    );
    state.db().recipes().update_step(step).await
}

/// DnD reorder: persists only the client-computed new `order` (midpoint of the
/// neighbours) and leaves all other step fields untouched.
/// Requires `EditRecipe` permission.
pub async fn reorder_step(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    step_order: f64,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    let mut step = state
        .db()
        .recipes()
        .get_steps(recipe_id)
        .await?
        .into_iter()
        .find(|s| s.id == step_id)
        .ok_or_else(|| Error::NotFound {
            entity: "RecipeStep",
            id: step_id.to_string(),
        })?;
    step.order = step_order;
    state.db().recipes().update_step(step).await?;
    Ok(())
}

/// Removes one ingredient from a recipe.
/// Requires `EditRecipe` permission.
pub async fn delete_ingredient(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    ingredient_id: IngredientId,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    state
        .db()
        .recipes()
        .delete_ingredient(recipe_id, ingredient_id)
        .await
}

/// Removes one subrecipe link from a recipe.
/// Requires `EditRecipe` permission.
pub async fn delete_subrecipe(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    subrecipe_id: RecipeId,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    state
        .db()
        .recipes()
        .delete_meta_ingredient(recipe_id, subrecipe_id)
        .await
}

/// Removes one step from a recipe.
/// Requires `EditRecipe` permission.
pub async fn delete_step(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    state.db().recipes().delete_step(recipe_id, step_id).await
}

/// Adds one ingredient to a recipe. The ingredient is resolved by name (the
/// user picks from a datalist, so the row submits the name rather than the id).
/// Requires `EditRecipe` permission.
pub async fn add_ingredient(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    ingredient_name: &str,
    amount: Amount,
    unit_id: UnitId,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    let ingredient = state.db().ingredients().get_by_name(ingredient_name).await?;
    state
        .db()
        .recipes()
        .add_ingredient(RecipeIngredient {
            recipe_id,
            ingredient_id: ingredient.id,
            amount,
            unit_id,
            name: Some(ingredient_name.to_string()),
        })
        .await
}

/// Adds one subrecipe to a recipe. The subrecipe is resolved by name (the
/// user picks from a datalist, so the row submits the name rather than the id).
/// Requires `EditRecipe` permission.
pub async fn add_subrecipe(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    subrecipe_name: &str,
    weight: Amount,
) -> Result<()> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    let subrecipe = state.db().recipes().get_by_name(subrecipe_name).await?;
    state
        .db()
        .recipes()
        .add_meta_ingredient(RecipeMetaIngredient {
            parent_id: recipe_id,
            child_id: subrecipe.id,
            weight,
            name: Some(subrecipe_name.to_string()),
        })
        .await
}

/// Appends one step to a recipe. The order is server-computed as
/// `max(existing orders) + 1.0`; any client-supplied order is ignored.
/// Requires `EditRecipe` permission.
#[allow(clippy::too_many_arguments)]
pub async fn add_step(
    state: &AppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    name: String,
    description: String,
    fixed_duration_minutes: f64,
    duration_per_kg_minutes: f64,
) -> Result<RecipeStep> {
    actor.assert_can_edit_recipe(recipe_id).await?;
    let existing = state.db().recipes().get_steps(recipe_id).await?;
    let order = existing.iter().map(|s| s.order).fold(0.0_f64, f64::max) + 1.0;
    let step = build_step(
        recipe_id,
        step_id,
        order,
        name,
        description,
        fixed_duration_minutes,
        duration_per_kg_minutes,
    );
    state.db().recipes().add_step(step).await
}

/// Deletes a recipe permanently. Only administrators may delete recipes.
pub async fn delete_recipe(state: &AppState, actor: &AppCtx, recipe_id: RecipeId) -> Result<()> {
    actor.require_human()?;
    actor.require_admin()?;
    state.db().recipes().delete(recipe_id).await
}

/// Fetches resolved allergen/dietary properties for a recipe.
pub async fn get_recipe_properties(
    state: &AppState,
    recipe_id: RecipeId,
) -> Result<RecipeProperties> {
    state
        .db()
        .properties()
        .get_recipe_properties(recipe_id)
        .await
}
