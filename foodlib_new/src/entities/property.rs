use crate::types::ids::{IngredientId, PropertyId, RecipeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub id: PropertyId,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IngredientProperties {
    pub ingredient_id: IngredientId,
    pub ingredient_name: String,
    pub properties: Vec<Property>,
}

// For summarizing dietary requirements/restrictions across recipes
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecipeProperties {
    pub recipe_id: RecipeId,
    pub recipe_name: String,
    pub properties: Vec<Property>,
}
