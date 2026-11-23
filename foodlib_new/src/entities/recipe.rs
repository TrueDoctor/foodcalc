use crate::entities::types::ids::{GroupId, IngredientId, RecipeId, RecipeStepId, UnitId};

use crate::entities::types::amount::Amount;
use serde::{Deserialize, Serialize};
use time::Duration;

/// Kind of the recipe: "meal" (Gericht) or "component" (Komponente).
/// Purely organizational — meals and components can reference each
/// other freely as sub-recipes.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "recipe_kind", rename_all = "lowercase")]
pub enum RecipeKind {
    Meal,
    Component,
}

impl RecipeKind {
    /// The database / JSON representation ("meal" | "component").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Meal => "meal",
            Self::Component => "component",
        }
    }
}


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
    pub kind: RecipeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub recipe_id: RecipeId,
    pub ingredient_id: IngredientId,
    pub amount: Amount,
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
    pub fixed_duration: Duration,  // duration in seconds
    pub duration_per_kg: Duration, // duration in seconds per kg
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetaIngredient {
    pub parent_id: RecipeId,
    pub child_id: RecipeId,
    pub weight: Amount,
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecipeStats {
    pub recipe_id: RecipeId,
    pub weight: Amount,
    pub energy: Amount,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubRecipe {
    pub subrecipe_id: RecipeId,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: Amount,
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
    pub weight: Amount,
    pub subrecipe_id: Option<RecipeId>,
    pub subrecipe_name: Option<String>,
    pub hierarchy_path: Option<String>,
}

/// A recipe together with its aggregated stats from the `recipe_stats`
/// view. Used by list endpoints that show the energy density (kJ/g).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeListItem {
    pub id: RecipeId,
    pub name: String,
    pub comment: Option<String>,
    pub group_id: GroupId,
    pub kind: RecipeKind,
    pub weighed_in: bool,
    /// Total resolved ingredient weight in kg (0 for empty recipes).
    pub weight: Amount,
    /// Total resolved energy in kJ.
    pub energy: Amount,
}

/// A node of the recipe recursion tree: a recipe together with its direct
/// ingredients (resolved weights) and its used sub-recipes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeTreeNode {
    pub recipe_id: RecipeId,
    pub name: String,
    pub kind: RecipeKind,
    /// kg of this sub-recipe used by its parent (`None` for the root).
    pub weight: Option<Amount>,
    /// Direct ingredients of this recipe, resolved to kg where possible.
    pub ingredients: Vec<RecipeTreeIngredient>,
    /// Sub-recipes used by this recipe.
    pub children: Vec<RecipeTreeNode>,
}

/// One leaf of the tree: an ingredient row with its raw amount and the
/// unit-converted weight the event views use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeTreeIngredient {
    pub ingredient_id: IngredientId,
    pub name: Option<String>,
    /// Raw amount as entered on the recipe.
    pub amount: Amount,
    pub unit_id: UnitId,
    /// Unit name for display ("kg", "Stück", "Liter", …).
    pub unit_name: Option<String>,
    /// Unit-converted weight (kg) from the `resolved_recipes` view;
    /// `None` when no conversion exists.
    pub weight: Option<Amount>,
}
