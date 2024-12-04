use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IngredientProperties {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub properties: Vec<Property>,
}

// For summarizing dietary requirements/restrictions across recipes
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecipeProperties {
    pub recipe_id: i32,
    pub recipe_name: String,
    pub properties: Vec<Property>,
}
