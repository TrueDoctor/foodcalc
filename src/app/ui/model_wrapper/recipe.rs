use std::fmt::Display;

use iced::{button, text_input, Alignment, Button, Element, Length, Row, Text, TextInput};

use super::style;
use crate::app::ui::Icon;
use crate::db::Recipe;

#[derive(Debug, Clone)]
pub enum RecipeState {
    Idle {
        edit_button: button::State,
    },
    Editing {
        text_input: text_input::State,
        delete_button: button::State,
    },
}

impl Default for RecipeState {
    fn default() -> Self {
        RecipeState::Idle {
            edit_button: button::State::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecipeWrapper {
    pub(crate) recipe: Recipe,
    pub(crate) state: RecipeState,
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum RecipeMessage {
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

impl RecipeWrapper {
    pub fn new(recipe: Recipe) -> Self {
        Self {
            recipe,
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: RecipeMessage) {
        match message {
            RecipeMessage::Edit => {
                let mut text_input = text_input::State::focused();
                text_input.select_all();

                self.state = RecipeState::Editing {
                    text_input,
                    delete_button: button::State::new(),
                };
            },
            RecipeMessage::DescriptionEdited(new_description) => {
                self.recipe.name = new_description;
            },
            RecipeMessage::FinishEdition => {
                if !self.recipe.name.is_empty() {
                    self.state = RecipeState::Idle {
                        edit_button: button::State::new(),
                    }
                }
            },
            RecipeMessage::Delete => {},
        }
    }

    pub fn view(&mut self) -> Element<RecipeMessage> {
        match &mut self.state {
            RecipeState::Idle { edit_button } => Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(Text::new(self.recipe.recipe_id.to_string()))
                .push(Text::new(self.recipe.name.to_string()).width(Length::Fill))
                .push(
                    Button::new(edit_button, Icon::Edit.text())
                        .on_press(RecipeMessage::Edit)
                        .padding(10)
                        .style(style::Button::Icon),
                )
                .into(),
            RecipeState::Editing {
                text_input,
                delete_button,
            } => {
                let text_input = TextInput::new(
                    text_input,
                    "Recipe Name…",
                    &self.recipe.name,
                    RecipeMessage::DescriptionEdited,
                )
                .on_submit(RecipeMessage::FinishEdition)
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
                        .on_press(RecipeMessage::Delete)
                        .padding(10)
                        .style(style::Button::Destructive),
                    )
                    .into()
            },
        }
    }
}
