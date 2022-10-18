use std::fmt::Display;

use iced::{button, Alignment, Button, Element, Length, Row, Text};

use crate::{
    app::ui::{style, Icon},
    db::Event,
};

use super::EventTabMessage;

#[derive(Debug, Clone, Default)]
pub struct EventWrapper {
    pub(crate) event: Event,
    edit_button: button::State,
    print_button: button::State,
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
        let event_id = Text::new(self.event.event_id.to_string()).color(theme.foreground());
        let name = Text::new(self.event.event_name.to_string())
            .width(Length::Fill)
            .color(theme.foreground());
        let edit_button = Button::new(&mut self.edit_button, Icon::Edit.text())
            .on_press(EventTabMessage::OpenModal(self.event.clone()))
            .padding(10)
            .style(style::Button::Icon);
        let print_button = Button::new(&mut self.print_button, Icon::RestaurantMenu.text())
            .on_press(EventTabMessage::PrintRecipes(self.event.clone()))
            .padding(10)
            .style(style::Button::Icon);

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
