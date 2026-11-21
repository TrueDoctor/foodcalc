//! Recipe use-cases: filtered list/search, detail/tree composition, CRUD,
//! exports, deep clone, ingredient/step/sub-recipe editors and ownership
//! transfer.

use std::collections::HashMap;

use super::audit;
use super::{assert_ownership_transfer, AppCtx, Error, MyAppState, Result};
use bigdecimal::ToPrimitive;
use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::recipe::{RecipeIngredient, RecipeKind, RecipeListItem, RecipeMetaIngredient, RecipeStep};
use foodlib_new::entities::types::ids::{GroupId, IngredientId, PropertyId, RecipeId, RecipeStepId, UnitId};
use foodlib_new::policy::{Permission, RecipePerm};
use foodlib_new::recipe::Recipe;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::exports::ExportedFile;
// Export artifact shared with the events exports (filename + mime + bytes).
use foodlib_new::entities::types::amount::Amount;

#[derive(Debug, Deserialize)]
pub struct RecipeListQuery {
    /// Filter by kind: "meal" | "component" (absent = all).
    pub r#type: Option<String>,
    /// Case-insensitive substring search on the name.
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecipeListItemDto {
    pub id: RecipeId,
    pub name: String,
    pub comment: Option<String>,
    pub group_id: GroupId,
    pub group_name: String,
    pub kind: RecipeKind,
    pub weighed_in: bool,
    /// Energy density in kJ/g — the scaling basis for meal planning.
    /// `None` for recipes without any resolvable ingredients.
    pub energy_kj_per_g: Option<f64>,
}

pub async fn create_new_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
    comment: Option<String>,
    kind: RecipeKind,
    group_id: GroupId,
    weighed_in: Option<bool>,
) -> Result<RecipeId> {
    // Permission checks
    actor.ctx.assert_global(foodlib_new::policy::GlobalPermission::CreateResources)?;
    actor.ctx.assert_group_perm(group_id, Permission::Recipe(RecipePerm::Create))?;

    let recipe = Recipe {
        id: RecipeId(0),
        name,
        comment,
        group_id,
        weighed_in: weighed_in.unwrap_or(false),
        kind,
    };


    let created_recipe = state.recipes().create(recipe).await
        .map_err(|e| Error::Validation {
            message: format!("failed to create recipe: {e:?}"),
        })?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_CREATED,
        Some("recipe"),
        Some(created_recipe.id.into()),
        Some(group_id),
        None,
        None,
    )
    .await;
    Ok(created_recipe.id)
}

#[derive(Debug, Deserialize)]
pub struct RecipeUpdateInput {
    pub name: Option<String>,
    /// `null` clears the comment, absent keeps it.
    pub comment: Option<Option<String>>,
    pub kind: Option<RecipeKind>,
    pub weighed_in: Option<bool>,
}

pub async fn update_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    input: RecipeUpdateInput,
) -> Result<()> {
    let current = state.recipes().get(id).await?;
    actor.ctx.assert_group_perm(current.group_id, Permission::Recipe(RecipePerm::Edit))?;

    let updated = Recipe {
        id,
        name: input.name.unwrap_or(current.name),
        comment: input.comment.unwrap_or(current.comment),
        group_id: current.group_id,
        weighed_in: input.weighed_in.unwrap_or(current.weighed_in),
        kind: input.kind.unwrap_or(current.kind),
    };

    let recipe = state.recipes().update(updated).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_UPDATED,
        Some("recipe"),
        Some(id.into()),
        Some(current.group_id),
        None,
        Some(json!(recipe)),
    )
    .await;
    Ok(())
}

pub async fn delete_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
) -> Result<()> {
    let current = state.recipes().get(id).await?;
    actor.ctx.assert_group_perm(current.group_id, Permission::Recipe(RecipePerm::Delete))?;

    state.recipes().delete(id).await?;

    audit::record(
        state,
        actor,
        audit::action::ENTITY_DELETED,
        Some("recipe"),
        Some(id.into()),
        Some(current.group_id),
        None,
        None,
    )
    .await;
    Ok(())
}

