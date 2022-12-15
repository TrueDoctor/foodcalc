use iced::alignment::Horizontal;
use iced::widget::*;
use iced::{Alignment, Command, Element, Length};
use sqlx::types::BigDecimal;

use super::IngredientTabMessage;
use crate::app::ui::util::{InputState, OptionString};
use crate::db::{FoodBase, Ingredient, IngredientCreate};

#[derive(Debug, Clone)]
pub struct IngredientCreationDialog {
    ingredient: IngredientCreate,
    name: InputState<String>,
    energy: InputState<BigDecimal>,
    comment: InputState<OptionString>,
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
    pub fn create() -> Self {
        Self {
            ingredient: IngredientCreate::default(),
            name: InputState::new(""),
            energy: InputState::new("0"),
            comment: InputState::new(""),
        }
    }
    pub fn edit(ingredient: Ingredient) -> Self {
        let comment = ingredient.comment.clone().unwrap_or_default();
        let energy = ingredient.energy.to_string();
        let name = ingredient.name.clone();
        Self {
            ingredient: ingredient.into(),
            name: InputState::new(name),
            energy: InputState::new(energy),
            comment: InputState::new(comment),
        }
    }

    pub fn update(&mut self, message: IngredientCreateMessage) -> Option<IngredientTabMessage> {
        match message {
            IngredientCreateMessage::SubmitValue(field, string) => {
                let input = string.trim();
                match field {
                    InputField::Name => self.name.update(input),
                    InputField::Energy => self.energy.update(input),
                    InputField::Comment => self.comment.update(input),
                };
            },
            IngredientCreateMessage::Cancel => {
                println!("Cancel");
                return Some(IngredientTabMessage::CloseCreateIngredient);
            },
            IngredientCreateMessage::Save => {
                if vec![&self.comment.valid(), &self.energy.valid(), &self.name.valid()]
                    .iter()
                    .all(|input| **input)
                {
                    return Some(IngredientTabMessage::UpdateIngredient(self.ingredient.clone()));
                } else {
                    println!("Invalid input {:#?}", self);
                }
            },
        }
        None
    }

    pub fn view(&self) -> Element<IngredientCreateMessage> {
        let comment_input = TextInput::new("Comment…", &self.comment.value, |value| {
            IngredientCreateMessage::SubmitValue(InputField::Comment, value)
        })
        .width(Length::Fill)
        .padding(10);

        let name_input = TextInput::new("Name…", &self.name.value, |value| {
            IngredientCreateMessage::SubmitValue(InputField::Name, value)
        })
        .padding(10);

        let energy_input = TextInput::new("Energy…", &self.energy.value, |value| {
            IngredientCreateMessage::SubmitValue(InputField::Energy, value)
        })
        .width(Length::FillPortion(1))
        .padding(10);

        let cancel_button = button(text("Cancel").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
            .style(iced::theme::Button::Destructive)
            .on_press(IngredientCreateMessage::Cancel);

        let ok_button = button(text("Save").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
            .style(iced::theme::Button::Primary)
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
            .push(text("Ingredient name:").size(10))
            .push(name_input)
            .push(text("Comment:").size(10))
            .push(comment_input)
            .push(text("Energy in kj/g:").size(10))
            .push(energy_input)
            .push(buttons)
            .into()
    }
}

impl Default for IngredientCreationDialog {
    fn default() -> Self {
        Self::create()
    }
}
