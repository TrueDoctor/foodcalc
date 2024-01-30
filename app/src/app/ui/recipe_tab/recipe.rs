use iced::widget::*;
use iced::{Alignment, Element, Length};

use super::RecipeTabMessage;
use crate::app::ui::Icon;
use crate::db::Recipe;

#[derive(Debug, Clone, Default)]
pub struct RecipeWrapper {
    pub(crate) recipe: Recipe,
}

impl RecipeWrapper {
    pub fn new(recipe: Recipe) -> Self {
        Self { recipe }
    }

    pub fn view(&self) -> Element<RecipeTabMessage> {
        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(text(self.recipe.recipe_id.to_string()))
            .push(text(self.recipe.name.to_string()).width(Length::Fill))
            .push(
                button(Icon::Edit.text())
                    .on_press(RecipeTabMessage::OpenModal(self.recipe.clone()))
                    .padding(10)
                    .style(iced::theme::Button::Text),
            )
            .into()
    }
}
