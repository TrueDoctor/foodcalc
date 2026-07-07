//! Read-only Gantt-style timeline for an event.
//!
//! Reconstructs the "big event overview" (previously a hand-maintained Excel
//! sheet) purely from existing DB state:
//!   - `event_meals`   -> meal bars   (start_time .. end_time)
//!   - `shopping_tours`-> tour markers (tour_date, nominal block)
//!   - `food_prep`     -> prep bars    (prep_date .. use_until)
//!
//! Layout: each day is a stack of horizontal "lanes". Every day shares one
//! event-wide hour window (so columns line up across days), drawn full-width
//! with gray hour gridlines. Items are packed into lanes per kind (tours, prep,
//! meals) via greedy interval scheduling, so non-overlapping bars reuse a lane
//! instead of waterfalling down. Bars are absolutely positioned by fractional
//! hour (percentage left/width) for exact sub-hour placement. No JavaScript.
//! This is the v1 read-only view; free-form activities, helper counts
//! (Engelsystem) and cost annotations are intentionally out of scope for now.
//!
//! Time handling: every timestamp column is `timestamptz`/`OffsetDateTime`, but
//! the app stores the wall-clock time the user typed via `.assume_utc()` (see
//! `parse_datetime_local`). So the stored value's UTC components ARE the numbers
//! the user sees. We therefore read `.date()`/`.time()` directly and never
//! convert offsets — matching how meals and shopping tours are displayed.

use std::collections::HashMap;

use axum::{extract::Path, routing::get};
use foodlib_new::auth_context::AuthCtx;
use maud::{html, Markup};
use time::{macros::format_description, Date, Duration, OffsetDateTime};

use crate::{frontend::MResponse, FoodLib, MyAppState};

pub(crate) fn timeline_router() -> axum::Router<MyAppState> {
    axum::Router::new().route("/{event_id}", get(timeline_view))
}

/// Width of one grid slot, in minutes. Bars snap to this granularity.
/// One column per hour so a full day fits a desktop screen; the fractional
/// position within an hour is still honored (bars start/end mid-column).
const SLOT_MINUTES: i64 = 60;

/// Number of slots in a full day — the theoretical max width of a row.
const SLOTS_PER_DAY: i64 = 24 * 60 / SLOT_MINUTES;

/// Minimum rendered width of one hour column. On desktop the grid stretches to
/// fill the container (columns wider than this); on narrow screens it falls
/// back to this width and the row scrolls horizontally.
const MIN_COL_REM: f64 = 2.75;

/// Engelsystem location whose helper shifts are shown on the timeline.
/// Hardcoded for v1 (the event's kitchen location).
const ENGELSYSTEM_LOCATION_ID: i32 = 4;

/// Which kind of activity a timeline bar represents. Drives the bar color and
/// the legend, mirroring the categories from the original Excel legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// A served meal (`event_meals`). Orange in the Excel.
    Meal,
    /// A shopping run (`shopping_tours`). Blue in the Excel.
    ShoppingTour,
    /// A food-prep task (`food_prep`). Yellow/amber in the Excel.
    FoodPrep,
    /// A helper shift pulled from Engelsystem (read-only). Fully-staffed shifts
    /// render green, understaffed ones red — see [`ItemKind::helper_classes`].
    Helper,
}

impl ItemKind {
    /// Tailwind classes for the bar background/border/text. Chosen to read in
    /// both light and dark mode (media-based dark mode, per project setup).
    fn bar_classes(self) -> &'static str {
        match self {
            ItemKind::Meal => {
                "bg-orange-200 border-orange-400 text-orange-950 \
                 dark:bg-orange-500/40 dark:border-orange-400/60 dark:text-orange-50"
            }
            ItemKind::ShoppingTour => {
                "bg-sky-200 border-sky-400 text-sky-950 \
                 dark:bg-sky-500/40 dark:border-sky-400/60 dark:text-sky-50"
            }
            ItemKind::FoodPrep => {
                "bg-amber-200 border-amber-400 text-amber-950 \
                 dark:bg-amber-500/40 dark:border-amber-400/60 dark:text-amber-50"
            }
            // Base (fully-staffed) helper color; understaffed shifts override
            // this per-item via `TimelineItem::class_override`.
            ItemKind::Helper => Self::helper_classes(true),
        }
    }

    /// Helper-shift colors keyed on staffing: green when full, red when short.
    fn helper_classes(full: bool) -> &'static str {
        if full {
            "bg-emerald-200 border-emerald-400 text-emerald-950 \
             dark:bg-emerald-500/40 dark:border-emerald-400/60 dark:text-emerald-50"
        } else {
            "bg-red-200 border-red-400 text-red-950 \
             dark:bg-red-500/40 dark:border-red-400/60 dark:text-red-50"
        }
    }

    fn legend_label(self) -> &'static str {
        match self {
            ItemKind::Meal => "Essen",
            ItemKind::ShoppingTour => "Einkauf",
            ItemKind::FoodPrep => "Vorkochen",
            ItemKind::Helper => "Helfer",
        }
    }
}

