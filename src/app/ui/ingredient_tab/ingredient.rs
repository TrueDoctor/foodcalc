use std::fmt::Display;
use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::{button, text_input, Alignment, Button, Command, Element, Length, Row, Text, TextInput};
use num::Zero;

use super::IngredientTabMessage;
use crate::app::ui::{style, Icon};
use crate::app::Error;
use crate::db::{FoodBase, Ingredient};

#[derive(Debug, Clone, Default)]
pub struct IngredientState {
    edit_button: button::State,
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
}

impl IngredientWrapper {
    pub fn new(ingredient: Ingredient) -> Self {
        Self {
            ingredient,
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: IngredientMessage) -> Command<IngredientTabMessage> {
        let move_id = self.ingredient.ingredient_id;
        let command = match message {
            IngredientMessage::Edit => Command::perform(async move { move_id }, IngredientTabMessage::EditIngredient),
        };
        command
    }

    pub fn view(&mut self) -> Element<IngredientMessage> {
        let theme = crate::theme();
        let energy_color = match self.ingredient.energy == sqlx::types::BigDecimal::zero() {
            true => [0.5, 0.5, 0.5].into(),
            false => theme.foreground(),
        };
        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new(self.ingredient.ingredient_id.to_string()).color(theme.foreground()))
            .push(
                Text::new(self.ingredient.name.to_string())
                    .width(Length::Fill)
                    .color(theme.foreground()),
            )
            .push(
                Text::new(
                    self.ingredient
                        .comment
                        .as_ref()
                        .map(|c| format!("({c})"))
                        .unwrap_or_default(),
                )
                .horizontal_alignment(Horizontal::Right)
                .color(theme.foreground())
                .width(Length::Fill),
            )
            .push(
                Text::new(format!("{} kj", self.ingredient.energy.round(2)))
                    .width(Length::Units(100))
                    .color(energy_color)
                    .horizontal_alignment(Horizontal::Right),
            )
            .push(
                Button::new(&mut self.state.edit_button, Icon::Edit.text())
                    .on_press(IngredientMessage::Edit)
                    .padding(10)
                    .style(style::Button::Icon),
            )
            .into()
    }
}
