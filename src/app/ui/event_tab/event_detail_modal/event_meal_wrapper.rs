use std::sync::Arc;

use iced::{button, Alignment, Button, Command, Element, Row, Text};

use crate::{
    app::ui::{style, Icon},
    db::{Meal, Place, Recipe, FoodBase},
};

use crate::app::ui::style::Button::Destructive;

use super::EventDetailMessage;

#[derive(Debug, Clone)]
pub struct MealWrapper {
    pub(crate) meal: Option<Meal>,
    all_recipes: Arc<Vec<Recipe>>,
    all_places: Arc<Vec<Place>>,
    print_button: button::State,
    delete_button: button::State,
    edit_button: button::State,
    database: Arc<FoodBase>
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
    pub fn new(meal: Option<Meal>, all_recipes: Arc<Vec<Recipe>>, all_places: Arc<Vec<Place>>, database: Arc<FoodBase>) -> Self {
        Self {
            meal: meal.clone(),
            all_recipes,
            all_places,
            print_button: Default::default(),
            delete_button: Default::default(),
            edit_button: Default::default(),
            database
        }
    }

    pub fn update(&mut self, message: MealWrapperMessage) -> Command<EventDetailMessage> {
        match message {
            MealWrapperMessage::PrintMeal(Some(meal)) => {
                let move_database = self.database.clone();
                return Command::perform(async move {
                    move_database.fetch_subrecipes_export(meal.recipe_id, meal.weight).await;
                }, |_| EventDetailMessage::MealWrapperMessage(0, MealWrapperMessage::Nothing));
            }
            _ => {}
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<MealWrapperMessage> {
        let theme = crate::theme();

        let label = self.all_recipes
        .iter()
        .find(|recipe| recipe.recipe_id == self.meal.clone().unwrap_or_default().recipe_id)
        .cloned()
        .unwrap_or_default()
        .name;

        let recipe = iced::Text::new(
            label,
        )
        .width(iced::Length::FillPortion(3))
        .color(theme.foreground());

        let label = self.all_places
        .iter()
        .find(|place| place.place_id == self.meal.clone().unwrap_or_default().place_id)
        .cloned()
        .unwrap_or_default()
        .name;

        let place = iced::Text::new(
            label
        )
        .width(iced::Length::FillPortion(3))
        .color(theme.foreground());


        let start = iced::Text::new(self.meal.clone().unwrap_or_default().start_time.to_string())
            .width(iced::Length::FillPortion(3))
            .color(theme.foreground());

        let print_button = Button::new(&mut self.print_button, Icon::RestaurantMenu.text())
        .on_press(MealWrapperMessage::PrintMeal(self.meal.clone()))
        .padding(10)
        .style(style::Button::Icon);

        let delete_button = Button::new(
            &mut self.delete_button,
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(MealWrapperMessage::Delete)
        .padding(10)
        .style(Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(recipe)
            .push(place)
            .push(start)
            .push(print_button)
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
