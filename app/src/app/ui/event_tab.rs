use std::sync::Arc;

use iced::widget::*;
use iced::{alignment, Command, Element, Length};
use log::debug;

use self::event_detail_modal::EventDetailMessage;
use super::{Icon, TabMessage};
use crate::app::Error;
use crate::db::{Event, FoodBase};

mod event;
pub use event::EventWrapper;

mod event_detail_modal;
pub use event_detail_modal::EventDetail;

#[derive(Debug, Clone)]
pub struct EventTab {
    event_list: Vec<EventWrapper>,
    input_value: String,
    database: Arc<FoodBase>,
    event_detail_modal: Option<EventDetail>,
}

#[derive(Debug, Clone)]
pub enum EventTabMessage {
    UpdateData(Result<Vec<EventWrapper>, Error>),
    EventDetailMessage(EventDetailMessage),
    InputChanged(String),
    OpenModal(Event),
    ShowModal(Result<EventDetail, Error>),
    CancelButtonPressed,
    CloseModal,
    SaveEvent(Result<(), Error>),
    AddEvent(Result<Event, Error>),
    PrintRecipes(Event),
    Nothing,
    NewEvent,
}

async fn load_events_with_price(database: Arc<FoodBase>) -> Result<Vec<EventWrapper>, Error> {
    let events = database.get_events().await?;
    let mut event_list = Vec::new();
    for event in events {
        let price = database.get_event_cost(event.event_id).await?.0;
        event_list.push(EventWrapper::new(event, (price as f64) / 100.0));
    }
    Ok(event_list)
}

fn load_events(database: Arc<FoodBase>) -> Command<EventTabMessage> {
    Command::perform(
        async move {
            let events = load_events_with_price(database).await?;
            Ok(events)
        },
        EventTabMessage::UpdateData,
    )
}

impl EventTab {
    pub fn new(database: Arc<FoodBase>) -> (Self, Command<TabMessage>) {
        let move_database = database.clone();
        let command = load_events(move_database);

        let events = EventTab {
            event_list: Vec::new(),
            input_value: Default::default(),
            database,
            event_detail_modal: None,
        };
        (events, command.map(|message| TabMessage::EventTab(message.into())))
    }

    pub(crate) fn update(&mut self, message: EventTabMessage) -> Command<TabMessage> {
        match message {
            EventTabMessage::UpdateData(Ok(events)) => {
                self.event_list = events;
            },
            EventTabMessage::UpdateData(Err(error)) => {
                log::error!("{error:?}");
            },
            EventTabMessage::InputChanged(input) => {
                self.input_value = input;
            },
            EventTabMessage::OpenModal(event) => {
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let meals = move_database.get_event_meals(event.event_id).await?;
                        let recipes = move_database.get_recipes().await?;
                        let places = move_database.get_places().await?;
                        Ok(EventDetail::new(event, move_database.clone(), meals, recipes, places))
                    },
                    EventTabMessage::ShowModal,
                )
                .map(|message| TabMessage::EventTab(message.into()));
            },
            EventTabMessage::SaveEvent(_) => {
                return load_events(self.database.clone()).map(|message| TabMessage::EventTab(message.into()));
            },
            EventTabMessage::ShowModal(Ok(event_detail)) => {
                self.event_detail_modal = Some(event_detail);
            },
            EventTabMessage::CancelButtonPressed => {
                println!("Cancel");
                self.event_detail_modal = None;
            },
            EventTabMessage::CloseModal => {
                self.event_detail_modal = None;
            },
            EventTabMessage::EventDetailMessage(message) => {
                if let Some(modal) = self.event_detail_modal.as_mut() {
                    return modal
                        .update(message)
                        .map(|message| TabMessage::EventTab(message.into()));
                }
            },
            EventTabMessage::NewEvent => {
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let event = move_database.add_empty_event().await?;
                        Ok(event)
                    },
                    |event| TabMessage::EventTab(EventTabMessage::AddEvent(event).into()),
                );
            },
            EventTabMessage::AddEvent(Ok(event)) => self.event_list.push(EventWrapper::new(event, 0.0)),
            EventTabMessage::PrintRecipes(event) => {
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let meals = move_database.get_event_meals(event.event_id).await?;
                        let futures = meals
                            .into_iter()
                            .map(|meal| move_database.save_recipe_export(meal.recipe_id, meal.weight));
                        futures::future::join_all(futures).await;
                        Ok(())
                    },
                    |_: Result<(), Error>| TabMessage::EventTab(EventTabMessage::Nothing.into()),
                );
            },
            _ => debug!("recieved event tab message without handler: {message:?}"),
        }
        Command::none()
    }
}

impl super::Tab for EventTab {
    type Message = TabMessage;

    fn title(&self) -> String {
        "Events".to_string()
    }

    fn content(&self) -> iced::Element<'_, Self::Message> {
        let _theme = crate::theme();

        let input = TextInput::new("Event Name", &self.input_value, EventTabMessage::InputChanged)
            .padding(15)
            .size(30);

        let filtered_events = self
            .event_list
            .iter()
            .filter(|event| crate::similar(&event.event.event_name, &self.input_value));

        let add_event_button = Button::new(
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(Text::new("Add Event")),
        )
        .on_press(EventTabMessage::NewEvent)
        .padding(10)
        .style(iced::theme::Button::Positive);

        let events: Element<_> = if filtered_events.clone().count() > 0 {
            self.event_list
                .iter()
                .enumerate()
                .filter(|(_, event)| crate::similar(&event.event.event_name, &self.input_value))
                .fold(Column::new().spacing(00), |column, (_i, event)| {
                    column.push(event.view())
                })
                .push(Row::new().push(add_event_button))
                .into()
        } else {
            empty_message("No matching event ...")
        };

        let scroll: Element<'_, EventTabMessage> = Scrollable::new(Container::new(events).width(Length::Fill)).into();

        let element: Element<'_, EventTabMessage> =
            Column::new().max_width(800).spacing(20).push(input).push(scroll).into();

        let element: Element<'_, EventTabMessage> = Container::new(element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into();

        let element: Element<'_, EventTabMessage> = match self.event_detail_modal.as_ref() {
            Some(modal) => modal.view().map(EventTabMessage::EventDetailMessage),
            None => element,
        };

        element.map(|message| TabMessage::EventTab(message.into()))
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        super::TabLabel::IconText(super::Icon::Calendar.into(), self.title())
    }
}

fn empty_message(message: &str) -> Element<'_, EventTabMessage> {
    Container::new(
        Text::new(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}
