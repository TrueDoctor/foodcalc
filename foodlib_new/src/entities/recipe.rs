use crate::types::ids::{GroupId, IngredientId, RecipeId, RecipeStepId, UnitId};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,
    pub comment: Option<String>,
    pub group_id: GroupId,
    /// True when every ingredient amount has been weighed and verified accurate
    /// to the gram ("eingewogen"). Shown as a warning in the edit tab while
    /// false and as a badge on the printed recipe sheet once true.
    pub weighed_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub recipe_id: RecipeId,
    pub ingredient_id: IngredientId,
    pub amount: BigDecimal,
    pub unit_id: UnitId,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RecipeStep {
    pub id: RecipeStepId,
    pub recipe_id: RecipeId,
    pub order: f64, // TODO: use a proper ordering type instead of f64
    pub name: String,
    pub description: String,
    pub fixed_duration: PgInterval,  // duration in seconds
    pub duration_per_kg: PgInterval, // duration in seconds per kg
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetaIngredient {
    pub parent_id: RecipeId,
    pub child_id: RecipeId,
    pub weight: BigDecimal,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecipeStats {
    pub recipe_id: RecipeId,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubRecipe {
    pub subrecipe_id: RecipeId,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: BigDecimal,
    pub is_subrecipe: bool,
    pub recipe_id: RecipeId,
    pub subrecipe_hierarchy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRecipe {
    pub recipe_id: RecipeId,
    pub recipe_name: String,
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    pub weight: BigDecimal,
    pub subrecipe_id: Option<RecipeId>,
    pub subrecipe_name: Option<String>,
    pub hierarchy_path: Option<String>,
}
