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

use axum::{
    extract::{Form, Path},
    routing::{get, post},
};
use foodlib_new::auth_context::AuthCtx;
use maud::{html, Markup};
use serde::Deserialize;
use time::{macros::format_description, Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};

use crate::{frontend::MResponse, FoodLib, MyAppState};

pub(crate) fn timeline_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/{event_id}", get(timeline_view))
        .route("/{event_id}/move", post(handle_move))
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
    /// DB primary key of the underlying row (meal_id / tour_id / prep_id). Used
    /// by the drag/resize JS and the update endpoints to identify the row.
    /// `None` for helper shifts, which come from Engelsystem and are read-only.
    pub id: Option<i32>,
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

impl ItemKind {
    /// The URL path segment identifying this kind in drag-update requests.
    /// Helpers are read-only and have no segment.
    fn drag_key(self) -> Option<&'static str> {
        match self {
            ItemKind::Meal => Some("meal"),
            ItemKind::ShoppingTour => Some("tour"),
            ItemKind::FoodPrep => Some("prep"),
            ItemKind::Helper => None,
        }
    }

    /// Whether a bar of this kind can have its end time changed by resizing.
    /// Tours are instants (no meaningful duration); helpers are read-only.
    fn resizable(self) -> bool {
        matches!(self, ItemKind::Meal | ItemKind::FoodPrep)
    }
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
            id: Some(m.meal_id),
            label: m.name,
            detail: Some(format!("{} · {} Port.", m.place, m.servings)),
            start: m.start_time,
            end: m.end_time,
            class_override: None,
        });
    }

    // A shopping tour is an instant in the DB; give it a nominal 2-hour block
    // so it renders as a realistically-sized bar rather than a sliver.
    for t in tours {
        let start = t.tour_date;
        items.push(TimelineItem {
            kind: ItemKind::ShoppingTour,
            id: Some(t.id),
            label: t.store_name.unwrap_or_else(|| "Einkauf".to_string()),
            detail: None,
            start,
            end: start + Duration::hours(2),
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
            id: Some(p.id),
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
                id: None,
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
    /// Pack items into as few lanes as possible, then order the lanes so the
    /// longest bars sit on top. Two separate concerns, handled in two passes:
    ///
    /// 1. **Minimize height (lane count).** Items are visited in start-time order
    ///    and each goes into the first lane whose last bar has already ended;
    ///    otherwise a new lane opens. Start-time ordering is the classic optimal
    ///    greedy for interval partitioning — it always yields the minimum number
    ///    of lanes, so several non-overlapping meals share one row instead of
    ///    each waterfalling onto its own. (Duration-first ordering does NOT
    ///    guarantee that: a long early bar can block a lane a short later bar
    ///    could have shared, wasting vertical space.)
    ///
    /// 2. **Long lines on top.** The packed lanes are then reordered top-to-
    ///    bottom by [`lane_rank`], floating the lane with the longest bar up.
    ///
    /// The `(start, label, id)` sort key makes both passes deterministic for
    /// identical input.
    fn pack(mut items: Vec<TimelineItem>) -> Self {
        items.sort_by(|a, b| lane_sort_key(a).cmp(&lane_sort_key(b)));
        let mut lanes: Vec<Vec<TimelineItem>> = Vec::new();
        // `lane_ends[i]` is the end-time of the last (latest) bar in `lanes[i]`.
        // Because we visit in start order, that's always the lane's current tail.
        let mut lane_ends: Vec<OffsetDateTime> = Vec::new();
        for item in items {
            let slot = lane_ends.iter().position(|&end| end <= item.start);
            match slot {
                Some(i) => {
                    lane_ends[i] = item.end;
                    lanes[i].push(item);
                }
                None => {
                    lane_ends.push(item.end);
                    lanes.push(vec![item]);
                }
            }
        }
        // Float the visually-heaviest lanes to the top. `sort_by` is stable, so
        // lanes of equal rank keep their (start-ordered) packing order.
        lanes.sort_by(|a, b| lane_rank(a).cmp(&lane_rank(b)));
        Band { lanes }
    }
}

/// Ranking of a whole lane for its final top-to-bottom position within a band
/// (lower rank = higher on screen). This is the "long lines above short lines"
/// rule applied at the lane level, without affecting how densely each lane was
/// packed.
///
/// A lane is ranked by its single longest bar (longest first), which floats the
/// lane holding the biggest anchor bar to the top. Ties fall through to the
/// lane's earliest start and then its first label, so equal-weight lanes always
/// order the same way (deterministic layout).
fn lane_rank(lane: &[TimelineItem]) -> (std::cmp::Reverse<Duration>, OffsetDateTime, String) {
    let longest = lane
        .iter()
        .map(|i| i.end - i.start)
        .max()
        .unwrap_or(Duration::ZERO);
    // Bars are in start order within a lane, so the first is the earliest.
    let earliest_start = lane
        .first()
        .map(|i| i.start)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let first_label = lane.first().map(|i| i.label.clone()).unwrap_or_default();
    (std::cmp::Reverse(longest), earliest_start, first_label)
}

/// Sort key that drives lane packing: **start time first**, then by name, then
/// id — a total order so the packing (and thus the whole layout) is
/// deterministic for identical input. Start-first ordering is what guarantees
/// the minimum lane count; the "longest on top" concern is handled afterwards
/// by [`lane_rank`], not here.
fn lane_sort_key(item: &TimelineItem) -> (OffsetDateTime, String, i32) {
    (item.start, item.label.clone(), item.id.unwrap_or(i32::MAX))
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

/// The current wall-clock instant, in foodcalc's storage convention (local
/// wall-clock components carried as UTC — see the module header). We read this
/// value's `.date()`/`.time()` directly, exactly like stored meals, so the
/// cursor lines up with the bars without any offset conversion. Falls back to
/// UTC if the local offset can't be determined (headless server, etc.).
fn now_wall_clock() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
}

/// Render the live "now" cursor: a thin vertical playhead across a day-row,
/// like the current-time line in a video editor. Positioned by the same
/// logical-hour → percent mapping as the bars.
///
/// Returns nothing when "now" falls outside the visible hour window (e.g. it's
/// 03:00 but the window starts at 08:00) — a cursor pinned to the edge would
/// just be misleading. The element carries the window geometry as `data-*` so
/// `timeline.js` can recompute its position every few seconds and animate it,
/// and `data-tl-cursor` marks it for that script.
fn render_now_cursor(bounds: WindowBounds, now: OffsetDateTime) -> Markup {
    let hour = hour_pos(now);
    // Only draw when "now" is actually within the shown columns.
    if (hour as i64) < bounds.first_hour || (hour as i64) >= bounds.last_hour {
        return html! {};
    }
    let left = pct(hour, bounds);
    let time_fmt = format_description!("[hour]:[minute]");
    let label = now.time().format(&time_fmt).unwrap_or_default();
    html! {
        div class="tl-cursor absolute top-0 bottom-0 w-px bg-red-500 dark:bg-red-400 \
                   pointer-events-none z-20"
            style=(format!("left: {:.3}%", left))
            data-tl-cursor="1" {
            // A small time chip and a knob at the top, like a playhead handle.
            div class="absolute -top-0.5 left-1/2 -translate-x-1/2 h-2 w-2 rounded-full \
                       bg-red-500 dark:bg-red-400" {}
            div class="absolute -top-5 left-1/2 -translate-x-1/2 px-1 rounded text-[10px] \
                       leading-4 font-semibold whitespace-nowrap bg-red-500 text-white \
                       dark:bg-red-400 dark:text-red-950 tl-cursor-label" {
                (label)
            }
        }
    }
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
///
/// `event_id` and `day` (the bar's logical date) are emitted as `data-*` so the
/// drag/resize JS can reconstruct the wall-clock timestamp and POST an update.
/// Meal/tour/prep bars are made draggable; helper bars stay static.
fn render_bar(item: &TimelineItem, bounds: WindowBounds, event_id: i32, day: Date) -> Markup {
    let left = pct(hour_pos(item.start), bounds);
    let right = pct(end_hour_pos(item), bounds);
    let width = (right - left).max(1.5);
    let time_fmt = format_description!("[hour]:[minute]");
    let start_label = item.start.time().format(&time_fmt).unwrap_or_default();
    let end_label = item.end.time().format(&time_fmt).unwrap_or_default();
    let colors = item.class_override.unwrap_or_else(|| item.kind.bar_classes());

    // A bar is interactive when it has both a DB id and a drag key (i.e. not a
    // read-only helper). The JS attaches handlers by the `data-drag` marker.
    let drag_key = item.id.and_then(|id| item.kind.drag_key().map(|k| (id, k)));
    let day_iso = day
        .format(format_description!("[year]-[month]-[day]"))
        .unwrap_or_default();
    let resizable = drag_key.is_some() && item.kind.resizable();
    // `group` lets the edge handles reveal themselves on bar hover (Tailwind
    // `group-hover`). `overflow-hidden` keeps the label clipped but we inset the
    // handles so they aren't cut off by the rounded corners.
    let base_class = format!(
        "group absolute top-0.5 bottom-0.5 rounded border px-2 py-0.5 text-xs \
         overflow-hidden whitespace-nowrap flex items-center {} {}",
        colors,
        if drag_key.is_some() { "cursor-grab select-none tl-bar" } else { "" },
    );

    html! {
        div class=(base_class)
            style=(format!("left: {:.3}%; width: {:.3}%", left, width))
            title=(format!("{}{} ({}–{})",
                item.label,
                item.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default(),
                start_label, end_label))
            data-drag=[drag_key.map(|(_, k)| k)]
            data-id=[drag_key.map(|(id, _)| id)]
            data-event=(event_id)
            data-day=(day_iso)
            data-resizable=(resizable)
            // Double-click target: meal bars open their edit form. Only meals
            // have an edit page in scope here.
            data-edit=[(item.kind == ItemKind::Meal)
                .then(|| item.id.map(|id| format!("/events/edit/event_edit_meal/{event_id}/{id}")))
                .flatten()] {
            span class="font-semibold pointer-events-none" { (item.label) }
            @if let Some(detail) = &item.detail {
                span class="opacity-80 ml-1 pointer-events-none" { "— " (detail) }
            }
            // Resize handles on BOTH edges for meals/prep. `tl-resize` is the JS
            // hook; `data-edge` tells it which end moves. The grip is faint until
            // you hover the bar, then brightens — a clear "drag me" affordance.
            @if resizable {
                (resize_handle("left"))
                (resize_handle("right"))
            }
        }
    }
}

/// One edge resize handle: a full-height hit target pinned to the given edge
/// (`"left"` or `"right"`), showing a two-bar grip glyph. The cursor is
/// *directional* — `w-resize` (←) on the left edge, `e-resize` (→) on the right —
/// so hovering an edge signals which way it will extend. The grip fades in on
/// bar hover via `group-hover`. `tl-resize` + `data-edge` are the JS hooks.
fn resize_handle(edge: &str) -> Markup {
    let (side, cursor) = if edge == "left" {
        ("left-0", "cursor-w-resize")
    } else {
        ("right-0", "cursor-e-resize")
    };
    html! {
        div class=(format!(
                "tl-resize absolute {} top-0 bottom-0 w-2.5 {} \
                 flex items-center justify-center", side, cursor))
            data-edge=(edge) {
            // Two short vertical bars = the standard resize grip. Faint by
            // default, full-opacity while hovering the bar.
            div class="flex gap-px opacity-40 group-hover:opacity-90 pointer-events-none" {
                span class="block w-px h-3 bg-current" {}
                span class="block w-px h-3 bg-current" {}
            }
        }
    }
}

/// Render one lane (grid row): a relative track holding the gridlines plus the
/// absolutely-positioned bars assigned to this lane.
fn render_lane(lane: &[TimelineItem], bounds: WindowBounds, event_id: i32, day: Date) -> Markup {
    html! {
        div class="relative h-7 w-full" {
            (render_gridlines(bounds))
            @for item in lane {
                (render_bar(item, bounds, event_id, day))
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

    // "Now" in wall-clock terms, and the logical day it belongs to. Both drive
    // the current-day highlight, the focus-on-load scroll, and the live cursor.
    let now = now_wall_clock();
    let today = logical_date(now);
    let today_iso = today
        .format(format_description!("[year]-[month]-[day]"))
        .unwrap_or_default();

    let weekday_fmt = format_description!("[weekday repr:long]");
    let date_fmt = format_description!("[day].[month].[year]");

    Ok(html! {
        // `data-timeline` marks the region the drag/resize JS operates on.
        // The JS converts a bar's pixel offset within a lane to minutes:
        //   data-cols       = number of hour columns in the shared window
        //   data-start-hour = the window's left edge, in LOGICAL hours since
        //                     DAY_START_HOUR (so a bar's window-relative minutes
        //                     + start-hour*60 == minutes-since-DAY_START_HOUR,
        //                     which is what the /move endpoint expects)
        //   data-snap       = snap granularity in minutes
        //   data-today      = today's logical date (ISO) — the row to focus on
        //                     load and to hang the live cursor on
        //
        // Background refresh: the section re-fetches itself every 10 minutes and
        // swaps in the fresh render (so a long-open timeline stays current with
        // other editors and the cursor's day). `hx-trigger` includes `htmx:abort`
        // suppression via the JS, which cancels the poll while a drag is active
        // so a refresh never yanks the DOM mid-gesture (see timeline.js).
        section class="w-full flex flex-col gap-4 p-4" id="timeline-root"
            data-timeline="1"
            data-event=(event_id)
            data-cols=(bounds.cols())
            data-start-hour=(bounds.first_hour)
            data-snap="30"
            data-today=(today_iso)
            hx-get=(format!("/events/edit/timeline/{event_id}"))
            hx-trigger="every 600s"
            hx-target="#timeline-root"
            hx-swap="outerHTML"
            hx-sync="this:replace" {
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
                @let day_iso = day.date
                    .format(format_description!("[year]-[month]-[day]"))
                    .unwrap_or_default();
                @let is_today = day.date == today;
                // `data-tl-day` is the drop-target + focus hook: the JS hit-tests
                // it to route a cross-day drop and scrolls today's row into view.
                // Today's row gets a subtle ring so it's easy to spot on load.
                div class=(format!(
                        "rounded-lg border p-3 overflow-x-auto {}",
                        if is_today {
                            "border-red-400/70 dark:border-red-400/60 ring-1 ring-red-400/40"
                        } else {
                            "border-gray-700"
                        }))
                    data-tl-day=(day_iso) {
                    div class="font-semibold flex items-center gap-2" {
                        (day.date.format(weekday_fmt).unwrap_or_default())
                        ", "
                        (day.date.format(date_fmt).unwrap_or_default())
                        @if is_today {
                            span class="text-xs font-normal px-1.5 rounded bg-red-500 text-white \
                                       dark:bg-red-400 dark:text-red-950" { "Heute" }
                        }
                    }
                    // Single inner track sized to the full window width so the
                    // outer `overflow-x-auto` has something wider than itself to
                    // scroll on narrow screens. `min-width` keeps it filling the
                    // container on desktop, `width: max-content` lets it grow.
                    // `relative` makes it the positioning context for the cursor.
                    div class="relative flex flex-col gap-1" style=(format!("min-width: {}rem", track_min_width_rem(bounds))) {
                        (render_hour_axis(bounds))
                        @for band in &day.bands {
                            @for lane in &band.lanes {
                                (render_lane(lane, bounds, event_id, day.date))
                            }
                        }
                        // Live "now" playhead, only on today's row (per design).
                        @if is_today {
                            (render_now_cursor(bounds, now))
                        }
                    }
                }
            }
        }
    })
}

// ── Drag / resize updates ────────────────────────────────────────────────────

/// Payload for a drag/resize update. Positions are in the timeline's own
/// coordinate system: `day` is the logical date the bar sits on (ISO
/// `YYYY-MM-DD`), and `start_min`/`end_min` are minutes since `DAY_START_HOUR`
/// (i.e. 0 == 04:00) on that logical day, snapped to 30-minute steps by the JS.
/// `kind` is one of `meal` | `tour` | `prep`. `end_min` is absent for a plain
/// move of a tour (an instant with no duration).
#[derive(Deserialize)]
struct MoveForm {
    kind: String,
    id: i32,
    day: String,
    start_min: i64,
    end_min: Option<i64>,
}

/// Reconstruct a stored `OffsetDateTime` from a logical `day` + minutes-since-
/// `DAY_START_HOUR`. Minutes ≥ (24h − DAY_START_HOUR) roll onto the next
/// calendar day (the early-morning tail of the logical day). The result is
/// built as UTC to match foodcalc's wall-clock-as-UTC storage convention.
fn to_stored(day: Date, minutes_since_day_start: i64) -> Option<OffsetDateTime> {
    let real = minutes_since_day_start + DAY_START_HOUR * 60;
    let (date, minute_of_day) = if real >= 24 * 60 {
        (day.next_day()?, real - 24 * 60)
    } else {
        (day, real)
    };
    let time = Time::from_hms(
        (minute_of_day / 60).clamp(0, 23) as u8,
        (minute_of_day % 60) as u8,
        0,
    )
    .ok()?;
    Some(PrimitiveDateTime::new(date, time).assume_utc())
}

/// Apply a drag/resize update to a meal, tour or prep, then re-render the whole
/// timeline. Read-modify-write: the current row is fetched and only its time
/// fields are patched, so no other data is trusted from the client.
async fn handle_move(
    foodlib: FoodLib,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
    Form(form): Form<MoveForm>,
) -> MResponse {
    ctx.assert_can_edit_event(event_id).await?;

    log::info!(
        "timeline move: kind={} id={} day={} start_min={} end_min={:?}",
        form.kind, form.id, form.day, form.start_min, form.end_min
    );

    let day = Date::parse(&form.day, format_description!("[year]-[month]-[day]"))
        .map_err(|e| foodlib_new::Error::Misc(format!("bad day: {e}")))?;
    let new_start = to_stored(day, form.start_min)
        .ok_or_else(|| foodlib_new::Error::Misc("invalid start time".into()))?;

    match form.kind.as_str() {
        "meal" => {
            let m = foodlib.meals().get_meal(form.id).await?;
            // Preserve the meal's duration on a plain move; honor end_min on a
            // resize. Guard against a non-positive span by falling back to the
            // original duration.
            let new_end = match form.end_min {
                Some(end_min) => to_stored(day, end_min)
                    .filter(|e| *e > new_start)
                    .unwrap_or(new_start + (m.end_time - m.start_time)),
                None => new_start + (m.end_time - m.start_time),
            };
            foodlib
                .meals()
                .update_meal(
                    m.meal_id,
                    m.recipe_id,
                    m.place_id,
                    new_start,
                    new_end,
                    m.energy,
                    m.servings,
                    m.comment,
                )
                .await?;
        }
        "tour" => {
            let mut tour = foodlib.events().get_shopping_tour(form.id).await?;
            tour.tour_date = new_start;
            foodlib.events().update_shopping_tour(tour).await?;
        }
        "prep" => {
            let mut prep = foodlib.events().get_food_prep(form.id).await?;
            // Keep the use window's length relative to the (possibly changed)
            // prep time so a moved prep still "uses until" the same offset.
            let delta = new_start - prep.prep_date;
            prep.prep_date = new_start;
            prep.use_until += delta;
            if let Some(from) = prep.use_from {
                prep.use_from = Some(from + delta);
            }
            foodlib.events().update_food_prep(prep).await?;
        }
        other => {
            return Err(foodlib_new::Error::Misc(format!("unknown kind: {other}")));
        }
    }

    timeline_view(foodlib, ctx, Path(event_id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn to_stored_maps_day_start_to_04() {
        // 0 minutes since day start == 04:00 on the same date.
        let dt = to_stored(date!(2026 - 07 - 06), 0).unwrap();
        assert_eq!(dt.date(), date!(2026 - 07 - 06));
        assert_eq!(dt.hour(), 4);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn to_stored_midday() {
        // 09:00 == 5h after 04:00 == 300 minutes.
        let dt = to_stored(date!(2026 - 07 - 06), 300).unwrap();
        assert_eq!(dt.hour(), 9);
    }

    #[test]
    fn to_stored_rolls_past_midnight() {
        // 21h after 04:00 == 01:00 the NEXT calendar day.
        let dt = to_stored(date!(2026 - 07 - 06), 21 * 60).unwrap();
        assert_eq!(dt.date(), date!(2026 - 07 - 07));
        assert_eq!(dt.hour(), 1);
    }

    /// Build a meal item spanning [start_h, end_h) o'clock on a fixed test day.
    fn meal(label: &str, start_h: u8, end_h: u8) -> TimelineItem {
        let day = date!(2026 - 07 - 06);
        let at = |h: u8| {
            PrimitiveDateTime::new(day, Time::from_hms(h, 0, 0).unwrap()).assume_utc()
        };
        TimelineItem {
            kind: ItemKind::Meal,
            id: Some(1),
            label: label.to_string(),
            detail: None,
            start: at(start_h),
            end: at(end_h),
            class_override: None,
        }
    }

    #[test]
    fn pack_no_overlap_within_lane() {
        // Two overlapping meals must land in different lanes; a third that fits
        // the gap after the first reuses a lane rather than opening a new one.
        let band = Band::pack(vec![
            meal("A", 8, 12),  // long
            meal("B", 9, 10),  // overlaps A
            meal("C", 13, 14), // after everything
        ]);
        // A and B overlap -> 2 lanes; C fits after A in A's lane.
        assert_eq!(band.lanes.len(), 2);
        for lane in &band.lanes {
            for pair in lane.windows(2) {
                assert!(
                    pair[0].end <= pair[1].start,
                    "lane items must not overlap and must be in start order",
                );
            }
        }
    }

    #[test]
    fn pack_uses_minimum_lanes() {
        // Compactness: several short, non-overlapping meals must all share ONE
        // lane rather than waterfalling. The max simultaneous overlap here is 1,
        // so the minimum (and required) lane count is 1.
        let band = Band::pack(vec![
            meal("a", 8, 9),
            meal("b", 10, 11),
            meal("c", 12, 13),
            meal("d", 14, 15),
        ]);
        assert_eq!(band.lanes.len(), 1, "non-overlapping meals should share a lane");
        assert_eq!(band.lanes[0].len(), 4);
    }

    #[test]
    fn pack_long_bar_does_not_waste_a_lane() {
        // Regression for the "reduce vertical height" fix: a long bar must not
        // block a lane that shorter, non-overlapping bars could share. Max
        // simultaneous overlap is 2 (Long always overlaps exactly one short at a
        // time), so the optimal layout is 2 lanes — one for Long, one holding all
        // three shorts. Duration-first packing would have wasted a 3rd lane here.
        let band = Band::pack(vec![
            meal("Long", 8, 16),
            meal("s1", 8, 9),
            meal("s2", 10, 11),
            meal("s3", 12, 13),
        ]);
        assert_eq!(band.lanes.len(), 2, "long bar must not waste a lane");
        // And the Long lane is on top (longest-on-top ordering).
        assert!(band.lanes[0].iter().any(|i| i.label == "Long"));
        // The three shorts all share the other lane.
        assert_eq!(band.lanes[1].len(), 3);
    }

    #[test]
    fn pack_longest_lane_on_top() {
        // The lane holding the longest bar ranks first (top of the band).
        let band = Band::pack(vec![
            meal("short", 8, 9),
            meal("long", 8, 16), // longest -> its lane should be lane 0
        ]);
        assert_eq!(band.lanes.len(), 2);
        let top_has_long = band.lanes[0].iter().any(|i| i.label == "long");
        assert!(top_has_long, "the lane with the longest bar should be on top");
    }

    #[test]
    fn pack_is_deterministic_by_name() {
        // Two same-duration, same-start meals differing only by name must always
        // pack in the same (name-sorted) order regardless of input order.
        let forward = Band::pack(vec![meal("alpha", 8, 9), meal("bravo", 8, 9)]);
        let reversed = Band::pack(vec![meal("bravo", 8, 9), meal("alpha", 8, 9)]);
        let names = |b: &Band| -> Vec<String> {
            b.lanes.iter().flatten().map(|i| i.label.clone()).collect()
        };
        assert_eq!(names(&forward), names(&reversed));
        // And "alpha" comes before "bravo" (name tiebreak, lane 0 first).
        assert_eq!(names(&forward), vec!["alpha".to_string(), "bravo".to_string()]);
    }
}
