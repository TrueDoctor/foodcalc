//! Recipe use-cases.

use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{GroupId, IngredientId, RecipeId, RecipeStepId, UnitId};
use foodlib_new::ingredient::Ingredient;
use foodlib_new::recipe::{
    Recipe, RecipeData, RecipeIngredient, RecipeKind, RecipeMetaIngredient, RecipeStats,
    RecipeStep,
};
use foodlib_new::unit::Unit;
use foodlib_new::user::User;
use time::Duration;

use crate::app_ctx::{assert_can_move_to, AppCtx};
use crate::{Error, MyAppState, Result};

/// Everything the recipe overview/list renders: all recipes plus the group
/// ids of the acting user (empty for anonymous callers), fetched in one use
/// case so transports only have to deal with presentation.
pub async fn gather_overview(
    state: &MyAppState,
    user: Option<&User>,
) -> Result<(Vec<Recipe>, Vec<GroupId>)> {
    let recipes = state.recipes().list().await?;
    let user_group_ids = match user {
        Some(u) => crate::users::user_group_ids(state, u.id).await?,
        None => vec![],
    };
    Ok((recipes, user_group_ids))
}

/// All measurement units (used to render unit pickers).
pub async fn list_units(state: &MyAppState) -> Result<Vec<Unit>> {
    state.units().list().await
}

/// Lists all recipes.
pub async fn list_recipes(state: &MyAppState) -> Result<Vec<Recipe>> {
    state.recipes().list().await
}

/// Fetches one recipe.
pub async fn get_recipe(state: &MyAppState, recipe_id: RecipeId) -> Result<Recipe> {
    state.recipes().get(recipe_id).await
}

/// Everything the recipe allergen dialog renders. The raw property labels
/// are classified here (regulatory "contains" vs "~spuren-" may-contain vs
/// informational "~hinweis-" notes) so transports only render.
pub struct RecipeAllergenData {
    pub recipe_name: String,
    pub dietary: foodlib_new::allergen::DietaryFlags,
    pub contains: Vec<String>,
    pub may_contain: Vec<String>,
    pub hinweise: Vec<String>,
}

pub async fn gather_recipe_allergens(
    state: &MyAppState,
    recipe_id: RecipeId,
) -> Result<RecipeAllergenData> {
    let rp = state.properties().get_recipe_properties(recipe_id).await?;
    let dietary = foodlib_new::ops::allergens::dietary_flags(&rp.properties);

    let mut contains_lower: Vec<String> = Vec::new();
    let mut may_lower: Vec<String> = Vec::new();
    let mut hinweis_lower: Vec<String> = Vec::new();
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
    contains_lower.sort();
    contains_lower.dedup();
    may_lower.sort();
    may_lower.dedup();
    hinweis_lower.sort();
    hinweis_lower.dedup();

    Ok(RecipeAllergenData {
        recipe_name: rp.recipe_name,
        dietary,
        contains: foodlib_new::ops::allergens::compact_allergen_set(&contains_lower),
        may_contain: foodlib_new::ops::allergens::compact_allergen_set(&may_lower),
        hinweise: hinweis_lower,
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
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
    comment: Option<String>,
) -> Result<RecipeId> {
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    let recipe = state
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
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
) -> Result<EditViewData> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let subrecipes = state.recipes().get_meta_ingredients(recipe_id).await?;

    let ingredients = state.recipes().get_ingredients(recipe_id).await?;
    let all_ingredients = state.ingredients().list().await?;
    let all_recipes = state.recipes().list().await?;
    let unit_types = state.units().list().await?;
    let steps = state.recipes().get_steps(recipe_id).await?;
    // The stats might not exist yet if we are creating a new recipe
    let stats = state
        .recipes()
        .get_recipe_stats(recipe_id)
        .await
        .unwrap_or_default();
    let recipe = state.recipes().get(recipe_id).await?;

    Ok(EditViewData {
        recipe,
        sub_recipes: subrecipes,
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
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    name: String,
    comment: String,
    group_id: GroupId,
    weighed_in: bool,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;

    let current = state.recipes().get(recipe_id).await?;
    if current.group_id != group_id {
        assert_can_move_to(actor, group_id)?;
    }

    state
        .recipes()
        .update(
            recipe_id,
            RecipeData {
                name,
                // An empty comment means "no comment", matching how
                // events::update_event normalizes its comment field.
                comment: if comment.is_empty() { None } else { Some(comment) },
                weighed_in,
                kind: current.kind,
            },
        )
        .await?;
    if current.group_id != group_id {
        state.recipes().set_group(recipe_id, group_id).await?;
    }
    Ok(())
}

/// Updates one recipe ingredient row (amount and unit). Returns the updated
/// ingredient for the caller's presentation layer.
/// Requires `EditRecipe` permission.
pub async fn update_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    ingredient_id: IngredientId,
    ingredient_name: String,
    amount: Amount,
    unit_id: UnitId,
) -> Result<RecipeIngredient> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let recipe_ingredient = state
        .recipes()
        .update_ingredient(RecipeIngredient {
            recipe_id,
            ingredient_id,
            amount,
            unit_id,
            name: Some(ingredient_name),
        })
        .await?;
    Ok(recipe_ingredient)
}

/// Updates one subrecipe row (weight). Returns the updated meta ingredient
/// for the caller's presentation layer.
/// Requires `EditRecipe` permission.
pub async fn update_subrecipe(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    subrecipe_id: RecipeId,
    subrecipe_name: String,
    weight: Amount,
) -> Result<RecipeMetaIngredient> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let sub_recipe = state
        .recipes()
        .update_meta_ingredient(RecipeMetaIngredient {
            parent_id: recipe_id,
            child_id: subrecipe_id,
            weight,
            name: Some(subrecipe_name),
        })
        .await?;
    Ok(sub_recipe)
}

