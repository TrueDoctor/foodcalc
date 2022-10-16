use std::sync::Arc;

use iced::{button, Alignment, Button, Command, Element, Row, Text};

use crate::{
    app::ui::{style, Icon},
    db::{Meal, Place, Recipe},
};

use crate::app::ui::style::Button::Destructive;

use super::EventDetailMessage;

#[derive(Debug, Clone)]
pub struct MealWrapper {
    pub(crate) meal: Option<Meal>,
    all_recipes: Arc<Vec<Recipe>>,
    all_places: Arc<Vec<Place>>,
    delete_button: button::State,
    edit_button: button::State,
}

#[derive(Clone, Debug)]
pub enum MealWrapperMessage {
    OpenModal(Option<Meal>),
    Delete,
    Focus,
    Unfocus,
}

impl MealWrapper {
    pub fn new(meal: Option<Meal>) -> Self {
        Self {
            meal: meal.clone(),
            all_recipes: Default::default(),
            all_places: Default::default(),
            delete_button: Default::default(),
            edit_button: Default::default(),
        }
    }

    pub fn update(&mut self, message: MealWrapperMessage) -> Command<EventDetailMessage> {
        match message {
            
            _ => {}
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<MealWrapperMessage> {
        let theme = crate::theme();

        let recipe = iced::Text::new(
            self.all_recipes
                .iter()
                .find(|recipe| recipe.recipe_id == self.meal.clone().unwrap_or_default().recipe_id)
                .cloned()
                .unwrap_or(Default::default())
                .name,
        )
        .width(iced::Length::FillPortion(3))
        .color(theme.foreground());

        let place = iced::Text::new(
            self.all_places
                .iter()
                .find(|place| place.place_id == self.meal.clone().unwrap_or_default().place_id)
                .cloned()
                .unwrap_or(Default::default())
                .name,
        )
        .width(iced::Length::FillPortion(3))
        .color(theme.foreground());

        let start = iced::Text::new(self.meal.clone().unwrap_or_default().start_time.to_string())
            .width(iced::Length::FillPortion(3))
            .color(theme.foreground());

        let delete_button = Button::new(
            &mut self.delete_button,
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(MealWrapperMessage::Delete)
        .padding(10)
        .width(iced::Length::FillPortion(1))
        .style(Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(recipe)
            .push(place)
            .push(start)
            .push(
                Button::new(&mut self.edit_button, Icon::Edit.text())
                    .on_press(MealWrapperMessage::OpenModal(self.meal.clone()))
                    .padding(10)
                    .style(style::Button::Icon),
            )
            .push(delete_button)
            .into()
    }
}
