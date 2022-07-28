use iced::{button, Background, Color, Vector};

pub enum Button {
    Icon,
    Destructive,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        let theme = crate::theme();
        match self {
            Button::Icon => button::Style {
                text_color: theme.border_color(),
                ..button::Style::default()
            },
            Button::Destructive => button::Style {
                background: Some(Background::Color(theme.warning())),
                border_radius: 5.0,
                text_color: theme.background(),
                shadow_offset: Vector::new(1.0, 1.0),
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            text_color: match self {
                Button::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                _ => active.text_color,
            },
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }
}