/// Updates one recipe step. Minutes are rounded to whole seconds and stored
/// as durations. Returns the updated step for the caller's presentation layer.
/// Requires `EditRecipe` permission.
pub async fn update_step(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    order: f64,
    name: String,
    description: String,
    fixed_duration_minutes: f64,
    duration_per_kg_minutes: f64,
) -> Result<RecipeStep> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let step = state
        .recipes()
        .update_step(RecipeStep {
            id: step_id,
            order,
            name,
            description,
            fixed_duration: Duration::seconds((fixed_duration_minutes * 60.0).round() as i64),
            duration_per_kg: Duration::seconds(
                (duration_per_kg_minutes * 60.0).round() as i64
            ),
            recipe_id,
        })
        .await?;
    Ok(step)
}

/// DnD reorder: persists only the client-computed new `order` (midpoint of the
/// neighbours) and leaves all other step fields untouched.
/// Requires `EditRecipe` permission.
pub async fn reorder_step(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    step_order: f64,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let mut step = state
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
    state.recipes().update_step(step).await?;
    Ok(())
}

/// Removes one ingredient from a recipe.
/// Requires `EditRecipe` permission.
pub async fn delete_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    ingredient_id: IngredientId,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    state
        .recipes()
        .delete_ingredient(recipe_id, ingredient_id)
        .await?;
    Ok(())
}

/// Removes one subrecipe link from a recipe.
/// Requires `EditRecipe` permission.
pub async fn delete_subrecipe(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    subrecipe_id: RecipeId,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    state
        .recipes()
        .delete_meta_ingredient(recipe_id, subrecipe_id)
        .await?;
    Ok(())
}

/// Removes one step from a recipe.
/// Requires `EditRecipe` permission.
pub async fn delete_step(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    state.recipes().delete_step(recipe_id, step_id).await?;
    Ok(())
}

/// Adds one ingredient to a recipe. The ingredient is resolved by name (the
/// user picks from a datalist, so the row submits the name rather than the id).
/// Requires `EditRecipe` permission.
pub async fn add_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    ingredient_name: &str,
    amount: Amount,
    unit_id: UnitId,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let ingredient = state.ingredients().get_by_name(ingredient_name).await?;
    state
        .recipes()
        .add_ingredient(RecipeIngredient {
            recipe_id,
            ingredient_id: ingredient.id,
            amount,
            unit_id,
            name: Some(ingredient_name.to_string()),
        })
        .await?;
    Ok(())
}

/// Adds one subrecipe to a recipe. The subrecipe is resolved by name (the
/// user picks from a datalist, so the row submits the name rather than the id).
/// Requires `EditRecipe` permission.
pub async fn add_subrecipe(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    subrecipe_name: &str,
    weight: Amount,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    let subrecipe = state.recipes().get_by_name(subrecipe_name).await?;
    state
        .recipes()
        .add_meta_ingredient(RecipeMetaIngredient {
            parent_id: recipe_id,
            child_id: subrecipe.id,
            weight,
            name: Some(subrecipe_name.to_string()),
        })
        .await?;
    Ok(())
}

/// Appends one step to a recipe. The order is server-computed as
/// `max(existing orders) + 1.0`; any client-supplied order is ignored.
/// Requires `EditRecipe` permission.
pub async fn add_step(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId,
    step_id: RecipeStepId,
    name: String,
    description: String,
    fixed_duration_minutes: f64,
    duration_per_kg_minutes: f64,
) -> Result<()> {
    actor.ctx.assert_can_edit_recipe(recipe_id).await?;
    // Order is no longer user-supplied — append after the current max.
    let existing = state.recipes().get_steps(recipe_id).await?;
    let order = existing.iter().map(|s| s.order).fold(0.0_f64, f64::max) + 1.0;
    state
        .recipes()
        .add_step(RecipeStep {
            id: step_id,
            order,
            name,
            description,
            fixed_duration: Duration::seconds((fixed_duration_minutes * 60.0).round() as i64),
            duration_per_kg: Duration::seconds(
                (duration_per_kg_minutes * 60.0).round() as i64
            ),
            recipe_id,
        })
    .await?;
    Ok(())
}

/// Deletes a recipe permanently. Only administrators may delete recipes.
pub async fn delete_recipe(
    state: &MyAppState,
    user: &User,
    recipe_id: RecipeId,
) -> Result<()> {
    if !user.is_admin {
        return Err(Error::Forbidden(
            "Only administrators can delete recipes".into(),
        ));
    }
    state.recipes().delete(recipe_id).await?;
    Ok(())
}
