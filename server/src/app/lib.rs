use crate::MyAppState;

use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::PlaceId;
use foodlib_new::Error;
use std::str::FromStr;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub fn parse_time(value: &str) -> foodlib_new::Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| Error::Validation {
        message: format!("invalid RFC 3339 timestamp: {value}"),
    })
}

pub fn parse_decimal(value: &str, field: &'static str) -> foodlib_new::Result<Amount> {
    Amount::from_str(value).map_err(|_| Error::Validation {
        message: format!("invalid {field}: {value}"),
    })
}



/// Falls back to the first configured place when the client omits `place_id`.
pub async fn get_default_place_for_event(state: &MyAppState) -> foodlib_new::Result<PlaceId> {
    let places = state.events().get_places().await?;
    places.first().map(|place| place.id).ok_or(Error::Validation {
        message: "no places configured".into(),
    })
}

/// Trims, drops empties, dedups and sorts role slugs.
pub fn normalize_roles(roles: Vec<String>) -> Vec<String> {
    let mut roles: Vec<String> = roles
        .into_iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    roles.sort();
    roles.dedup();
    roles
}