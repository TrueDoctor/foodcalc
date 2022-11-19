use iced::alignment::Horizontal;
use iced::{button, Alignment, Button, Column, Command, Element, Length, Row, Text, TextInput};
use log::debug;
use num::FromPrimitive;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::types::BigDecimal;

use super::IngredientTabMessage;
use crate::app::ui::style;
use crate::app::ui::util::InputState;
use crate::db::{FoodBase, Ingredient, IngredientCreate};

#[derive(Debug, Clone)]
pub struct IngredientCreationDialog {
    ingredient: IngredientCreate,
    name: InputState,
    energy: InputState,
    comment: InputState,

    pub(crate) ok_state: button::State,
    pub(crate) cancel_state: button::State,
}

#[derive(Debug, Clone)]
pub enum InputField {
    Name,
    Energy,
    Comment,
}

#[derive(Debug, Clone)]
pub enum IngredientCreateMessage {
    SubmitValue(InputField, String),
    Cancel,
    Save,
}

impl IngredientCreationDialog {
    pub fn new() -> Self {
        Self {
            ingredient: IngredientCreate::default(),
            name: InputState {
                value: String::new(),
                valid: false,
                ..Default::default()
            },
            energy: InputState {
                value: String::from("0"),
                valid: true,
                ..Default::default()
            },
            comment: InputState {
                value: String::new(),
                valid: true,
                ..Default::default()
            },
            ok_state: Default::default(),
            cancel_state: Default::default(),
        }
    }

    pub fn update(&mut self, message: IngredientCreateMessage) -> Option<IngredientTabMessage> {
        match message {
            IngredientCreateMessage::SubmitValue(field, s) => {
                match field {
                    InputField::Name => {
                        // TODO: Check if name is valid
                        self.name.valid = true;
                        self.ingredient.name = s.clone();
                        self.name.value = s
                    },
                    InputField::Energy => {
                        self.energy.value = s;
                        match self.energy.value.trim().parse() {
                            Ok(n) if n > BigDecimal::from_u8(0).unwrap() => {
                                self.energy.valid = true;
                                self.ingredient.energy = n;
                            },
                            _ => {
                                self.energy.valid = false;
                            },
                        }
                    },
                    InputField::Comment => {
                        if s.trim().is_empty() {
                            self.ingredient.comment = None;
                        } else {
                            self.ingredient.comment = Some(s);
                        }
                        self.comment.valid = true;
                    },
                };
            },
            IngredientCreateMessage::Cancel => {
                println!("Cancel");
                return Some(IngredientTabMessage::CloseCreateIngredient);
            },
            IngredientCreateMessage::Save => {
                if vec![&self.comment, &self.energy, &self.name]
                    .iter()
                    .all(|input| input.valid)
                {
                    return Some(IngredientTabMessage::CreateIngredient(self.ingredient.clone()));
                } else {
                    println!("Invalid input {:#?}", self);
                }
            },
        }
        None
    }

    pub fn view(&mut self) -> Element<IngredientCreateMessage> {
        let theme = crate::theme();

        let comment_input = TextInput::new(&mut self.comment.state, "Comment…", &self.comment.value, |value| {
            IngredientCreateMessage::SubmitValue(InputField::Comment, value)
        })
        .width(Length::Fill)
        .style(style::TextInput::Normal)
        .padding(10);

        let text_theme = match self.name.valid {
            true => style::TextInput::Normal,
            false => style::TextInput::Error,
        };

        let name_input = TextInput::new(&mut self.name.state, "Name…", &self.name.value, |value| {
            IngredientCreateMessage::SubmitValue(InputField::Name, value)
        })
        .width(Length::FillPortion(1))
        .style(text_theme)
        .padding(10);

        let text_theme = match self.energy.valid {
            true => style::TextInput::Normal,
            false => style::TextInput::Error,
        };

        let energy_input = TextInput::new(&mut self.energy.state, "Energy…", &self.energy.value, |value| {
            IngredientCreateMessage::SubmitValue(InputField::Energy, value)
        })
        .width(Length::FillPortion(1))
        .style(text_theme)
        .padding(10);

        let cancel_button = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(IngredientCreateMessage::Cancel);

        let ok_button = Button::new(
            &mut self.ok_state,
            Text::new("Save").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(IngredientCreateMessage::Save);

        let buttons = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(ok_button)
            .push(cancel_button);

        Column::new()
            .spacing(20)
            .max_width(800)
            .align_items(Alignment::Center)
            .push(Text::new("Ingredient name:").size(10))
            .push(name_input)
            .push(Text::new("Comment:").size(10))
            .push(comment_input)
            .push(Text::new("Energy in kj/g:").size(10))
            .push(energy_input)
            .push(buttons)
            .into()
    }
}

impl Default for IngredientCreationDialog {
    fn default() -> Self {
        Self::new()
    }
}
