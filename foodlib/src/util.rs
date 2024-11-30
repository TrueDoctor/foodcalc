use std::{borrow::Borrow, fmt::Display, str::FromStr};

use bigdecimal::BigDecimal;
use num::{FromPrimitive, ToPrimitive};
use serde::Deserialize;
use sqlx::postgres::types::PgInterval;

pub(crate) fn serialize_money<S>(money: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&money.to_string())
}

pub(crate) fn deserialize_money<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let money = FromStr::from_str(&s).unwrap();
    Ok(money)
}

pub(crate) fn serialize_optional_money<S>(
    money: &Option<BigDecimal>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match money {
        Some(money) => serializer.serialize_f64(money.to_f64().unwrap()),
        None => serializer.serialize_none(),
    }
}

pub(crate) fn deserialize_optional_money<'de, D>(
    deserializer: D,
) -> Result<Option<BigDecimal>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let money: f64 = f64::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(BigDecimal::from_f64(money))
    }
}

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
