use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitConversion {
    pub from_unit: i32,
    pub to_unit: i32,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
}

// Mapping directly to the conversions materialized view
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct ConversionRow {
    pub from_unit: i32,
    pub to_unit: i32,
    pub from_amount: BigDecimal,
    pub to_amount: BigDecimal,
}
