use std::sync::Arc;

use iced::scrollable::{self, Scrollable};
use iced::{text_input, Command, TextInput, Element, Column, Container, Length, alignment, Text};
use log::debug;
use crate::app::Error;
use crate::db::{FoodBase, Event};

use self::event_detail_modal::EventDetailMessage;

use super::TabMessage;

mod event;
pub use event::EventWrapper;

mod event_detail_modal;
pub use event_detail_modal::EventDetail;

#[derive(Debug, Clone)]
pub struct EventTab {
    event_list: Vec<EventWrapper>,
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    database: Arc<FoodBase>,
    event_detail_modal: Option<EventDetail>
}

#[derive(Debug, Clone)]
pub enum EventTabMessage {
    UpdateData(Result<Vec<EventWrapper>, Error>),
    EventDetailMessage(EventDetailMessage),
    InputChanged(String),
    OpenModal(Event),
    ShowModal(Result<EventDetail, Error>)

}

impl EventTab {
    pub fn new(database: Arc<FoodBase>) -> (Self, Command<TabMessage>) {
        let move_database = database.clone();
        let command = Command::perform(
            async move {
                let events = move_database
                    .get_events()
                    .await?
                    .into_iter()
                    .map(EventWrapper::new)
                    .collect();
                Ok(events)
            },
            EventTabMessage::UpdateData,
        );

        let events = EventTab {
            event_list: Vec::new(),
            scroll: scrollable::State::default(),
            input: text_input::State::default(),
            input_value: String::new(),
            database: database,
            event_detail_modal: None
        };
        (events, command.map(|message| TabMessage::EventTab(message.into())))
    }

    pub(crate) fn update(&mut self, message: EventTabMessage) -> Command<TabMessage> {
        match message {
            EventTabMessage::UpdateData(Ok(events)) => {
                self.event_list = events;
            },
            _ => debug!("recieved message without handler: {message:?}"),
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
            EventTabMessage::InputChanged
        )
        .padding(15)
        .style(theme)
        .size(30);

        let filtered_events = self
            .event_list
            .iter()
            .filter(|event| crate::similar(&event.event.event_name,&*self.input_value));

        let events: Element<_> = if filtered_events.clone().count()>0 {
            self.event_list
                .iter_mut()
                .enumerate()
                .filter(|(_, event)| crate::similar(&event.event.event_name, &self.input_value))
                .fold(Column::new().spacing(00), |column, (_i, event)| {
                    column.push(event.view())
                })
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
        super::TabLabel::IconText(super::Icon::Burger.into(), self.title())
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
