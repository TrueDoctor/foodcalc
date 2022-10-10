use std::sync::Arc;

use iced::{Command, Alignment, Text, Element, Column};
use log::debug;

use crate::db::{Event, FoodBase};

use super::EventTabMessage;


#[derive(Debug, Clone)]
pub struct EventDetail {
    pub(crate) event: Event,
    database: Arc<FoodBase>,
}

#[derive(Debug,Clone)]
pub enum EventDetailMessage {

}


impl EventDetail {
    fn new(event: Event, database: Arc<FoodBase>) -> Self {
        Self {
            event,
            database
        }
    }

    pub fn update(&mut self, message: EventDetailMessage) -> Command<EventTabMessage> {
        match message {
            _ => debug!("recieved message without handler: {message:?}"),
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<EventDetailMessage> {
        let theme = crate::theme();
        
        let title = Text::new(&self.event.event_name).color(theme.foreground()).size(30);

        Column::new()
        .spacing(20)
        .align_items(Alignment::Center)
        .push(title)
        .into()
    }
}
