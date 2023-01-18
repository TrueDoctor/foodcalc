use std::fmt::Display;

use iced::widget::*;
use iced::{Alignment, Element, Length};

use crate::{app::ui::Icon, db::Event};

use super::EventTabMessage;

#[derive(Debug, Clone, Default)]
pub struct EventWrapper {
    pub(crate) event: Event,
    pub price: f64,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.event_name.as_str())
    }
}

impl EventWrapper {
    pub fn new(event: Event, price: f64) -> Self {
        Self { event, price }
    }

    pub(crate) fn view(&self) -> Element<EventTabMessage> {
        let event_id = Text::new(self.event.event_id.to_string());
        let name = Text::new(self.event.event_name.to_string()).width(Length::Fill);
        let price = Text::new(self.price.to_string()).width(Length::Units(50));
        let edit_button = Button::new(Icon::Edit.text())
            .on_press(EventTabMessage::OpenModal(self.event.clone()))
            .style(iced::theme::Button::Text)
            .padding(10);
        let print_button = Button::new(Icon::RestaurantMenu.text())
            .on_press(EventTabMessage::PrintRecipes(self.event.clone()))
            .style(iced::theme::Button::Text)
            .padding(10);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(event_id)
            .push(name)
            .push(price)
            .push(print_button)
            .push(edit_button)
            .into()
    }
}
