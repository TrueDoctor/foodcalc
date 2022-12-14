use std::sync::Arc;

use iced::widget::*;
use iced::{alignment::Horizontal, Alignment, Command, Element, Length};
use log::debug;
use sqlx::{postgres::types::PgMoney, types::BigDecimal};

use crate::{
    app::{ui::Icon, Error},
    db::{Event, FoodBase, Meal, Place, Recipe},
};

use self::{
    event_meal_wrapper::{MealWrapper, MealWrapperMessage},
    meal_detail_modal::MealDetail,
};
mod meal_detail_modal;
use self::meal_detail_modal::MealDetailMessage;

use super::EventTabMessage;

mod event_meal_wrapper;

#[derive(Debug, Clone)]
pub struct EventDetail {
    pub(crate) event: Event,
    all_recipes: Arc<Vec<Recipe>>,
    all_places: Arc<Vec<Place>>,
    meals: Vec<MealWrapper>,

    meal_modal: Option<MealDetail>,

    database: Arc<FoodBase>,
}

#[derive(Debug, Clone)]
pub enum EventDetailMessage {
    MealWrapperMessage(usize, MealWrapperMessage),
    MealDetailMessage(MealDetailMessage),
    ShowModal(Result<MealDetail, Error>),
    CloseModal(Result<(), Error>),
    UpdateMeals(Result<Vec<Meal>, Error>),
    TitleChange(String),
    CommentChanged(String),
    BudgetChanged(String),
    AddMeal,
    Save,
    Cancel,
}

impl EventDetail {
    pub fn new(
        event: Event,
        database: Arc<FoodBase>,
        meals: Vec<Meal>,
        recipes: Vec<Recipe>,
        places: Vec<Place>,
    ) -> Self {
        let recipes = Arc::new(recipes);
        let places = Arc::new(places);
        Self {
            event,
            database: database.clone(),
            meals: meals
                .into_iter()
                .map(|meal| MealWrapper::new(Some(meal), recipes.clone(), places.clone(), database.clone()))
                .collect(),
            meal_modal: None,
            all_recipes: recipes,
            all_places: places,
        }
    }

    pub fn update(&mut self, message: EventDetailMessage) -> Command<EventTabMessage> {
        match message {
            EventDetailMessage::UpdateMeals(Ok(meals)) => {
                self.meals = meals
                    .into_iter()
                    .map(|meal| {
                        MealWrapper::new(
                            Some(meal),
                            self.all_recipes.clone(),
                            self.all_places.clone(),
                            self.database.clone(),
                        )
                    })
                    .collect()
            },
            EventDetailMessage::MealWrapperMessage(i, MealWrapperMessage::Focus) => {
                for (j, meal) in self.meals.iter_mut().enumerate() {
                    if j != i {
                        meal.update(MealWrapperMessage::Unfocus);
                    }
                }
            },
            EventDetailMessage::MealWrapperMessage(i, MealWrapperMessage::Delete) => {
                let remove = self.meals.remove(i).meal;
                log::trace!("Deleted recipe entry: {:?}", remove);
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        move_database.update_single_meal(remove, None).await?;
                        Ok(())
                    },
                    |_: Result<(), Error>| EventTabMessage::EventDetailMessage(EventDetailMessage::CloseModal(Ok(()))),
                );
            },
            EventDetailMessage::MealWrapperMessage(_, MealWrapperMessage::OpenModal(meal)) => {
                let move_database = self.database.clone();
                let event_id = self.event.event_id;
                return Command::perform(
                    async move {
                        let all_recipes = move_database.get_recipes().await?;
                        let all_places = move_database.get_places().await?;
                        Ok(MealDetail::new(
                            meal,
                            Arc::new(all_recipes),
                            Arc::new(all_places),
                            move_database,
                            event_id,
                        ))
                    },
                    EventDetailMessage::ShowModal,
                )
                .map(|message| EventTabMessage::EventDetailMessage(message.into()));
            },
            EventDetailMessage::ShowModal(Ok(meal_modal)) => {
                self.meal_modal = Some(meal_modal);
            },
            EventDetailMessage::MealDetailMessage(message) => {
                if let Some(meal_detail) = self.meal_modal.as_mut() {
                    return meal_detail.update(message).map(EventTabMessage::EventDetailMessage);
                }
            },
            EventDetailMessage::MealWrapperMessage(i, message) => {
                if let Some(meal) = self.meals.get_mut(i) {
                    return meal.update(message).map(EventTabMessage::EventDetailMessage);
                }
            },
            EventDetailMessage::AddMeal => self.meals.push(MealWrapper::new(
                None,
                self.all_recipes.clone(),
                self.all_places.clone(),
                self.database.clone(),
            )),
            EventDetailMessage::Save => {
                let move_database = self.database.clone();
                let event = self.event.clone();
                return Command::perform(
                    async move {
                        move_database.update_event(&event).await?;
                        Ok(())
                    },
                    EventTabMessage::SaveEvent,
                );
            },

