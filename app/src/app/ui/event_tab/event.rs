

use iced::widget::*;
use iced::{Alignment, Element, Length};
use sqlx::postgres::types::PgMoney;

use crate::{app::ui::Icon, db::Event};

use super::EventTabMessage;

#[derive(Debug, Clone, Default)]
pub struct EventWrapper {
    pub(crate) event: Event,
    pub price: f64,
}

impl EventWrapper {
    pub fn new(event: Event, price: f64) -> Self {
        Self { event, price }
    }

    pub(crate) fn view(&self) -> Element<EventTabMessage> {
        let event_id = Text::new(self.event.event_id.to_string());
        let name = Text::new(self.event.event_name.to_string()).width(Length::Fill);
        let budget = (self.event.budget.unwrap_or(PgMoney(0)).0 as f32) * 0.01;
        let budget = Text::new(format!("{:.2}€", budget)).width(Length::Shrink);
        let budget = Container::new(budget)
            .align_x(iced::alignment::Horizontal::Right)
            .width(Length::Units(75));

        let price = Text::new(format!("{:.2}€", self.price)).width(Length::Shrink);
        let price = Container::new(price)
            .align_x(iced::alignment::Horizontal::Right)
            .width(Length::Units(75));
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
            .push(budget)
            .push(print_button)
            .push(edit_button)
            .into()
    }
}
