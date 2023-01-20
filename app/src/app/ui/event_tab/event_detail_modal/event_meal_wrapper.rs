use std::sync::Arc;

use iced::widget::*;
use iced::{Alignment, Command, Element};

use crate::{
    app::ui::Icon,
    db::{FoodBase, Meal, Place, Recipe},
};

use super::EventDetailMessage;

#[derive(Debug, Clone)]
pub struct MealWrapper {
    pub(crate) meal: Option<Meal>,
    all_recipes: Arc<Vec<Recipe>>,
    all_places: Arc<Vec<Place>>,
    database: Arc<FoodBase>,
}

#[derive(Clone, Debug)]
pub enum MealWrapperMessage {
    OpenModal(Option<Meal>),
    PrintMeal(Option<Meal>),
    Delete,
    Focus,
    Nothing,
    Unfocus,
}

impl MealWrapper {
    pub fn new(
        meal: Option<Meal>,
        all_recipes: Arc<Vec<Recipe>>,
        all_places: Arc<Vec<Place>>,
        database: Arc<FoodBase>,
    ) -> Self {
        Self {
            meal,
            all_recipes,
            all_places,
            database,
        }
    }

    pub fn update(&mut self, message: MealWrapperMessage) -> Command<EventDetailMessage> {
        if let MealWrapperMessage::PrintMeal(Some(meal)) = message {
            let move_database = self.database.clone();
            return Command::perform(
                async move {
                    move_database.fetch_subrecipes_export(meal.recipe_id, meal.weight).await.unwrap_or_else(|e| log::error!("{e}"));
                },
                |_| EventDetailMessage::MealWrapperMessage(0, MealWrapperMessage::Nothing),
            );
        }
        Command::none()
    }

    pub fn view(&self) -> Element<MealWrapperMessage> {
        let label = self
            .all_recipes
            .iter()
            .find(|recipe| recipe.recipe_id == self.meal.clone().unwrap_or_default().recipe_id)
            .cloned()
            .unwrap_or_default()
            .name;

        let recipe = text(label).width(iced::Length::FillPortion(3));

        let label = self
            .all_places
            .iter()
            .find(|place| place.place_id == self.meal.clone().unwrap_or_default().place_id)
            .cloned()
            .unwrap_or_default()
            .name;

        let place = text(label).width(iced::Length::FillPortion(3));

        let start =
            text(self.meal.clone().unwrap_or_default().start_time.to_string()).width(iced::Length::FillPortion(3));

        let print_button = Button::new(Icon::RestaurantMenu.text())
            .on_press(MealWrapperMessage::PrintMeal(self.meal.clone()))
            .padding(10);

        let delete_button = Button::new(
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(MealWrapperMessage::Delete)
        .padding(10)
        .style(iced::theme::Button::Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(recipe)
            .push(place)
            .push(start)
            .push(print_button)
            .push(
                Button::new(Icon::Edit.text())
                    .on_press(MealWrapperMessage::OpenModal(self.meal.clone()))
                    .padding(10),
            )
            .push(delete_button)
            .into()
    }
}
