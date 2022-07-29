use std::sync::RwLock;

use iced::Color;
use iced_aw::tabs::Style;

pub struct TabBar;
pub use style::Theme;

lazy_static::lazy_static! {
    pub static  ref THEME: RwLock<Theme> = RwLock::new(Theme::default());
}

pub fn theme() -> Theme {
    THEME.read().as_deref().cloned().unwrap_or_default().clone()
}

impl From<style::Theme> for String {
    fn from(theme: style::Theme) -> Self {
        String::from(match theme {
            style::Theme::Light => "Light",
            style::Theme::Dark => "Dark",
        })
    }
}

mod style {
    use iced::{
        button, checkbox, container, progress_bar, radio, rule, scrollable, slider, text_input, toggler, Color,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Theme {
        #[default]
        Light,
        Dark,
    }

    impl Theme {
        pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];

        pub fn background(&self) -> Color {
            match self {
                Theme::Light => Color::WHITE,
                Theme::Dark => dark::SURFACE,
            }
        }

        pub fn foreground(&self) -> Color {
            match self {
                Theme::Light => Color::BLACK,
                Theme::Dark => Color::WHITE,
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn container::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Container.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn radio::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Radio.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn text_input::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::TextInput.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn button::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => light::Button.into(),
                Theme::Dark => dark::Button.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn scrollable::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Scrollable.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn slider::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Slider.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn progress_bar::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::ProgressBar.into(),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn checkbox::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Checkbox.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn toggler::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Toggler.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn rule::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::Rule.into(),
            }
        }
    }

    impl iced_aw::tabs::StyleSheet for Theme {
        fn active(&self, is_active: bool) -> iced_aw::tabs::Style {
            match self {
                Theme::Light => light::Tabs.active(is_active),
                Theme::Dark => dark::Tabs.active(is_active),
            }
        }

        fn hovered(&self, is_active: bool) -> iced_aw::tabs::Style {
            match self {
                Theme::Light => light::Tabs.hovered(is_active),
                Theme::Dark => dark::Tabs.hovered(is_active),
            }
        }
    }

