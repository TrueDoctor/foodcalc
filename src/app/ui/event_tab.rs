use std::sync::Arc;

use crate::app::Error;
use crate::db::{Event, FoodBase};
use iced::scrollable::{self, Scrollable};
use iced::{alignment, text_input, Button, Column, Command, Container, Element, Length, Row, Text, TextInput};
use log::debug;

use self::event_detail_modal::EventDetailMessage;

use super::{style, Icon, TabMessage};

mod event;
pub use event::EventWrapper;

mod event_detail_modal;
pub use event_detail_modal::EventDetail;

#[derive(Debug, Clone)]
pub struct EventTab {
    event_list: Vec<EventWrapper>,
    scroll: scrollable::State,
    input: text_input::State,
    add_event_button: iced::button::State,
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
    AddEvent(Result<Event,Error>),
    PrintRecipes(Event),
    Nothing,
    NewEvent,
}

fn load_events(database: Arc<FoodBase>) -> Command<EventTabMessage> {
    Command::perform(
        async move {
            let events = database
                .get_events()
                .await?
                .into_iter()
                .map(|event| EventWrapper::new(event))
                .collect();
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
            scroll: Default::default(),
            input: Default::default(),
            add_event_button: Default::default(),
            input_value: Default::default(),
            database: database,
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
                    |event|TabMessage::EventTab(EventTabMessage::AddEvent(event).into()),
                );
            },
            EventTabMessage::AddEvent(Ok(event)) => self.event_list.push(EventWrapper::new(event)),
            EventTabMessage::PrintRecipes(event) => {
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let meals = move_database.get_event_meals(event.event_id).await?;
                        for meal in meals {
                            move_database.fetch_subrecipes_export(meal.recipe_id, meal.weight).await;
                        }
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

    fn content(&mut self) -> iced::Element<'_, Self::Message> {
        let theme = crate::theme();

        let input = TextInput::new(
            &mut self.input,
            "Event Name",
            &self.input_value,
            EventTabMessage::InputChanged,
        )
        .padding(15)
        .style(theme)
        .size(30);

        let filtered_events = self
            .event_list
            .iter()
            .filter(|event| crate::similar(&event.event.event_name, &*self.input_value));

        let add_event_button = Button::new(
            &mut self.add_event_button,
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(Text::new("Add Event")),
        )
        .on_press(EventTabMessage::NewEvent)
        .padding(10)
        .style(style::Button::Add);

        let events: Element<_> = if filtered_events.clone().count() > 0 {
            self.event_list
                .iter_mut()
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

        let scroll: Element<'_, EventTabMessage> = Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(Container::new(events).width(Length::Fill))
            .into();

        let element: Element<'_, EventTabMessage> =
            Column::new().max_width(800).spacing(20).push(input).push(scroll).into();

        let element: Element<'_, EventTabMessage> = Container::new(element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into();

        let element: Element<'_, EventTabMessage> = match self.event_detail_modal.as_mut() {
            Some(modal) => modal.view().map(EventTabMessage::EventDetailMessage),
            None => element,
        };

        element.map(|message| TabMessage::EventTab(message.into()))
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        super::TabLabel::IconText(super::Icon::Calendar.into(), self.title())
    }
}

fn empty_message<'a>(message: &str) -> Element<'a, EventTabMessage> {
    Container::new(
        Text::new(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .color([0.7, 0.7, 0.7]),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}
