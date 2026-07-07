//! Monochrome inline SVG icons (Font Awesome 6 "solid" set) for use in action
//! toolbars. Icons inherit the current text color via `fill="currentColor"` and
//! carry a `text-*` class so they opt out of the global `svg:not([class*="text-"])`
//! red debug fallback in index.css. Callers pass extra classes (e.g. sizing or a
//! danger color) to keep the set coherent.

use maud::{html, Markup, PreEscaped};

/// Default icon sizing — one place so every icon matches.
const SIZE: &str = "h-5 w-5";

/// Render one icon glyph with the given viewBox and path data. `extra` appends
/// classes (color, spacing). `text-inherit` is always present so the icon
/// follows the surrounding text color and escapes the red SVG fallback.
fn icon(view_box: &str, path: &str, extra: &str) -> Markup {
    let class = format!("{SIZE} text-inherit {extra}");
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox=(view_box)
            class=(class) fill="currentColor" aria-hidden="true" {
            (PreEscaped(format!(r#"<path d="{path}"/>"#)))
        }
    }
}

const PRINT: &str = "M128 0C92.7 0 64 28.7 64 64l0 96 64 0 0-96 226.7 0L384 93.3l0 66.7 64 0 0-66.7c0-17-6.7-33.3-18.7-45.3L400 18.7C388 6.7 371.7 0 354.7 0L128 0zM384 352l0 32 0 64-256 0 0-64 0-16 0-16 256 0zm64 32l32 0c17.7 0 32-14.3 32-32l0-96c0-35.3-28.7-64-64-64L64 192c-35.3 0-64 28.7-64 64l0 96c0 17.7 14.3 32 32 32l32 0 0 64c0 35.3 28.7 64 64 64l256 0c35.3 0 64-28.7 64-64l0-64zm-16-88a24 24 0 1 1 0-48 24 24 0 1 1 0 48z";
const EDIT: &str = "M362.7 19.3L314.3 67.7 444.3 197.7l48.4-48.4c25-25 25-65.5 0-90.5L453.3 19.3c-25-25-65.5-25-90.5 0zm-71 71L58.6 323.5c-10.4 10.4-18 23.3-22.2 37.4L1 481.2C-1.5 489.7 .8 498.8 7 505s15.3 8.5 23.7 6.1l120.3-35.4c14.1-4.2 27-11.8 37.4-22.2L421.7 220.3 291.7 90.3z";
const DUPLICATE: &str = "M208 0L332.1 0c12.7 0 24.9 5.1 33.9 14.1l67.9 67.9c9 9 14.1 21.2 14.1 33.9L448 336c0 26.5-21.5 48-48 48l-192 0c-26.5 0-48-21.5-48-48l0-288c0-26.5 21.5-48 48-48zM48 128l80 0 0 64-64 0 0 256 192 0 0-32 64 0 0 48c0 26.5-21.5 48-48 48L48 512c-26.5 0-48-21.5-48-48L0 176c0-26.5 21.5-48 48-48z";
const DELETE: &str = "M135.2 17.7L128 32 32 32C14.3 32 0 46.3 0 64S14.3 96 32 96l384 0c17.7 0 32-14.3 32-32s-14.3-32-32-32l-96 0-7.2-14.3C305.1 6.8 293.4 0 280.6 0L167.4 0c-12.8 0-24.5 6.8-32.2 17.7zM416 128L32 128 53.2 467c1.6 25.3 22.6 45 47.9 45l245.8 0c25.3 0 46.3-19.7 47.9-45L416 128z";
// List / ingredients-per-serving (bars + dots).
const LIST: &str = "M40 48C26.7 48 16 58.7 16 72l0 48c0 13.3 10.7 24 24 24l48 0c13.3 0 24-10.7 24-24l0-48c0-13.3-10.7-24-24-24L40 48zM192 64c-17.7 0-32 14.3-32 32s14.3 32 32 32l288 0c17.7 0 32-14.3 32-32s-14.3-32-32-32L192 64zm0 160c-17.7 0-32 14.3-32 32s14.3 32 32 32l288 0c17.7 0 32-14.3 32-32s-14.3-32-32-32l-288 0zm0 160c-17.7 0-32 14.3-32 32s14.3 32 32 32l288 0c17.7 0 32-14.3 32-32s-14.3-32-32-32l-288 0zM16 232l0 48c0 13.3 10.7 24 24 24l48 0c13.3 0 24-10.7 24-24l0-48c0-13.3-10.7-24-24-24l-48 0c-13.3 0-24 10.7-24 24zM40 368c-13.3 0-24 10.7-24 24l0 48c0 13.3 10.7 24 24 24l48 0c13.3 0 24-10.7 24-24l0-48c0-13.3-10.7-24-24-24l-48 0z";

/// Ingredients-per-serving (list).
pub fn list() -> Markup {
    icon("0 0 512 512", LIST, "")
}

/// Printer.
pub fn print() -> Markup {
    icon("0 0 512 512", PRINT, "")
}

/// Pencil (edit).
pub fn edit() -> Markup {
    icon("0 0 512 512", EDIT, "")
}

/// Overlapping sheets (duplicate).
pub fn duplicate() -> Markup {
    icon("0 0 448 512", DUPLICATE, "")
}

/// Trash can (delete) — danger tint, hover deepens.
pub fn delete() -> Markup {
    icon(
        "0 0 448 512",
        DELETE,
        "!text-red-500 hover:!text-red-600",
    )
}

// Triangle with exclamation mark (warning).
const WARNING: &str = "M256 32c14.2 0 27.3 7.5 34.5 19.8l216 368c7.3 12.4 7.3 27.7 .2 40.1S486.3 480 472 480L40 480c-14.3 0-27.6-7.7-34.7-20.1s-7-27.8 .2-40.1l216-368C228.7 39.5 241.8 32 256 32zm0 128c-13.3 0-24 10.7-24 24l0 112c0 13.3 10.7 24 24 24s24-10.7 24-24l0-112c0-13.3-10.7-24-24-24zm32 224a32 32 0 1 0 -64 0 32 32 0 1 0 64 0z";

/// Warning triangle. Small and amber by default; pass `extra` to override.
pub fn warning(extra: &str) -> Markup {
    icon("0 0 512 512", WARNING, extra)
}
