use crate::entities::types::ids::{GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId, UnitId};

use crate::entities::types::amount::Amount;
use crate::entities::types::money::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ingredient {
    pub id: IngredientId,
    pub name: String,
    pub energy: Amount,
    pub comment: Option<String>,
    pub group_id: GroupId,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateIngredient {
    pub name: String,
    pub energy: Amount,
    pub comment: Option<String>,
    pub group_id: GroupId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientWithSource {
    pub id: IngredientId,
    pub name: String,
    pub energy: Amount,
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
    pub package_size: Amount,
    pub unit_id: UnitId,
    pub price: Money,
    pub url: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientWeight {
    pub ingredient_id: IngredientId,
    pub unit_id: UnitId,
    pub weight: Amount,
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
