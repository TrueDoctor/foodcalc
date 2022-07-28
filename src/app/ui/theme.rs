use std::sync::RwLock;

use iced::{Background, Color};
use iced_aw::tabs::{self, Style};

pub struct TabBar;

lazy_static::lazy_static! {
    pub static  ref THEME: RwLock<Theme> = RwLock::new(Theme::default());
}

pub fn theme() -> Theme {
    THEME.read().as_deref().cloned().unwrap_or_default().clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
    pub const RED: Color = Color::from_rgb(0.8, 0.2, 0.2);
    pub const GREEN: Color = Color::from_rgb(0., 1., 0.);
    pub const BLUE: Color = Color::from_rgb(0., 0., 1.);
    pub const YELLOW: Color = Color::from_rgb(1., 1., 0.);
    pub const CYAN: Color = Color::from_rgb(0., 1., 1.);
    pub const MAGENTA: Color = Color::from_rgb(0.7, 0., 1.);
    pub const WHITE: Color = Color::from_rgb(1., 1., 1.);
    pub const BLACK: Color = Color::from_rgb(0., 0., 0.);
    pub const GRAY: Color = Color::from_rgb(0.5, 0.5, 0.5);
    pub const LIGHT_GRAY: Color = Color::from_rgb(0.75, 0.75, 0.75);
    pub const DARK_GRAY: Color = Color::from_rgb(0.25, 0.25, 0.25);
    pub const LIGHT_BLUE: Color = Color::from_rgb(0.75, 0.75, 1.);
    pub const LIGHT_GREEN: Color = Color::from_rgb(0.75, 1., 0.75);
    pub const LIGHT_RED: Color = Color::from_rgb(1., 0.75, 0.75);
    pub const LIGHT_YELLOW: Color = Color::from_rgb(1., 1., 0.75);
    pub const LIGHT_CYAN: Color = Color::from_rgb(0.75, 1., 1.);
    pub const LIGHT_MAGENTA: Color = Color::from_rgb(1., 0.75, 1.);

    pub fn background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(1., 1., 1.),
            Theme::Dark => Color::from_rgb(0.1, 0.1, 0.1),
        }
    }

    pub fn accent_background(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.95, 0.95, 0.95),
            Theme::Dark => Color::from_rgb(0.1, 0.1, 0.1),
        }
    }

    pub fn accent_background_darker(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.90, 0.90, 0.90),
            Theme::Dark => Color::from_rgb(0.15, 0.15, 0.15),
        }
    }

    pub fn foreground(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.1, 0.1, 0.1),
            Theme::Dark => Color::from_rgb(0.9, 0.9, 0.9),
        }
    }

    pub fn accent_light(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.75, 0.75, 0.75),
            Theme::Dark => Color::from_rgb(0.25, 0.25, 0.25),
        }
    }

    pub fn accent_dark(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.25, 0.25, 0.25),
            Theme::Dark => Color::from_rgb(0.75, 0.75, 0.75),
        }
    }

    pub fn warning(&self) -> Color {
        Self::RED
    }

    pub fn border_color(&self) -> Color {
        match self {
            Theme::Light => Color::from_rgb(0.5, 0.5, 0.5),
            Theme::Dark => Color::from_rgb(0.5, 0.5, 0.5),
        }
    }

    pub fn highlight_light(&self) -> Color {
        match self {
            Theme::Light => Theme::LIGHT_MAGENTA,
            Theme::Dark => Theme::LIGHT_CYAN,
        }
    }

    pub fn highlight_dark(&self) -> Color {
        match self {
            Theme::Light => Theme::MAGENTA,
            Theme::Dark => Theme::CYAN,
        }
    }
}

impl From<Theme> for String {
    fn from(theme: Theme) -> Self {
        String::from(match theme {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        })
    }
}

impl iced::widget::text_input::StyleSheet for Theme {
    fn active(&self) -> iced::text_input::Style {
        iced::text_input::Style {
            background: iced::Background::Color(self.background()),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: self.accent_dark(),
        }
    }

    fn focused(&self) -> iced::text_input::Style {
        iced::text_input::Style {
            border_color: self.accent_light(),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        self.accent_light()
    }

    fn value_color(&self) -> Color {
        self.foreground()
    }

    fn selection_color(&self) -> Color {
        self.highlight_light()
    }
}

impl iced::widget::button::StyleSheet for Theme {
    fn active(&self) -> iced::button::Style {
        iced::button::Style {
            shadow_offset: iced::Vector::default(),
            background: None,
            border_radius: 5.0,
            border_width: 1.0,
            border_color: self.border_color(),
            text_color: self.foreground(),
        }
    }
}

impl iced::widget::radio::StyleSheet for Theme {
    fn active(&self) -> iced::radio::Style {
        iced::radio::Style {
            background: self.accent_background().into(),
            dot_color: self.accent_dark(),
            border_width: 1.0,
            border_color: self.border_color(),
            text_color: Some(self.foreground()),
        }
    }

    fn hovered(&self) -> iced::radio::Style {
        iced::radio::Style {
            background: self.accent_background_darker().into(),
            ..self.active()
        }
    }
}

impl iced::widget::pick_list::StyleSheet for Theme {
    fn menu(&self) -> iced::widget::pick_list::Menu {
        iced::widget::pick_list::Menu {
            text_color: self.foreground(),
            background: iced::Background::Color(self.accent_background()),
            border_width: 1.0,
            border_color: self.border_color(),
            selected_text_color: Color::WHITE,
            selected_background: iced::Background::Color(self.highlight_dark()),
        }
    }

    fn active(&self) -> iced_searchable_picklist::Style {
        iced_searchable_picklist::Style {
            text_color: self.foreground(),
            placeholder_color: self.accent_light(),
            background: Background::Color(self.accent_background()),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: self.border_color(),
            icon_size: 0.7,
        }
    }

    fn hovered(&self) -> iced_searchable_picklist::Style {
        iced_searchable_picklist::Style {
            border_color: self.foreground(),
            ..self.active()
        }
    }
}

impl iced_aw::tabs::StyleSheet for Theme {
    fn active(&self, is_active: bool) -> iced_aw::tab_bar::Style {
        let text_color = if is_active {
            self.highlight_dark()
        } else {
            self.foreground()
        };

        Style {
            background: None,
            border_color: None,
            border_width: 0.0,
            tab_label_background: self.background().into(),
            tab_label_border_color: Color::TRANSPARENT,
            tab_label_border_width: 0.0,
            icon_color: text_color,
            text_color,
        }
    }

    fn hovered(&self, is_active: bool) -> iced_aw::tab_bar::Style {
        let text_color = self.highlight_dark();

        Style {
            icon_color: text_color,
            text_color,
            ..self.active(is_active)
        }
    }
}
