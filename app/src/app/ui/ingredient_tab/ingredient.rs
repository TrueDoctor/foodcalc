use std::fmt::Display;
use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::widget::*;
use iced::{Alignment, Command, Element, Length};
use num::Zero;

use super::IngredientTabMessage;
use crate::app::ui::Icon;
use crate::app::Error;
use crate::db::{FoodBase, Ingredient};

#[derive(Debug, Clone, Default)]
pub struct IngredientState {}

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
            false => theme.palette().primary,
        };
        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(text(self.ingredient.ingredient_id.to_string()))
            .push(text(self.ingredient.name.to_string()).width(Length::Fill))
            .push(
                text(
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
                text(format!("{} kj", self.ingredient.energy.round(2)))
                    .width(Length::Units(100))
                    .style(iced::theme::Text::Color(energy_color))
                    .horizontal_alignment(Horizontal::Right),
            )
            .push(
                button(Icon::Edit.text())
                    .on_press(IngredientMessage::Edit)
                    .padding(10)
                    .style(iced::theme::Button::Primary),
            )
            .into()
    }
}