/// Recipes visible to the caller (groups where they hold `RecipePerm::View`,
/// or all groups for admins), with group names resolved and optional kind/text filtering.
pub async fn list_recipes(
    state: &MyAppState,
    ctx: &AuthCtx,
    query: RecipeListQuery,
) -> Result<Vec<RecipeListItemDto>> {
    let recipes = state.recipes().list_with_stats().await?;
    let groups = if ctx.user.is_admin {
        state.users().list_groups().await?
    } else {
        state.users().get_user_groups(ctx.user.id).await?
    };

    let group_name_by_id: HashMap<GroupId, String> = groups
        .into_iter()
        .map(|group| (group.id, group.name))
        .collect();
    let search = query.search.unwrap_or_default().to_lowercase();
    let kind_filter: Option<RecipeKind> = match query.r#type.as_deref() {
        None | Some("") => None,
        Some("meal") => Some(RecipeKind::Meal),
        Some("component") => Some(RecipeKind::Component),
        Some(other) => {
            return Err(Error::Validation {
                message: format!("unknown recipe kind filter: {other}"),
            })
        }
    };

    Ok(recipes
        .into_iter()
        // Group visibility: members with `RecipePerm::View` see their groups' recipes; admins see all.
        .filter(|recipe| {
            ctx.has_group_permission(recipe.group_id, Permission::Recipe(RecipePerm::View))
        })
        .filter(|recipe| kind_filter.as_ref().map_or(true, |kind| &recipe.kind == kind))
        .filter(|recipe| search.is_empty() || recipe.name.to_lowercase().contains(&search))
        .map(|recipe| to_dto(recipe, &group_name_by_id))
        .collect())
}

fn to_dto(recipe: RecipeListItem, group_name_by_id: &HashMap<GroupId, String>) -> RecipeListItemDto {
    let weight_g = recipe.weight.to_f64().unwrap_or(0.0) * 1000.0;
    let energy_kj_per_g =
        (weight_g > 0.0).then(|| recipe.energy.to_f64().unwrap_or(0.0) / weight_g);
    RecipeListItemDto {
        id: recipe.id,
        name: recipe.name,
        comment: recipe.comment,
        group_id: recipe.group_id,
        group_name: group_name_by_id
            .get(&recipe.group_id)
            .cloned()
            .unwrap_or_default(),
        kind: recipe.kind,
        weighed_in: recipe.weighed_in,
        energy_kj_per_g,
    }
}


// ----------------------------------------------------------------- detail

