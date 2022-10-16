use std::sync::Arc;

use iced::{alignment::Horizontal, button, Alignment, Button, Column, Command, Element, Length, Row, Scrollable, Text};
use log::debug;

use crate::{
    app::{
        ui::{style, Icon},
        Error,
    },
    db::{Event, FoodBase, Meal},
};

use self::{
    event_meal_wrapper::{MealMessage, MealWrapper},
    meal_detail_modal::MealDetail,
};
mod meal_detail_modal;
use self::meal_detail_modal::MealDetailMessage;

use super::EventTabMessage;

mod event_meal_wrapper;

#[derive(Debug, Clone)]
pub struct EventDetail {
    pub(crate) event: Event,
    database: Arc<FoodBase>,
    pub(crate) meals: Vec<MealWrapper>,
    pub(crate) scroll: iced::scrollable::State,
    pub(crate) cancel_state: button::State,
    pub(crate) ok_state: button::State,
    pub(crate) add_meal_button: button::State,
    pub(crate) meal_modal: Option<MealDetail>,
}

#[derive(Debug, Clone)]
pub enum EventDetailMessage {
    MealMessage(usize, MealMessage),
    MealDetailMessage(MealDetailMessage),
    ShowModal(Result<MealDetail, Error>),
    AddMeal,
    Save,
    SaveMeal(Meal),
    Cancel,
    CancelButtonPressed,
}

impl EventDetail {
    pub fn new(event: Event, database: Arc<FoodBase>, meals: Vec<Meal>) -> Self {
        Self {
            event: event,
            database: database.clone(),
            meals: meals.into_iter().map(|meal| MealWrapper::new(meal)).collect(),
            scroll: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
            add_meal_button: Default::default(),
            meal_modal: None,
        }
    }

    pub fn update(&mut self, message: EventDetailMessage) -> Command<EventTabMessage> {
        match message {
            EventDetailMessage::MealMessage(i, MealMessage::Focus) => {
                for (j, meal) in self.meals.iter_mut().enumerate() {
                    if j != i {
                        meal.update(MealMessage::Unfocus);
                    }
                }
            },
            EventDetailMessage::MealMessage(i, MealMessage::Delete) => {
                log::trace!("Deleted recipe entry: {:?}", self.meals.remove(i).meal);
            },
            EventDetailMessage::MealMessage(_, MealMessage::OpenModal(meal)) => {
                let move_database = self.database.clone();
                debug!(
                    "Opening modal for {}:{} @ {}, {}",
                    meal.event_id, meal.recipe_id, meal.place_id, meal.start_time
                );
                return Command::perform(
                    async move {
                        let all_recipes = move_database.get_recipes().await?;
                        let all_places = move_database.get_places().await?;
                        Ok(MealDetail::new(meal, Arc::new(all_recipes), Arc::new(all_places)))
                    },
                    EventDetailMessage::ShowModal,
                )
                .map(|message| EventTabMessage::EventDetailMessage(message.into()));
            },
            EventDetailMessage::ShowModal(Ok(meal_modal)) => {
                self.meal_modal = Some(meal_modal);
            },
            EventDetailMessage::MealDetailMessage(message) =>{
                if let Some(meal_detail) = self.meal_modal.as_mut() {
                    meal_detail.update(message);
                }
            },
            EventDetailMessage::MealMessage(i, message) => {
                if let Some(meal) = self.meals.get_mut(i) {
                    meal.update(message);
                }
            },
            EventDetailMessage::AddMeal => self.meals.push(MealWrapper::new(Meal::default())),
            EventDetailMessage::Save => {
                let move_database = self.database.clone();
                let event = self.event.clone();
                let meals: Vec<_> = self
                    .meals
                    .iter()
                    .map(|meal_wrapper| meal_wrapper.meal.clone())
                    .collect();

                return Command::perform(
                    async move {
                        move_database.update_event(&event).await?;
                        move_database.update_event_meals(&event, meals.into_iter()).await?;
                        Ok(())
                    },
                    EventTabMessage::SaveEvent,
                );
            },
            EventDetailMessage::SaveMeal(meal) => {
                let move_database = self.database.clone();
                let event = self.event.clone();
                let meals: Vec<_> = self
                    .meals
                    .iter()
                    .map(|meal_wrapper| meal_wrapper.meal.clone())
                    .collect();

                return Command::perform(
                    async move {
                        move_database.update_event_meals(&event, meals.into_iter()).await?;
                        Ok(())
                    },
                    EventTabMessage::SaveEvent,
                );
            },
            EventDetailMessage::Cancel => {
                println!("Cancel");
                return Command::perform(async {}, |_| EventTabMessage::CancelButtonPressed);
            },
            _ => {
                debug!("recieved message without handler: {message:?}")
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

        let element: Element<'_, EventDetailMessage> = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(title)
            .push(meals)
            .push(footer)
            .into();
        match self.meal_modal.as_mut() {
            Some(modal) => modal.view().map(EventDetailMessage::MealDetailMessage),
            None => element,
        }
    }
}
