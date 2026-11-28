use bigdecimal::BigDecimal;
use crate::types::ids::{GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId, UnitId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ingredient {
    pub id: IngredientId,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
    pub group_id: GroupId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientWithSource {
    pub id: IngredientId,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
    pub group_id: GroupId,
    pub has_sources: bool,
}

impl From<Ingredient> for IngredientWithSource {
    fn from(value: Ingredient) -> Self {
        let Ingredient {
            id,
            name,
            energy,
            comment,
            group_id,
        } = value;
        Self {
            id,
            name,
            energy,
            comment,
            group_id,
            has_sources: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientSource {
    pub id: IngredientSourceId,
    pub ingredient_id: IngredientId,
    pub store_id: StoreId,
    pub package_size: BigDecimal,
    pub unit_id: UnitId,
    pub price: BigDecimal,
    pub url: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientWeight {
    pub ingredient_id: IngredientId,
    pub unit_id: UnitId,
    pub weight: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientProperty {
    pub ingredient_id: IngredientId,
    pub property_id: PropertyId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientCategory {
    pub ingredient_source_id: IngredientSourceId,
    pub category: String,
}