            EventDetailMessage::Cancel => {
                println!("Cancel");
                return Command::perform(async {}, |_| EventTabMessage::CancelButtonPressed);
            },
            EventDetailMessage::CloseModal(Ok(_)) => {
                self.meal_modal = None;
                let event_id = self.event.event_id;
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let meals = move_database.get_event_meals(event_id).await?;
                        Ok(meals)
                    },
                    |result| EventTabMessage::EventDetailMessage(EventDetailMessage::UpdateMeals(result)),
                );
            },
            EventDetailMessage::BudgetChanged(budget) => {
                let budget = if let Some(budget) = BigDecimal::parse_bytes(budget.as_bytes(), 10) {
                    match PgMoney::from_bigdecimal(budget, 2) {
                        Ok(budget) => Some(budget),
                        Err(error) => {
                            debug!("{}", error);
                            None
                        },
                    }
                } else {
                    None
                };
                self.event.budget = budget;
            },
            EventDetailMessage::TitleChange(title) => self.event.event_name = title,
            EventDetailMessage::CommentChanged(comment) => {
                self.event.comment = Some(comment);
            },
            _ => {
                debug!("recieved message without handler: {message:?}")
            },
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<EventDetailMessage> {
        let theme = crate::theme();

        let title = text_input(
            "Event Title...",
            &self.event.event_name,
            EventDetailMessage::TitleChange,
        )
        .width(Length::FillPortion(1))
        .padding(10);

        let comment = self.event.comment.clone().unwrap_or_default();

        let comment = text_input("Comment ...", &comment, EventDetailMessage::CommentChanged)
            .width(Length::FillPortion(2))
            .padding(10);

        let budget = if let Some(budget) = self.event.budget {
            (budget.0 / 100).to_string()
        } else {
            "".to_string()
        };

        let budget = text_input("Budget ...", &budget, EventDetailMessage::BudgetChanged)
            .width(Length::FillPortion(1))
            .padding(10);

        //let title = Text::new(&self.event.event_name).color(theme.foreground()).size(30);

        let meals: Element<'_, EventDetailMessage> = self
            .meals
            .iter_mut()
            .enumerate()
            .fold(Column::new(), |column, (i, meal)| {
                column.push(
                    meal.view()
                        .map(move |message| EventDetailMessage::MealWrapperMessage(i, message)),
                )
            })
            .into();

        let add_meal_button = Button::new(
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(Text::new("Add Meal")),
        )
        .on_press(EventDetailMessage::AddMeal)
        .padding(10)
        .style(iced::theme::Button::Positive);

        let meals = Scrollable::new(
            iced::widget::column![meals, add_meal_button]
                .align_items(Alignment::Start)
                .spacing(20),
        )
        .height(Length::Fill);

        let cancel_button = Button::new(Text::new("Cancel").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
            .on_press(EventDetailMessage::Cancel);

        let ok_button = Button::new(Text::new("Save").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
            .on_press(EventDetailMessage::Save);

        let header = Row::new()
            .spacing(10)
            .padding(5)
            .width(Length::Fill)
            .height(Length::Units(50))
            .push(budget)
            .push(comment);

        let footer = Row::new()
            .spacing(10)
            .padding(5)
            .width(Length::Fill)
            .height(Length::Units(50))
            .push(cancel_button)
            .push(ok_button);

        let element: Element<'_, EventDetailMessage> = Column::new()
            .spacing(20)
            .max_width(800)
            .align_items(Alignment::Center)
            .push(title)
            .push(header)
            .push(meals)
            .push(footer)
            .into();
        match self.meal_modal.as_mut() {
            Some(modal) => modal.view().map(EventDetailMessage::MealDetailMessage),
            None => element,
        }
    }
}