    impl<'a> From<Theme> for Box<dyn iced_searchable_picklist::StyleSheet + 'a> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => Default::default(),
                Theme::Dark => dark::PickList.into(),
            }
        }
    }

    mod light {
        use iced::{button, Color, Vector};

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Color::from_rgb(0.11, 0.42, 0.87).into(),
                    border_radius: 12.0,
                    shadow_offset: Vector::new(1.0, 1.0),
                    text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    text_color: Color::WHITE,
                    shadow_offset: Vector::new(1.0, 2.0),
                    ..self.active()
                }
            }
        }

        pub struct Tabs;

        impl iced_aw::tabs::StyleSheet for Tabs {
            fn active(&self, is_active: bool) -> iced_aw::tabs::Style {
                let text_color = if is_active {
                    Color::from_rgb(0.11, 0.42, 0.87)
                } else {
                    Color::BLACK
                };

                iced_aw::tabs::Style {
                    background: None,
                    border_color: None,
                    border_width: 0.0,
                    tab_label_background: Color::WHITE.into(),
                    tab_label_border_color: Color::TRANSPARENT,
                    tab_label_border_width: 0.0,
                    icon_color: text_color,
                    text_color,
                }
            }

            fn hovered(&self, is_active: bool) -> iced_aw::tab_bar::Style {
                let text_color = Color::from_rgb(0.11, 0.42, 0.87);

                iced_aw::tabs::Style {
                    icon_color: text_color,
                    text_color,
                    ..self.active(is_active)
                }
            }
        }
    }

    mod dark {
        use iced::{
            button, checkbox, container, progress_bar, radio, rule, scrollable, slider, text_input, toggler, Color,
        };

        pub const SURFACE: Color = Color::from_rgb(0x40 as f32 / 255.0, 0x44 as f32 / 255.0, 0x4B as f32 / 255.0);

        pub const ACCENT: Color = Color::from_rgb(0x6F as f32 / 255.0, 0xFF as f32 / 255.0, 0xE9 as f32 / 255.0);

        pub const ACTIVE: Color = Color::from_rgb(0x72 as f32 / 255.0, 0x89 as f32 / 255.0, 0xDA as f32 / 255.0);

        pub const HOVERED: Color = Color::from_rgb(0x67 as f32 / 255.0, 0x7B as f32 / 255.0, 0xC4 as f32 / 255.0);

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Color::from_rgb8(0x36, 0x39, 0x3F).into(),
                    text_color: Color::WHITE.into(),
                    ..container::Style::default()
                }
            }
        }

        pub struct Radio;

        impl radio::StyleSheet for Radio {
            fn active(&self) -> radio::Style {
                radio::Style {
                    background: SURFACE.into(),
                    dot_color: ACTIVE,
                    border_width: 1.0,
                    border_color: ACTIVE,
                    text_color: None,
                }
            }

            fn hovered(&self) -> radio::Style {
                radio::Style {
                    background: Color { a: 0.5, ..SURFACE }.into(),
                    ..self.active()
                }
            }
        }

        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: SURFACE.into(),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: Color::from_rgb(0.4, 0.4, 0.4),
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1.0,
                    border_color: ACCENT,
                    ..self.active()
                }
            }

            fn hovered(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1.0,
                    border_color: Color { a: 0.3, ..ACCENT },
                    ..self.focused()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.4, 0.4, 0.4)
            }

            fn value_color(&self) -> Color {
                Color::WHITE
            }

            fn selection_color(&self) -> Color {
                ACTIVE
            }
        }

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: ACTIVE.into(),
                    border_radius: 3.0,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: HOVERED.into(),
                    text_color: Color::WHITE,
                    ..self.active()
                }
            }

            fn pressed(&self) -> button::Style {
                button::Style {
                    border_width: 1.0,
                    border_color: Color::WHITE,
                    ..self.hovered()
                }
            }
        }

        pub struct Scrollable;

        impl scrollable::StyleSheet for Scrollable {
            fn active(&self) -> scrollable::Scrollbar {
                scrollable::Scrollbar {
                    background: SURFACE.into(),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    scroller: scrollable::Scroller {
                        color: ACTIVE,
                        border_radius: 2.0,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> scrollable::Scrollbar {
                let active = self.active();

                scrollable::Scrollbar {
                    background: Color { a: 0.5, ..SURFACE }.into(),
                    scroller: scrollable::Scroller {
                        color: HOVERED,
                        ..active.scroller
                    },
                    ..active
                }
            }

            fn dragging(&self) -> scrollable::Scrollbar {
                let hovered = self.hovered();

                scrollable::Scrollbar {
                    scroller: scrollable::Scroller {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..hovered.scroller
                    },
                    ..hovered
                }
            }
        }

        pub struct Slider;

        impl slider::StyleSheet for Slider {
            fn active(&self) -> slider::Style {
                slider::Style {
                    rail_colors: (ACTIVE, Color { a: 0.1, ..ACTIVE }),
                    handle: slider::Handle {
                        shape: slider::HandleShape::Circle { radius: 9.0 },
                        color: ACTIVE,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                }
            }

            fn hovered(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: HOVERED,
                        ..active.handle
                    },
                    ..active
                }
            }

            fn dragging(&self) -> slider::Style {
                let active = self.active();

                slider::Style {
                    handle: slider::Handle {
                        color: Color::from_rgb(0.85, 0.85, 0.85),
                        ..active.handle
                    },
                    ..active
                }
            }
        }

        pub struct ProgressBar;

        impl progress_bar::StyleSheet for ProgressBar {
            fn style(&self) -> progress_bar::Style {
                progress_bar::Style {
                    background: SURFACE.into(),
                    bar: ACTIVE.into(),
                    border_radius: 10.0,
                }
            }
        }

        pub struct Checkbox;

        impl checkbox::StyleSheet for Checkbox {
            fn active(&self, is_checked: bool) -> checkbox::Style {
                checkbox::Style {
                    background: if is_checked { ACTIVE } else { SURFACE }.into(),
                    checkmark_color: Color::WHITE,
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: ACTIVE,
                    text_color: None,
                }
            }

            fn hovered(&self, is_checked: bool) -> checkbox::Style {
                checkbox::Style {
                    background: Color {
                        a: 0.8,
                        ..if is_checked { ACTIVE } else { SURFACE }
                    }
                    .into(),
                    ..self.active(is_checked)
                }
            }
        }

        pub struct Toggler;

        impl toggler::StyleSheet for Toggler {
            fn active(&self, is_active: bool) -> toggler::Style {
                toggler::Style {
                    background: if is_active { ACTIVE } else { SURFACE },
                    background_border: None,
                    foreground: if is_active { Color::WHITE } else { ACTIVE },
                    foreground_border: None,
                }
            }

            fn hovered(&self, is_active: bool) -> toggler::Style {
                toggler::Style {
                    background: if is_active { ACTIVE } else { SURFACE },
                    background_border: None,
                    foreground: if is_active {
                        Color { a: 0.5, ..Color::WHITE }
                    } else {
                        Color { a: 0.5, ..ACTIVE }
                    },
                    foreground_border: None,
                }
            }
        }

        pub struct Rule;

        impl rule::StyleSheet for Rule {
            fn style(&self) -> rule::Style {
                rule::Style {
                    color: SURFACE,
                    width: 2,
                    radius: 1.0,
                    fill_mode: rule::FillMode::Padded(15),
                }
            }
        }

        pub struct Tabs;

        impl iced_aw::tabs::StyleSheet for Tabs {
            fn active(&self, is_active: bool) -> iced_aw::tabs::Style {
                let text_color = if is_active { ACCENT } else { Color::WHITE };

                iced_aw::tabs::Style {
                    background: None,
                    border_color: None,
                    border_width: 0.0,
                    tab_label_background: SURFACE.into(),
                    tab_label_border_color: Color::TRANSPARENT,
                    tab_label_border_width: 0.0,
                    icon_color: text_color,
                    text_color,
                }
            }

            fn hovered(&self, is_active: bool) -> iced_aw::tab_bar::Style {
                let text_color = ACCENT;

                iced_aw::tabs::Style {
                    icon_color: text_color,
                    text_color,
                    ..self.active(is_active)
                }
            }
        }

        pub struct PickList;

        impl iced::widget::pick_list::StyleSheet for PickList {
            fn menu(&self) -> iced::widget::pick_list::Menu {
                iced::widget::pick_list::Menu {
                    text_color: Color::WHITE,
                    background: iced::Background::Color(SURFACE),
                    border_width: 1.0,
                    border_color: Color::from_rgb(0.4, 0.4, 0.4),
                    selected_text_color: Color::WHITE,
                    selected_background: iced::Background::Color(ACTIVE),
                }
            }

            fn active(&self) -> iced_searchable_picklist::Style {
                use iced::text_input::StyleSheet;
                iced_searchable_picklist::Style {
                    text_color: Color::WHITE,
                    placeholder_color: TextInput.placeholder_color(),
                    background: SURFACE.into(),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: TextInput.placeholder_color(),
                    icon_size: 0.7,
                }
            }

            fn hovered(&self) -> iced_searchable_picklist::Style {
                iced_searchable_picklist::Style {
                    border_color: HOVERED,
                    ..self.active()
                }
            }
        }
    }
}
