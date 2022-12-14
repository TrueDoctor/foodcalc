use std::fmt::Display;

use iced::widget::*;
use iced::{Alignment, Element, Length};

use crate::{app::ui::Icon, db::Event};

use super::EventTabMessage;

#[derive(Debug, Clone, Default)]
pub struct EventWrapper {
    pub(crate) event: Event,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.event_name.as_str())
    }
}

impl EventWrapper {
    pub fn new(event: Event) -> Self {
        Self {
            event,
            ..Default::default()
        }
    }

    pub(crate) fn view(&mut self) -> Element<EventTabMessage> {
        let theme = crate::theme();
        let event_id = Text::new(self.event.event_id.to_string());
        let name = Text::new(self.event.event_name.to_string()).width(Length::Fill);
        let edit_button = Button::new(Icon::Edit.text())
            .on_press(EventTabMessage::OpenModal(self.event.clone()))
            .padding(10);
        let print_button = Button::new(Icon::RestaurantMenu.text())
            .on_press(EventTabMessage::PrintRecipes(self.event.clone()))
            .padding(10);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(event_id)
            .push(name)
            .push(print_button)
            .push(edit_button)
            .into()
    }
}
