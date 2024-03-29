extern crate eyre;
pub mod app;
use foodlib as db;

pub fn similar(a: &str, b: &str) -> bool {
    a.to_lowercase().contains(&b.to_lowercase())
}

pub use app::ui::theme::theme;
