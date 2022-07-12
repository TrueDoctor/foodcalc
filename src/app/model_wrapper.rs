use std::fmt::Display;

use iced::{
    button, text_input, Alignment, Application, Button, Checkbox, Column, Command, Container, Element, Font, Length,
    Row, Settings, Text, TextInput,
};

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../../fonts/icons.ttf"),
};

use super::db::Ingredient;

#[derive(Debug, Clone)]
pub enum IngredientState {
    Idle {
        edit_button: button::State,
    },
    Editing {
        text_input: text_input::State,
        delete_button: button::State,
    },
}

impl Default for IngredientState {
    fn default() -> Self {
        IngredientState::Idle {
            edit_button: button::State::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IngredientWrapper {
    pub(crate) ingredient: Ingredient,
    pub(crate) state: IngredientState,
}

impl Display for Ingredient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}
#[derive(Debug, Clone)]
pub enum IngredientMessage {
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

impl IngredientWrapper {
    fn new(ingredient: Ingredient) -> Self {
        Self {
            ingredient,
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: IngredientMessage) {
        match message {
            IngredientMessage::Edit => {
                let mut text_input = text_input::State::focused();
                text_input.select_all();

                self.state = IngredientState::Editing {
                    text_input,
                    delete_button: button::State::new(),
                };
            },
            IngredientMessage::DescriptionEdited(new_description) => {
                self.ingredient.name = new_description;
            },
            IngredientMessage::FinishEdition => {
                if !self.ingredient.name.is_empty() {
                    self.state = IngredientState::Idle {
                        edit_button: button::State::new(),
                    }
                }
            },
            IngredientMessage::Delete => {},
        }
    }

    pub fn view(&mut self) -> Element<IngredientMessage> {
        match &mut self.state {
            IngredientState::Idle { edit_button } => Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(Text::new(self.ingredient.ingredient_id.to_string()))
                .push(Text::new(self.ingredient.name.to_string()))
                .push(
                    Button::new(edit_button, edit_icon())
                        .on_press(IngredientMessage::Edit)
                        .padding(10)
                        .style(style::Button::Icon),
                )
                .into(),
            IngredientState::Editing {
                text_input,
                delete_button,
            } => {
                let text_input = TextInput::new(
                    text_input,
                    "Describe your task...",
                    &self.ingredient.name,
                    IngredientMessage::DescriptionEdited,
                )
                .on_submit(IngredientMessage::FinishEdition)
                .padding(10);

                Row::new()
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(text_input)
                    .push(
                        Button::new(
                            delete_button,
                            Row::new().spacing(10).push(delete_icon()).push(Text::new("Delete")),
                        )
                        .on_press(IngredientMessage::Delete)
                        .padding(10)
                        .style(style::Button::Destructive),
                    )
                    .into()
            },
        }
    }
}

fn icon(unicode: char) -> Text {
    Text::new(unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(iced::alignment::Horizontal::Center)
        .size(20)
}

fn edit_icon() -> Text {
    icon('\u{F303}')
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}
mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        FilterActive,
        FilterSelected,
        Icon,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            match self {
                Button::FilterActive => button::Style::default(),
                Button::FilterSelected => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.7))),
                    border_radius: 10.0,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                },
                Button::Icon => button::Style {
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..button::Style::default()
                },
                Button::Destructive => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                    border_radius: 5.0,
                    text_color: Color::WHITE,
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
                    Button::FilterActive => Color::from_rgb(0.2, 0.2, 0.7),
                    _ => active.text_color,
                },
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            }
        }
    }
}
