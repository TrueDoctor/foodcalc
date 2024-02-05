use iced::alignment::Horizontal;
use iced::widget::*;
use iced::{Alignment, Command, Element, Length};
use num::Zero;

use super::IngredientTabMessage;
use crate::app::ui::Icon;

use foodlib::Ingredient;

#[derive(Debug, Clone, Default)]
pub struct IngredientWrapper {
    pub(crate) ingredient: Ingredient,
}

#[derive(Debug, Clone)]
pub enum IngredientMessage {
    Edit,
}

impl IngredientWrapper {
    pub fn new(ingredient: Ingredient) -> Self {
        Self { ingredient }
    }

    pub fn update(&mut self, message: IngredientMessage) -> Command<IngredientTabMessage> {
        let move_id = self.ingredient.ingredient_id;

        match message {
            IngredientMessage::Edit => Command::perform(async move { move_id }, IngredientTabMessage::EditIngredient),
        }
    }

    pub fn view(&self) -> Element<IngredientMessage> {
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
                    .style(iced::theme::Button::Text),
            )
            .into()
    }
}
