use iced::{widget::button, widget::text_input, Background, Color, Vector};

pub enum Button {
    Icon,
    Destructive,
    Add,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Appearance {
        match self {
            Button::Icon => button::Appearance {
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..button::Appearance::default()
            },
            Button::Destructive => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 1.0),
                ..button::Appearance::default()
            },
            Button::Add => button::Appearance {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.8, 0.2))),
                border_radius: 5.0,
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 1.0),
                ..button::Appearance::default()
            },
        }
    }

    fn hovered(&self) -> button::Appearance {
        let active = self.active();

        button::Appearance {
            text_color: match self {
                Button::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                _ => active.text_color,
            },
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    type Style = Button;
}

pub enum TextInput {
    Normal,
    Error,
}

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Appearance {
        let theme: Box<dyn text_input::StyleSheet> = crate::theme().into();
        theme.active()
    }

    fn focused(&self) -> text_input::Appearance {
        let theme: Box<dyn text_input::StyleSheet> = crate::theme().into();
        match self {
            TextInput::Normal => theme.focused(),
            TextInput::Error => text_input::Appearance {
                border_color: Color::from_rgb(0.8, 0.2, 0.2),
                ..theme.focused()
            },
        }
    }

    fn placeholder_color(&self) -> Color {
        let theme: Box<dyn text_input::StyleSheet> = crate::theme().into();
        theme.placeholder_color()
    }

    fn value_color(&self) -> Color {
        let theme: Box<dyn text_input::StyleSheet> = crate::theme().into();
        theme.value_color()
    }

    fn selection_color(&self) -> Color {
        let theme: Box<dyn text_input::StyleSheet> = crate::theme().into();
        theme.selection_color()
    }

    type Style = TextInput;
}