/// A single normalized bar on the timeline. All three DB sources collapse into
/// this shape so the renderer only deals with one type.
#[derive(Debug, Clone)]
pub struct TimelineItem {
    pub kind: ItemKind,
    pub label: String,
    /// Secondary line shown under the label (place, servings, …).
    pub detail: Option<String>,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    /// Optional bar-color override (Tailwind classes). Used by helper shifts to
    /// switch between the green "full" and red "understaffed" palettes. When
    /// `None`, the color comes from `kind.bar_classes()`.
    pub class_override: Option<&'static str>,
}

/// Build the normalized item list for an event from the three DB sources.
///
/// Food-prep rows only carry a `recipe_id`, so recipe names are resolved once
/// via a single `recipes().list()` into a lookup map (avoids N queries).
async fn collect_items(foodlib: &FoodLib, event_id: i32) -> foodlib_new::Result<Vec<TimelineItem>> {
    let meals = foodlib.meals().get_event_meals(event_id).await?;
    let tours = foodlib.events().get_shopping_tours(event_id).await?;
    let preps = foodlib.events().get_food_prep_tasks(event_id).await?;

    let recipe_names: HashMap<i32, String> = foodlib
        .recipes()
        .list()
        .await?
        .into_iter()
        .map(|r| (r.id, r.name))
        .collect();

    let mut items = Vec::new();

    for m in meals {
        items.push(TimelineItem {
            kind: ItemKind::Meal,
            label: m.name,
            detail: Some(format!("{} · {} Port.", m.place, m.servings)),
            start: m.start_time,
            end: m.end_time,
            class_override: None,
        });
    }

    // A shopping tour is an instant in the DB; give it a nominal block so it
    // renders as a visible bar rather than a zero-width sliver.
    for t in tours {
        let start = t.tour_date;
        items.push(TimelineItem {
            kind: ItemKind::ShoppingTour,
            label: t.store_name.unwrap_or_else(|| "Einkauf".to_string()),
            detail: None,
            start,
            end: start + Duration::minutes(SLOT_MINUTES),
            class_override: None,
        });
    }

    // Food prep is anchored at `prep_date` (when the cooking happens). The
    // stored `use_until` span is often huge (a stock that lasts weeks), which
    // is a shelf-life fact, not a timeline event — so we render a fixed nominal
    // block on the prep day rather than a bar stretching across every day.
    for p in preps {
        let label = recipe_names
            .get(&p.recipe_id)
            .cloned()
            .unwrap_or_else(|| format!("Rezept #{}", p.recipe_id));
        items.push(TimelineItem {
            kind: ItemKind::FoodPrep,
            label,
            detail: None,
            start: p.prep_date,
            end: p.prep_date + Duration::minutes(2 * SLOT_MINUTES),
            class_override: None,
        });
    }

    // Helper shifts from Engelsystem (read-only overlay). Only the shifts that
    // fall within the event's own day range are relevant — Engelsystem may hold
    // shifts for other events/days at the same location. Best-effort: an unset
    // integration or a failed fetch simply contributes no bars.
    if let Some(engel) = foodlib_new::ops::engelsystem::EngelSystem::from_env() {
        let event_days: std::collections::HashSet<time::Date> =
            items.iter().map(|i| logical_date(i.start)).collect();
        for shift in engel.get_location_shifts(ENGELSYSTEM_LOCATION_ID).await {
            if !event_days.is_empty() && !event_days.contains(&logical_date(shift.start)) {
                continue;
            }
            let label = if shift.shift_type.is_empty() {
                shift.title.clone()
            } else {
                shift.shift_type.clone()
            };
            items.push(TimelineItem {
                kind: ItemKind::Helper,
                label,
                detail: Some(format!("Helfer {}/{}", shift.filled, shift.needed)),
                start: shift.start,
                end: shift.end,
                class_override: Some(ItemKind::helper_classes(shift.is_full())),
            });
        }
    }

    Ok(items)
}

