//! Engelsystem integration — read-only helper-shift data for the event timeline.
//!
//! We fetch shifts for a fixed location from the Engelsystem REST API
//! (`GET {ENGELSYSTEM_URL}/api/v0-beta/locations/{id}/shifts`, authenticated
//! with the `x-api-key` header) and expose them as [`EngelShift`]s the timeline
//! can render alongside meals, tours and food prep.
//!
//! Timezone: Engelsystem returns true UTC instants (ISO-8601 with `Z`). The
//! event runs in Europe/Berlin, and the rest of foodcalc stores wall-clock time
//! as UTC (see `event_meals` handling). To line shifts up with meals on the
//! timeline we convert each Engelsystem instant to Berlin wall-clock and hand
//! back an `OffsetDateTime` whose components ARE that local time — matching the
//! foodcalc convention. The CET/CEST rule is computed directly (EU DST: last
//! Sunday of March 01:00 UTC → last Sunday of October 01:00 UTC = +2h, else
//! +1h), so we need no timezone database.
//!
//! This is a best-effort read overlay: any misconfiguration or network/parse
//! failure yields an empty shift list rather than breaking the timeline. Errors
//! are logged, not propagated.

use serde::Deserialize;
use time::{Date, Month, OffsetDateTime, UtcOffset, Weekday};

/// API version path segment for the Engelsystem REST API.
const API_BASE: &str = "/api/v0-beta";

/// One helper shift, normalized for the timeline. `start`/`end` are Berlin
/// wall-clock (see module docs). `needed`/`filled` drive the "Helfer X/Y" badge.
#[derive(Debug, Clone)]
pub struct EngelShift {
    pub title: String,
    /// Shift type name (e.g. "Catering"), used as the primary label.
    pub shift_type: String,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    /// Total helpers needed across all angel types on this shift.
    pub needed: i64,
    /// Total helpers actually signed up across all angel types.
    pub filled: i64,
}

impl EngelShift {
    /// True when the shift is fully (or over-) staffed.
    pub fn is_full(&self) -> bool {
        self.filled >= self.needed
    }
}

// ── Wire types (subset of the OpenAPI `Shift` schema we actually use) ─────────

#[derive(Debug, Deserialize)]
struct ShiftsResponse {
    data: Vec<WireShift>,
}

#[derive(Debug, Deserialize)]
struct WireShift {
    name: String,
    starts_at: String,
    ends_at: String,
    shift_type: WireReference,
    #[serde(default)]
    needed_angel_types: Vec<WireNeededAngelType>,
}

#[derive(Debug, Deserialize)]
struct WireReference {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WireNeededAngelType {
    #[serde(default)]
    needs: i64,
    #[serde(default)]
    entries: Vec<serde_json::Value>,
}

/// Client configuration, read from the environment. `None` when the integration
/// is not configured (either env var missing) — callers then skip the overlay.
pub struct EngelSystem {
    base_url: String,
    api_key: String,
}

impl EngelSystem {
    /// Build a client from `ENGELSYSTEM_URL` and `ENGELSYSTEM_KEY`. Returns
    /// `None` (integration disabled) when either is unset or empty.
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("ENGELSYSTEM_URL").ok().filter(|s| !s.is_empty())?;
        let api_key = std::env::var("ENGELSYSTEM_KEY").ok().filter(|s| !s.is_empty())?;
        Some(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }

    /// Fetch and normalize all shifts for a location. Best-effort: on any
    /// network, HTTP-status or parse error this logs and returns an empty vec,
    /// so a flaky Engelsystem never breaks the timeline.
    pub async fn get_location_shifts(&self, location_id: i32) -> Vec<EngelShift> {
        match self.try_get_location_shifts(location_id).await {
            Ok(shifts) => shifts,
            Err(e) => {
                log::warn!("Engelsystem shift fetch for location {location_id} failed: {e}");
                Vec::new()
            }
        }
    }

