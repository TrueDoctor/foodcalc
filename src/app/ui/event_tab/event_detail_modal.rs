use std::sync::Arc;

use iced::{alignment::Horizontal, button, Alignment, Button, Column, Command, Element, Length, Row, Scrollable, Text};
use log::debug;

use crate::{
    app::ui::{style, Icon},
    db::{Event, FoodBase, Meal, Recipe},
};

use self::event_meal_wrapper::{MealMessage, MealWrapper};

use super::EventTabMessage;

mod event_meal_wrapper;

#[derive(Debug, Clone)]
pub struct EventDetail {
    pub(crate) event: Event,
    database: Arc<FoodBase>,
    pub(crate) all_recipes: Arc<Vec<Recipe>>,
    pub(crate) meals: Vec<MealWrapper>,
    pub(crate) scroll: iced::scrollable::State,
    pub(crate) cancel_state: button::State,
    pub(crate) ok_state: button::State,
    pub(crate) add_meal_button: button::State,
}

#[derive(Debug, Clone)]
pub enum EventDetailMessage {
    MealMessage(usize, MealMessage),
    AddMeal,
    Save,
    Cancel,
}

impl EventDetail {
    pub fn new(event: Event, database: Arc<FoodBase>, recipes: Arc<Vec<Recipe>>, meals: Vec<Meal>) -> Self {
        Self {
            event: event,
            database: database,
            all_recipes: recipes.clone(),
            meals: meals
                .into_iter()
                .map(|meal| MealWrapper::new(meal, recipes.clone()))
                .collect(),
            scroll: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
            add_meal_button: Default::default(),
        }
    }

    pub fn update(&mut self, message: EventDetailMessage) -> Command<EventTabMessage> {
        match message {
            EventDetailMessage::MealMessage(i, MealMessage::Focus) => {
                for (j, ingredient) in self.meals.iter_mut().enumerate() {
                    if j != i {
                        ingredient.update(MealMessage::Unfocus);
                    }
                }
            },
            EventDetailMessage::MealMessage(i, MealMessage::Delete) => {
                log::trace!("Deleted recipe entry: {:?}", self.meals.remove(i).meal);
            },
            EventDetailMessage::MealMessage(i, message) => {
                if let Some(meal) = self.meals.get_mut(i) {
                    meal.update(message);
                }
            },
            EventDetailMessage::AddMeal => self
                .meals
                .push(MealWrapper::new(Meal::default(), self.all_recipes.clone())),
            EventDetailMessage::Save => {
                let move_database = self.database.clone();
                let event = self.event.clone();
                let meals : Vec<_> = self
                    .meals
                    .iter()
                    .map(|meal_wrapper| meal_wrapper.meal.clone())
                    .collect();

                return Command::perform(
                    async move {
                        move_database.update_event(&event).await?;
                        move_database
                            .update_event_meals(&event, meals.into_iter())
                            .await?;
                        Ok(())
                    }, 
                    EventTabMessage::SaveEvent,
                )
            },
            EventDetailMessage::Cancel => {
                println!("Cancel");
                return Command::perform(async {}, |_| EventTabMessage::CancelButtonPressed);
            },
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<EventDetailMessage> {
        let theme = crate::theme();

        let title = Text::new(&self.event.event_name).color(theme.foreground()).size(30);

        let meals: Element<'_, EventDetailMessage> = self
            .meals
            .iter_mut()
            .enumerate()
            .fold(Column::new(), |column, (i, meal)| {
                column.push(
                    meal.view()
                        .map(move |message| EventDetailMessage::MealMessage(i, message)),
                )
            })
            .into();

        let add_meal_button = Button::new(
            &mut self.add_meal_button,
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(Text::new("Add Meal")),
        )
        .on_press(EventDetailMessage::AddMeal)
        .padding(10)
        .style(style::Button::Add);

        let meals = Scrollable::new(&mut self.scroll)
            .push(meals)
            .push(add_meal_button)
            .align_items(Alignment::Start)
            .spacing(20)
            .height(Length::Fill);

        let cancel_button = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(EventDetailMessage::Cancel);

        let ok_button = Button::new(
            &mut self.ok_state,
            Text::new("Save").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(EventDetailMessage::Save);

        let footer = Row::new()
            .spacing(10)
            .padding(5)
            .width(Length::Fill)
            .height(Length::Units(50))
            .push(cancel_button)
            .push(ok_button);

        Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(title)
            .push(meals)
            .push(footer)
            .into()
    }
}