/// Fractional grid position of an instant, in hours-since-midnight (e.g. 13:30
/// -> 13.5). The stored `OffsetDateTime` is already the user's wall-clock (via
/// `.assume_utc()`), so components are read directly — no offset conversion.
/// The hour at which a new "day" starts on the timeline. Catering schedules run
/// late, so anything before this hour is treated as the tail of the previous
/// day rather than a lone sliver at the top of the next — much easier to read.
const DAY_START_HOUR: i64 = 4;

/// The calendar date of the *logical* day an instant belongs to. Times before
/// `DAY_START_HOUR` roll back to the previous calendar date, so e.g. a 00:30
/// meal groups with the evening it belongs to.
fn logical_date(ts: OffsetDateTime) -> Date {
    if (ts.time().hour() as i64) < DAY_START_HOUR {
        ts.date().previous_day().expect("date within representable range")
    } else {
        ts.date()
    }
}

/// Position of an instant in "hours since the logical day's `DAY_START_HOUR`",
/// in `[0, 24)`. 04:00 -> 0, 03:59 -> ~24. Used for both axis labels and bar
/// placement so everything shares the 04:00-anchored coordinate system.
fn hour_pos(ts: OffsetDateTime) -> f64 {
    let raw = (ts.time().hour() as f64) + (ts.time().minute() as f64) / 60.0;
    let shifted = raw - DAY_START_HOUR as f64;
    if shifted < 0.0 {
        shifted + SLOTS_PER_DAY as f64
    } else {
        shifted
    }
}

/// End position of an item's bar, in the logical-day hour coordinate.
///
/// Bars are drawn on their start's logical day, so an end that lands on a later
/// logical day (a shift/meal running past the next 04:00 boundary) is clamped
/// to 24 — the bar runs to the right edge of its day rather than wrapping. The
/// same clamp catches bad data where `end <= start` (e.g. an end time
/// mistakenly set earlier than the start): the bar extends to day's end instead
/// of rendering negative-width.
fn end_hour_pos(item: &TimelineItem) -> f64 {
    if logical_date(item.end) > logical_date(item.start) || item.end <= item.start {
        SLOTS_PER_DAY as f64
    } else {
        hour_pos(item.end)
    }
}

/// The hour window every day-row shares, so all days line up column-for-column.
/// Derived once from the earliest start and latest end across the whole event,
/// snapped out to whole hours and padded, then clamped to a real day.
#[derive(Clone, Copy)]
struct WindowBounds {
    /// First hour column shown (0..=23).
    first_hour: i64,
    /// One past the last hour column shown (1..=24).
    last_hour: i64,
}

impl WindowBounds {
    /// Total number of hour columns in a row.
    fn cols(self) -> i64 {
        self.last_hour - self.first_hour
    }

    /// Compute the shared window from all items. Falls back to a sensible
    /// working-day window (08:00–20:00) when the event has no items yet.
    fn from_items(items: &[TimelineItem]) -> Self {
        let earliest = items.iter().map(|i| hour_pos(i.start)).fold(f64::MAX, f64::min);
        let latest = items.iter().map(end_hour_pos).fold(f64::MIN, f64::max);
        if items.is_empty() || earliest > latest {
            return WindowBounds { first_hour: 8, last_hour: 20 };
        }
        // Floor/ceil to whole hours, pad one hour each side, clamp to [0, 24].
        let first_hour = ((earliest.floor() as i64) - 1).max(0);
        let last_hour = ((latest.ceil() as i64) + 1).min(SLOTS_PER_DAY);
        WindowBounds { first_hour, last_hour }
    }
}

/// A group of same-kind items packed into horizontal lanes (grid rows), so that
/// non-overlapping bars share a lane instead of each taking its own row.
struct Band {
    /// `lanes[i]` holds the items assigned to lane (grid row) `i`.
    lanes: Vec<Vec<TimelineItem>>,
}

