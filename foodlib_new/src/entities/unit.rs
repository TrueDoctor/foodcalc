use crate::entities::types::ids::UnitId;

use crate::entities::types::amount::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitConversion {
    pub from_unit: UnitId,
    pub to_unit: UnitId,
    pub from_amount: Amount,
    pub to_amount: Amount,
}

// Mapping directly to the conversions materialized view
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct ConversionRow {
    pub from_unit: UnitId,
    pub to_unit: UnitId,
    pub from_amount: Amount,
    pub to_amount: Amount,
}
