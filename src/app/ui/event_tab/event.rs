use std::fmt::Display;

use iced::{button, Element, Row, Text, Length, Button, Alignment};

use crate::{db::{Event}, app::ui::{style, Icon}};

use super::EventTabMessage;

#[derive(Debug, Clone, Default)]
pub struct EventWrapper {
    pub(crate) event: Event,
    pub(crate) edit_button: button::State,
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
        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(Text::new(self.event.event_id.to_string()).color(theme.foreground()))
            .push(
                Text::new(self.event.event_name.to_string())
                    .width(Length::Fill)
                    .color(theme.foreground()), 
            )
            .push(
                Button::new(&mut self.edit_button, Icon::Edit.text())
                    .on_press(EventTabMessage::OpenModal(self.event.clone()))
                    .padding(10)
                    .style(style::Button::Icon),
            )
            .into()
        
    }
}