impl Band {
    /// Greedy interval packing: sort by start, then place each item in the first
    /// lane whose last item ends at or before this item starts; otherwise open a
    /// new lane. Classic activity-selection lane assignment.
    fn pack(mut items: Vec<TimelineItem>) -> Self {
        items.sort_by_key(|i| i.start);
        let mut lanes: Vec<Vec<TimelineItem>> = Vec::new();
        for item in items {
            let slot = lanes
                .iter_mut()
                .find(|lane| lane.last().is_none_or(|prev| prev.end <= item.start));
            match slot {
                Some(lane) => lane.push(item),
                None => lanes.push(vec![item]),
            }
        }
        Band { lanes }
    }
}

/// One calendar day's worth of items, grouped into per-kind packed bands.
/// Bands render in a fixed order (tours, prep, meals) so categories stay in
/// consistent horizontal bands like the original Excel.
struct DayRow {
    date: Date,
    bands: Vec<Band>,
}

/// Group items by start day, then within each day pack each kind into lanes.
/// Days with no items are omitted; days are returned in chronological order.
fn group_by_day(items: &[TimelineItem]) -> Vec<DayRow> {
    let mut by_date: HashMap<Date, Vec<TimelineItem>> = HashMap::new();
    for item in items {
        by_date
            .entry(logical_date(item.start))
            .or_default()
            .push(item.clone());
    }

    // Fixed band order — matches the reading order of the legend.
    const KIND_ORDER: [ItemKind; 4] = [
        ItemKind::ShoppingTour,
        ItemKind::FoodPrep,
        ItemKind::Meal,
        ItemKind::Helper,
    ];

    let mut rows: Vec<DayRow> = by_date
        .into_iter()
        .map(|(date, day_items)| {
            let bands = KIND_ORDER
                .iter()
                .filter_map(|&kind| {
                    let of_kind: Vec<TimelineItem> =
                        day_items.iter().filter(|i| i.kind == kind).cloned().collect();
                    (!of_kind.is_empty()).then(|| Band::pack(of_kind))
                })
                .collect();
            DayRow { date, bands }
        })
        .collect();

    rows.sort_by_key(|r| r.date);
    rows
}

/// The min inner width of a day-row's timeline track, in rem: one `MIN_COL_REM`
/// per hour column. The track is `width: 100%` but never narrower than this, so
/// it fills a desktop screen yet scrolls on mobile.
fn track_min_width_rem(bounds: WindowBounds) -> f64 {
    bounds.cols() as f64 * MIN_COL_REM
}

/// Convert an hours-since-midnight position to a percentage across the window
/// [0,100], for absolute positioning within a track.
fn pct(hour: f64, bounds: WindowBounds) -> f64 {
    let pos = (hour - bounds.first_hour as f64).clamp(0.0, bounds.cols() as f64);
    pos / bounds.cols() as f64 * 100.0
}

/// Render the hour-label axis: one label per hour column, left-aligned over its
/// gridline. Uses the same flex track so labels line up with the bars below.
fn render_hour_axis(bounds: WindowBounds) -> Markup {
    html! {
        div class="flex w-full" {
            @for hour in bounds.first_hour..bounds.last_hour {
                // Axis positions are logical hours (0 = DAY_START_HOUR); convert
                // back to the real clock hour for the label.
                @let clock = (hour + DAY_START_HOUR) % SLOTS_PER_DAY;
                div class="flex-1 text-xs text-gray-500 dark:text-gray-400 border-l border-gray-300 dark:border-gray-700 pl-1" {
                    (format!("{:02}:00", clock))
                }
            }
        }
    }
}

/// Render the vertical hour gridlines as an absolutely-positioned background
/// layer inside a track (so bars sit on top of them).
fn render_gridlines(bounds: WindowBounds) -> Markup {
    html! {
        div class="absolute inset-0 flex pointer-events-none" {
            @for _hour in bounds.first_hour..bounds.last_hour {
                div class="flex-1 border-l border-gray-200/70 dark:border-gray-700/60" {}
            }
        }
    }
}