#[derive(Debug, Serialize)]
pub struct RecipeStatsDto {
    /// Total resolved ingredient weight in kg.
    pub weight_kg: Amount,
    /// Total resolved energy in kJ.
    pub energy_kj: Amount,
    /// Energy density — the basis for scaling portions.
    pub energy_kj_per_g: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct RecipeIngredientDto {
    pub ingredient_id: IngredientId,
    pub name: Option<String>,
    pub amount: Amount,
    pub unit_id: UnitId,
}

#[derive(Debug, Serialize)]
pub struct SubRecipeDto {
    pub recipe_id: RecipeId,
    pub name: String,
    pub weight: Amount,
}

#[derive(Debug, Serialize)]
pub struct RecipeStepDto {
    pub id: RecipeStepId,
    pub order: f64,
    pub name: String,
    pub description: String,
    pub fixed_duration_seconds: i64,
    pub duration_per_kg_seconds: i64,
    /// Marks this step as a HACCP control point.
    pub is_haccp: bool,
}

#[derive(Debug, Serialize)]
pub struct PropertyDto {
    pub id: PropertyId,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RecipeDetailDto {
    pub id: RecipeId,
    pub name: String,
    pub comment: Option<String>,
    pub group_id: GroupId,
    pub kind: RecipeKind,
    pub weighed_in: bool,
    pub stats: Option<RecipeStatsDto>,
    pub ingredients: Vec<RecipeIngredientDto>,
    pub sub_recipes: Vec<SubRecipeDto>,
    pub steps: Vec<RecipeStepDto>,
    pub properties: Vec<PropertyDto>,
}

/// Energy density in kJ/g from a stats row (kg / kJ); `None` for zero weight.
pub fn energy_kj_per_g(weight_kg: &Amount, energy_kj: &Amount) -> Option<f64> {
    let weight_g = weight_kg.to_f64()? * 1000.0;
    (weight_g > 0.0).then(|| energy_kj.to_f64().unwrap_or(0.0) / weight_g)
}

pub async fn get_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    recipe_id: RecipeId
) -> Result<RecipeDetailDto> {
    let recipe = state.recipes().get(recipe_id).await?;
    // Group-private reads: non-members get a plain 404.
    if !actor.ctx.can_access_group(recipe.group_id) {
        return Err(Error::NotFound {
            entity: "Recipe",
            id: recipe_id.to_string(),
        });
    }

    let stats = state.recipes().get_recipe_stats(recipe_id).await.ok();
    let ingredients = state.recipes().get_ingredients(recipe_id).await?;
    let sub_recipes = state.recipes().get_meta_ingredients(recipe_id).await?;
    let steps = state.recipes().get_steps(recipe_id).await?;
    let properties = state
        .properties()
        .get_recipe_properties(recipe_id)
        .await
        .map(|summary| summary.properties)
        .unwrap_or_default();

    Ok(RecipeDetailDto {
        stats: stats.map(|stats| RecipeStatsDto {
            energy_kj_per_g: energy_kj_per_g(&stats.weight, &stats.energy),
            weight_kg: stats.weight,
            energy_kj: stats.energy,
        }),
        ingredients: ingredients
            .into_iter()
            .map(|ingredient| RecipeIngredientDto {
                ingredient_id: ingredient.ingredient_id,
                name: ingredient.name,
                amount: ingredient.amount,
                unit_id: ingredient.unit_id,
            })
            .collect(),
        sub_recipes: sub_recipes
            .into_iter()
            .map(|meta| SubRecipeDto {
                recipe_id: meta.child_id,
                name: meta.name.unwrap_or_default(),
                weight: meta.weight,
            })
            .collect(),
        steps: steps
            .into_iter()
            .map(|step| RecipeStepDto {
                id: step.id,
                order: step.order,
                name: step.name,
                description: step.description,
                fixed_duration_seconds: step.fixed_duration.to_f64().unwrap() as i64,
                duration_per_kg_seconds: step.duration_per_kg.to_f64().unwrap() as i64,
                is_haccp: step.is_haccp,
            })
            .collect(),
        properties: properties
            .into_iter()
            .map(|property| PropertyDto {
                id: property.id,
                name: property.name,
            })
            .collect(),
        id: recipe.id,
        name: recipe.name,
        comment: recipe.comment,
        group_id: recipe.group_id,
        kind: recipe.kind,
        weighed_in: recipe.weighed_in,
    })
}

/// explicit Besitz-Übertragung (siehe
/// `events::transfer_owner`): Besitzer oder Manager der aktuellen Gruppe;
/// Zielgruppe muss zugänglich sein.
pub async fn transfer_owner(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    target_group_id: GroupId,
) -> Result<()> {
    let ctx = &actor.ctx;
    let current = state.recipes().get(id).await?;
    assert_ownership_transfer(
        ctx,
        "recipe",
        current.group_id,
        target_group_id,
        Permission::Recipe(RecipePerm::TransferOwner),
    )?;
    state.recipes().set_group(id, target_group_id).await?;
    audit::record(
        state,
        actor,
        audit::action::OWNER_TRANSFERRED,
        Some("recipe"),
        Some(id.into()),
        Some(target_group_id),
        None,
        Some(json!({ "from_group_id": current.group_id })),
    )
    .await;
    Ok(())
}

// ------------------------------------------------------------------- tree

/// the recursion tree with resolved ingredient
/// weights per node (view mode visualization).
pub async fn tree(state: &MyAppState, actor: &AppCtx, recipe_id: RecipeId) -> Result<serde_json::Value> {
    let recipe = state.recipes().get(recipe_id).await?;
    // Group-private reads: non-members get a plain 404.
    if !actor.ctx.can_access_group(recipe.group_id) {
        return Err(Error::NotFound {
            entity: "Recipe",
            id: recipe_id.to_string(),
        });
    }
    let tree = state.recipes().get_recipe_tree(recipe_id).await?;
    Ok(serde_json::to_value(tree).map_err(|err| Error::Misc(err.to_string()))?)
}

// ------------------------------------------------------------------ export

#[derive(Debug, Deserialize)]
pub struct RecipeExportQuery {
    /// Target portions for the PDF (default: the recipe as-is).
    pub servings: Option<u32>,
    /// Target energy per portion in kJ (default: the recipe's own density).
    pub energy_per_serving: Option<u32>,
}

pub async fn export_pdf(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    query: RecipeExportQuery,
) -> Result<ExportedFile> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Export).await?;
    let recipe = state.recipes().get(id).await?;
    if !actor.ctx.can_access_group(recipe.group_id) {
        return Err(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        });
    }

    // Defaults: render the base recipe (1 portion at its own total energy).
    let stats = state.recipes().get_recipe_stats(id).await.ok();
    let servings = query.servings.unwrap_or(1) as f64;
    let energy_per_serving = match query.energy_per_serving {
        Some(energy) => energy,
        None => {
            let total = stats
                .as_ref()
                .map(|stats| stats.energy.to_f64().unwrap_or(0.0))
                .unwrap_or(0.0);
            (total / servings.max(1.0)).round() as u32
        }
    };

    let recipe_info = state
        .export()
        .fetch_user_input_meal(id, servings, energy_per_serving, String::new())
        .await?;
    let filename = crate::app::exports::sanitize_filename(&recipe_info.name);
    crate::app::exports::render_pdf(state, filename, move |pdf| {
        pdf.export_recipes(vec![recipe_info])
    })
        .await
}

