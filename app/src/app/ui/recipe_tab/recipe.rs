use std::fmt::Display;

use iced::{button, Alignment, Button, Element, Length, Row, Text};

use super::RecipeTabMessage;
use crate::app::ui::style;
use crate::app::ui::Icon;
use crate::db::Recipe;

#[derive(Debug, Clone, Default)]
pub struct RecipeWrapper {
    pub(crate) recipe: Recipe,
    pub(crate) edit_button: button::State,
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
            .push(Text::new(self.recipe.recipe_id.to_string()).color(theme.foreground()))
            .push(
                Text::new(self.recipe.name.to_string())
                    .width(Length::Fill)
                    .color(theme.foreground()),
            )
            .push(
                Button::new(&mut self.edit_button, Icon::Edit.text())
                    .on_press(RecipeTabMessage::OpenModal(self.recipe.clone()))
                    .padding(10)
                    .style(style::Button::Icon),
            )
            .into()
    }
}