/// Render one bar as an absolutely-positioned block within its lane track,
/// using percentage left/width from fractional hours (exact sub-hour position).
fn render_bar(item: &TimelineItem, bounds: WindowBounds) -> Markup {
    let left = pct(hour_pos(item.start), bounds);
    let right = pct(end_hour_pos(item), bounds);
    let width = (right - left).max(1.5);
    let time_fmt = format_description!("[hour]:[minute]");
    let start_label = item.start.time().format(&time_fmt).unwrap_or_default();
    let end_label = item.end.time().format(&time_fmt).unwrap_or_default();
    let colors = item.class_override.unwrap_or_else(|| item.kind.bar_classes());
    html! {
        div class=(format!(
                "absolute top-0.5 bottom-0.5 rounded border px-2 py-0.5 text-xs \
                 overflow-hidden whitespace-nowrap flex items-center {}",
                colors
            ))
            style=(format!("left: {:.3}%; width: {:.3}%", left, width))
            title=(format!("{} ({}–{})", item.label, start_label, end_label)) {
            span class="font-semibold" { (item.label) }
            @if let Some(detail) = &item.detail {
                span class="opacity-80 ml-1" { "— " (detail) }
            }
        }
    }
}

/// Render one lane (grid row): a relative track holding the gridlines plus the
/// absolutely-positioned bars assigned to this lane.
fn render_lane(lane: &[TimelineItem], bounds: WindowBounds) -> Markup {
    html! {
        div class="relative h-7 w-full" {
            (render_gridlines(bounds))
            @for item in lane {
                (render_bar(item, bounds))
            }
        }
    }
}

/// Render the whole timeline for one event.
pub async fn timeline_view(foodlib: FoodLib, ctx: AuthCtx, Path(event_id): Path<i32>) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;

    let event = foodlib.events().get(event_id).await?;
    let items = collect_items(&foodlib, event_id).await?;
    let bounds = WindowBounds::from_items(&items);
    let days = group_by_day(&items);

    let weekday_fmt = format_description!("[weekday repr:long]");
    let date_fmt = format_description!("[day].[month].[year]");

    Ok(html! {
        section class="w-full flex flex-col gap-4 p-4" {
            div class="flex flex-row items-center justify-between flex-wrap gap-3" {
                h1 class="text-2xl font-semibold" { (event.name) " — Timeline" }
                a class="btn btn-primary" hx-target="#content" hx-push-url="true"
                    hx-get=(format!("/events/edit/{}", event_id)) { "Zurück zum Event" }
            }

            // Legend. Helper shifts appear in two colors (fully staffed vs
            // understaffed), so they get two swatches.
            div class="flex flex-row flex-wrap gap-4 text-sm" {
                @for kind in [ItemKind::ShoppingTour, ItemKind::FoodPrep, ItemKind::Meal] {
                    div class="flex flex-row items-center gap-2" {
                        span class=(format!("inline-block h-3 w-5 rounded border {}", kind.bar_classes())) {}
                        span { (kind.legend_label()) }
                    }
                }
                div class="flex flex-row items-center gap-2" {
                    span class=(format!("inline-block h-3 w-5 rounded border {}", ItemKind::helper_classes(true))) {}
                    span { "Helfer (voll)" }
                }
                div class="flex flex-row items-center gap-2" {
                    span class=(format!("inline-block h-3 w-5 rounded border {}", ItemKind::helper_classes(false))) {}
                    span { "Helfer (fehlt)" }
                }
            }

            @if days.is_empty() {
                p class="text-gray-500 dark:text-gray-400" {
                    "Für dieses Event sind noch keine Mahlzeiten, Einkäufe oder Vorkoch-Aufgaben geplant."
                }
            }

            @for day in &days {
                div class="rounded-lg border border-gray-700 p-3 overflow-x-auto" {
                    div class="font-semibold" {
                        (day.date.format(weekday_fmt).unwrap_or_default())
                        ", "
                        (day.date.format(date_fmt).unwrap_or_default())
                    }
                    // Single inner track sized to the full window width so the
                    // outer `overflow-x-auto` has something wider than itself to
                    // scroll on narrow screens. `min-width` keeps it filling the
                    // container on desktop, `width: max-content` lets it grow.
                    div class="flex flex-col gap-1" style=(format!("min-width: {}rem", track_min_width_rem(bounds))) {
                        (render_hour_axis(bounds))
                        @for band in &day.bands {
                            @for lane in &band.lanes {
                                (render_lane(lane, bounds))
                            }
                        }
                    }
                }
            }
        }
    })
}
