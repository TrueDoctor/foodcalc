use crate::types::ids::UnitId;
use bigdecimal::BigDecimal;
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
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
}

// Mapping directly to the conversions materialized view
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct ConversionRow {
    pub from_unit: UnitId,
    pub to_unit: UnitId,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
}