// ------------------------------------------------------------------- clone

#[derive(Debug, Deserialize)]
pub struct RecipeCloneInput {
    /// Optional new name; defaults to `"<name> (Kopie)"`.
    pub name: Option<String>,
    /// Optional target group; defaults to the source recipe's group.
    pub group_id: Option<i32>,
}

/// deep clone into an (optional) target
/// group. Read access to the source is enough — the copy is owned by the
/// target group, which requires write access.
pub async fn clone_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    input: RecipeCloneInput,
) -> Result<RecipeId> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::View).await?;
    let source = state.recipes().get(id).await?;
    if !actor.ctx.can_access_group(source.group_id) {
        return Err(Error::NotFound {
            entity: "Recipe",
            id: id.to_string(),
        });
    }

    let target_group = input.group_id
        .map(GroupId)
        .unwrap_or(source.group_id);

    actor
        .ctx
        .assert_group_perm(target_group, Permission::Recipe(RecipePerm::Edit))?;
    let new_id = state
        .recipes()
        .duplicate(id, input.name, target_group)
        .await?;
    Ok(new_id)
}

// ------------------------------------------------------------- ingredients

#[derive(Debug, Deserialize)]
pub struct RecipeIngredientInput {
    pub ingredient_id: IngredientId,
    pub amount: String,
    pub unit_id: UnitId,
}

fn parse_amount(value: &str) -> Result<Amount> {
    use std::str::FromStr;
    Amount::from_str(value)
        .map_err(|_| Error::Validation {
            message: format!("invalid decimal amount: {value}"),
        })
}

/// add an ingredient line.
pub async fn add_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    input: RecipeIngredientInput,
) -> Result<RecipeIngredientDto> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    let amount = parse_amount(&input.amount)?;
    let line = RecipeIngredient {
        recipe_id: id,
        ingredient_id: input.ingredient_id,
        amount: amount.clone(),
        unit_id: input.unit_id,
        name: None,
    };
    state.recipes().add_ingredient(line).await?;
    Ok(RecipeIngredientDto {
        ingredient_id: input.ingredient_id,
        name: None,
        amount,
        unit_id: input.unit_id,
    })
}

/// replace an ingredient line.
pub async fn update_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    input: RecipeIngredientInput,
) -> Result<RecipeIngredientDto> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    let amount = parse_amount(&input.amount)?;
    let line = RecipeIngredient {
        recipe_id: id,
        ingredient_id: input.ingredient_id,
        amount: amount.clone(),
        unit_id: input.unit_id,
        name: None,
    };
    let updated = state.recipes().update_ingredient(line).await?;
    Ok(RecipeIngredientDto {
        ingredient_id: updated.ingredient_id,
        name: updated.name,
        amount,
        unit_id: updated.unit_id,
    })
}

/// remove the line.
pub async fn remove_ingredient(state: &MyAppState, actor: &AppCtx, id: RecipeId, ingredient_id: IngredientId) -> Result<()> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    state.recipes().delete_ingredient(id, ingredient_id).await?;
    Ok(())
}

