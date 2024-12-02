use std::{borrow::Borrow, fmt::Display};

use bigdecimal::BigDecimal;
use serde::Deserialize;
use sqlx::postgres::types::PgInterval;

pub(crate) fn serialize_interval<S>(interval: &PgInterval, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration = interval.microseconds;
    serializer.serialize_str(&duration.to_string())
}

pub(crate) fn deserialize_interval<'de, D>(deserializer: D) -> Result<PgInterval, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let microseconds = s
        .parse()
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse interval: {}", e)))?;
    let interval = PgInterval {
        microseconds,
        days: 0,
        months: 0,
    };
    Ok(interval)
}

pub(crate) fn display_optional<T: Display>(value: &Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}

pub(crate) fn display_optional_money(value: &Option<BigDecimal>) -> String {
    match value {
        Some(value) => crate::util::format_pg_money(value),
        None => "".to_string(),
    }
}

pub(crate) fn format_pg_money(money: impl Borrow<BigDecimal>) -> String {
    format!("{} €", money.borrow())
}
