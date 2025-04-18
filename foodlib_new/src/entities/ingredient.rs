use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ingredient {
    pub id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
    pub owner_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientWithSource {
    pub id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
    pub owner_id: i64,
    pub has_sources: bool,
}

impl From<Ingredient> for IngredientWithSource {
    fn from(value: Ingredient) -> Self {
        let Ingredient {
            id,
            name,
            energy,
            comment,
            owner_id,
        } = value;
        Self {
            id,
            name,
            energy,
            comment,
            owner_id,
            has_sources: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientSource {
    pub id: i32,
    pub ingredient_id: i32,
    pub store_id: i32,
    pub package_size: BigDecimal,
    pub unit_id: i32,
    pub price: BigDecimal,
    pub url: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientWeight {
    pub ingredient_id: i32,
    pub unit_id: i32,
    pub weight: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientProperty {
    pub ingredient_id: i32,
    pub property_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientCategory {
    pub ingredient_source_id: i32,
    pub category: String,
}