// -------------------------------------------------------------------- steps

#[derive(Debug, Deserialize)]
pub struct RecipeStepInput {
    pub order: f64,
    pub name: String,
    pub description: String,
    pub fixed_duration_seconds: i64,
    pub duration_per_kg_seconds: i64,
    /// Marks this step as a HACCP control point.
    pub is_haccp: bool,
}

fn step_input_to_entity(recipe_id: RecipeId, id: RecipeStepId, payload: &RecipeStepInput) -> RecipeStep {
    RecipeStep {
        id,
        recipe_id,
        order: payload.order,
        name: payload.name.clone(),
        description: payload.description.clone(),
        fixed_duration: payload.fixed_duration_seconds.to_f64().unwrap(),
        duration_per_kg: payload.duration_per_kg_seconds.to_f64().unwrap(),
        is_haccp: payload.is_haccp,
    }
}

fn step_to_dto(step: RecipeStep) -> RecipeStepDto {
    RecipeStepDto {
        id: step.id,
        order: step.order,
        name: step.name,
        description: step.description,
        fixed_duration_seconds: step.fixed_duration.to_i64().unwrap(),
        duration_per_kg_seconds: step.duration_per_kg.to_i64().unwrap(),
        is_haccp: step.is_haccp,
    }
}

/// add a step.
pub async fn add_step(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    input: RecipeStepInput,
) -> Result<RecipeStepDto> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    let step = step_input_to_entity(id, RecipeStepId(0), &input); //TODO: RecipeStepId should be generated by the database, not hardcoded to 0
    let record = state.recipes().add_step(step).await?;
    Ok(step_to_dto(record))
}

/// replace a step.
pub async fn update_step(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    step_id: RecipeStepId,
    input: RecipeStepInput,
) -> Result<RecipeStepDto> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    let step = step_input_to_entity(id, step_id, &input);
    let record = state.recipes().update_step(step).await?;
    Ok(step_to_dto(record))
}

/// remove a step.
pub async fn remove_step(state: &MyAppState, actor: &AppCtx, id: RecipeId, step_id: RecipeStepId) -> Result<()> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    state.recipes().delete_step(id, step_id).await?;
    Ok(())
}

// -------------------------------------------------------------- sub-recipes

#[derive(Debug, Deserialize)]
pub struct SubRecipeInput {
    pub child_recipe_id: RecipeId,
    /// Weight of the child recipe used by the parent, in kg.
    pub weight: String,
}

#[derive(Debug, Deserialize)]
pub struct SubRecipeWeightInput {
    pub weight: String,
}

pub async fn add_sub_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    input: SubRecipeInput,
) -> Result<SubRecipeDto> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    let weight = parse_amount(&input.weight)?;
    let meta = RecipeMetaIngredient {
        parent_id: id,
        child_id: input.child_recipe_id,
        weight: weight.clone(),
        name: None,
    };
    state.recipes().add_meta_ingredient(meta).await?;
    let name = state
        .recipes()
        .get(input.child_recipe_id)
        .await
        .map(|recipe| recipe.name)
        .unwrap_or_default();
    Ok(SubRecipeDto {
        recipe_id: input.child_recipe_id,
        name,
        weight,
    })
}

pub async fn update_sub_recipe(
    state: &MyAppState,
    actor: &AppCtx,
    id: RecipeId,
    child_id: RecipeId,
    input: SubRecipeWeightInput,
) -> Result<SubRecipeDto> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    let weight = parse_amount(&input.weight)?;
    let meta = RecipeMetaIngredient {
        parent_id: id,
        child_id,
        weight: weight.clone(),
        name: None,
    };
    state.recipes().update_meta_ingredient(meta).await?;
    let name = state
        .recipes()
        .get(child_id)
        .await
        .map(|recipe| recipe.name)
        .unwrap_or_default();
    Ok(SubRecipeDto {
        recipe_id: child_id,
        name,
        weight,
    })
}

pub async fn remove_sub_recipe(state: &MyAppState, actor: &AppCtx, id: RecipeId, child_id: RecipeId) -> Result<()> {
    actor.ctx.assert_recipe_perm(id, RecipePerm::Edit).await?;
    state.recipes().delete_meta_ingredient(id, child_id).await?;
    Ok(())
}
