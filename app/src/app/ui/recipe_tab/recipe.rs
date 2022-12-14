use std::fmt::Display;

use iced::widget::*;
use iced::{Alignment, Element, Length};

use super::RecipeTabMessage;
use crate::app::ui::Icon;
use crate::db::Recipe;

#[derive(Debug, Clone, Default)]
pub struct RecipeWrapper {
    pub(crate) recipe: Recipe,
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

impl RecipeWrapper {
    pub fn new(recipe: Recipe) -> Self {
        Self {
            recipe,
            ..Default::default()
        }
    }

    pub fn view(&mut self) -> Element<RecipeTabMessage> {
        let theme = crate::theme();
        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(text(self.recipe.recipe_id.to_string()))
            .push(text(self.recipe.name.to_string()).width(Length::Fill))
            .push(
                button(Icon::Edit.text())
                    .on_press(RecipeTabMessage::OpenModal(self.recipe.clone()))
                    .padding(10)
                    .style(iced::theme::Button::Primary),
            )
            .into()
    }
}