    async fn try_get_location_shifts(
        &self,
        location_id: i32,
    ) -> Result<Vec<EngelShift>, reqwest::Error> {
        let url = format!("{}{}/locations/{}/shifts", self.base_url, API_BASE, location_id);
        let resp = reqwest::Client::new()
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json::<ShiftsResponse>()
            .await?;

        let shifts = resp
            .data
            .into_iter()
            .filter_map(|w| {
                let start = parse_berlin(&w.starts_at)?;
                let end = parse_berlin(&w.ends_at)?;
                let needed: i64 = w.needed_angel_types.iter().map(|n| n.needs).sum();
                let filled: i64 = w
                    .needed_angel_types
                    .iter()
                    .map(|n| n.entries.len() as i64)
                    .sum();
                Some(EngelShift {
                    title: w.name,
                    shift_type: w.shift_type.name.unwrap_or_default(),
                    start,
                    end,
                    needed,
                    filled,
                })
            })
            .collect();
        Ok(shifts)
    }
}

/// Parse an Engelsystem ISO-8601 UTC timestamp and return an `OffsetDateTime`
/// whose components are the equivalent Europe/Berlin wall-clock — matching the
/// foodcalc convention where stored components are read directly.
fn parse_berlin(s: &str) -> Option<OffsetDateTime> {
    let utc = OffsetDateTime::parse(
        s,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .ok()?;
    let offset = berlin_offset(utc);
    // Shift the instant into Berlin local time, then relabel it as UTC so its
    // .date()/.time() components read as the local wall-clock (foodcalc style).
    let local = utc.to_offset(offset);
    local.replace_offset(UtcOffset::UTC).into()
}

/// The Europe/Berlin UTC offset in effect at the given instant: +2h during
/// CEST, +1h during CET. EU summer time runs from the last Sunday of March at
/// 01:00 UTC to the last Sunday of October at 01:00 UTC.
fn berlin_offset(utc: OffsetDateTime) -> UtcOffset {
    let year = utc.year();
    let dst_start = last_sunday(year, Month::March).with_hms(1, 0, 0).unwrap().assume_utc();
    let dst_end = last_sunday(year, Month::October).with_hms(1, 0, 0).unwrap().assume_utc();
    if utc >= dst_start && utc < dst_end {
        UtcOffset::from_hms(2, 0, 0).unwrap()
    } else {
        UtcOffset::from_hms(1, 0, 0).unwrap()
    }
}

/// The date of the last Sunday in the given month/year.
fn last_sunday(year: i32, month: Month) -> Date {
    // Start at the last day of the month and walk back to a Sunday.
    let mut d = Date::from_calendar_date(year, month, days_in_month(year, month)).unwrap();
    while d.weekday() != Weekday::Sunday {
        d = d.previous_day().unwrap();
    }
    d
}

/// Number of days in the given month, accounting for leap years.
fn days_in_month(year: i32, month: Month) -> u8 {
    match month {
        Month::January
        | Month::March
        | Month::May
        | Month::July
        | Month::August
        | Month::October
        | Month::December => 31,
        Month::April | Month::June | Month::September | Month::November => 30,
        Month::February => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn last_sunday_of_march_2026() {
        // EU DST 2026 starts Sun 29 Mar.
        assert_eq!(
            last_sunday(2026, Month::March),
            Date::from_calendar_date(2026, Month::March, 29).unwrap()
        );
    }

    #[test]
    fn last_sunday_of_october_2026() {
        // EU DST 2026 ends Sun 25 Oct.
        assert_eq!(
            last_sunday(2026, Month::October),
            Date::from_calendar_date(2026, Month::October, 25).unwrap()
        );
    }

    #[test]
    fn summer_is_plus_two() {
        // A July instant is CEST (+2).
        let july = datetime!(2026-07-06 07:00:00 UTC);
        assert_eq!(berlin_offset(july), UtcOffset::from_hms(2, 0, 0).unwrap());
    }

    #[test]
    fn winter_is_plus_one() {
        let jan = datetime!(2026-01-06 07:00:00 UTC);
        assert_eq!(berlin_offset(jan), UtcOffset::from_hms(1, 0, 0).unwrap());
    }

    #[test]
    fn parse_converts_utc_to_berlin_wallclock() {
        // 07:00Z in July → 09:00 Berlin; components read as 09:00.
        let dt = parse_berlin("2026-07-06T07:00:00.000000Z").unwrap();
        assert_eq!(dt.hour(), 9);
        assert_eq!(dt.minute(), 0);
    }
}
