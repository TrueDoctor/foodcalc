use std::str::FromStr;

use serde::Deserialize;
use sqlx::postgres::types::{PgInterval, PgMoney};

pub(crate) fn serialize_money<S>(money: &PgMoney, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&money.0.to_string())
}

pub(crate) fn deserialize_money<'de, D>(deserializer: D) -> Result<PgMoney, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let money = PgMoney::from_bigdecimal(FromStr::from_str(&s).unwrap(), 2);
    Ok(money.unwrap())
}

pub(crate) fn serialize_optional_money<S>(
    money: &Option<PgMoney>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match money {
        Some(money) => serializer.serialize_str(&money.0.to_string()),
        None => serializer.serialize_none(),
    }
}

pub(crate) fn deserialize_optional_money<'de, D>(
    deserializer: D,
) -> Result<Option<PgMoney>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        let money: i64 = i64::from_str(&s).map_err(serde::de::Error::custom)?;
        let money = PgMoney(money);
        Ok(Some(money))
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
