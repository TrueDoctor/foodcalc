use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub recipe_id: i32,
    pub ingredient_id: i32,
    pub amount: BigDecimal,
    pub unit_id: i32,
}

#[derive(Debug, Clone)]
pub struct RecipeStep {
    pub id: i32,
    pub recipe_id: i32,
    pub order: f64,
    pub name: String,
    pub description: String,
    pub fixed_duration: PgInterval,  // duration in seconds
    pub duration_per_kg: PgInterval, // duration in seconds per kg
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetaIngredient {
    pub parent_id: i32,
    pub child_id: i32,
    pub weight: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStats {
    pub recipe_id: i32,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRecipe {
    pub recipe_id: i32,
    pub recipe_name: String,
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub weight: BigDecimal,
    pub subrecipe_id: Option<i32>,
    pub subrecipe_name: Option<String>,
    pub hierarchy_path: Option<String>,
}
