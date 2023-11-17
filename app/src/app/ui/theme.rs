use std::sync::RwLock;

pub struct TabBar;
use iced::theme::Theme;

lazy_static::lazy_static! {
    pub static  ref THEME: RwLock<Theme> = RwLock::new(Theme::default());
}

pub fn theme() -> Theme {
    THEME.read().as_deref().cloned().unwrap_or_default()
}

pub fn theme_to_string(theme: Theme) -> String {
    String::from(match theme {
        Theme::Light => "Light",
        Theme::Dark => "Dark",
        Theme::Custom(_) => "Custom",
    })
}
