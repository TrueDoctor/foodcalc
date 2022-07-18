use std::fmt::Display;

use iced::alignment::Horizontal;
use iced::{button, text_input, Alignment, Button, Element, Length, Row, Text, TextInput};
use num::Zero;

use super::style;
use crate::app::ui::Icon;
use crate::db::Ingredient;

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
    pub fn new(ingredient: Ingredient) -> Self {
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
        let energy_color = match self.ingredient.energy == sqlx::types::BigDecimal::zero() {
            true => [0.7, 0.7, 0.7],
            false => [0., 0., 0.],
        };
        match &mut self.state {
            IngredientState::Idle { edit_button } => Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(Text::new(self.ingredient.ingredient_id.to_string()))
                .push(Text::new(self.ingredient.name.to_string()).width(Length::Fill))
                .push(
                    Text::new(
                        self.ingredient
                            .comment
                            .as_ref()
                            .map(|c| format!("({c})"))
                            .unwrap_or_default(),
                    )
                    .horizontal_alignment(Horizontal::Right)
                    .width(Length::Fill),
                )
                .push(
                    Text::new(format!("{} kj", self.ingredient.energy.round(2)))
                        .width(Length::Units(100))
                        .color(energy_color)
                        .horizontal_alignment(Horizontal::Right),
                )
                .push(
                    Button::new(edit_button, Icon::Edit.text())
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
                    "Ingredient Name…",
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
                            Row::new()
                                .spacing(10)
                                .push(Icon::Delete.text())
                                .push(Text::new("Delete")),
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