use std::str::FromStr;
use std::sync::Arc;

use chrono::Duration;
use iced::{button, text_input, Alignment, Button, Element, Length, Row, Text, TextInput};
use num::Num;
use sqlx::postgres::types::PgInterval;
use sqlx::types::BigDecimal;

use crate::app::ui::style::Button::Destructive;
use crate::app::ui::util::{DurationInput, InputState};
use crate::app::ui::{style, Icon};
use crate::db::{RecipeEntry, RecipeMetaIngredient, RecipeStep, Unit};

#[derive(Debug, Clone, Default)]
pub struct RecipeStepWrapper {
    pub(crate) entry: RecipeStep,
    name: InputState<String>,
    description: InputState<String>,
    fixed_duration: InputState<DurationInput>,
    duration_per_kg: InputState<DurationInput>,
    step_order: InputState<f64>,

    delete_button: button::State,
}

#[derive(Debug, Clone)]
pub enum RecipeStepMessage {
    NameChanged(String),
    DescriptionChanged(String),
    FixedDurationChanged(String),
    DurationPerKgChanged(String),
    StepOrderChanged(String),
    Delete,
}
pub enum ReturnMessage {
    Delete,
}

impl RecipeStepWrapper {
    pub fn edit(entry: RecipeStep) -> Self {
        Self {
            entry: entry.clone(),
            name: InputState::new(entry.step_name),
            description: InputState::new(entry.step_description),
            fixed_duration: InputState::new(DurationInput(entry.fixed_duration)),
            duration_per_kg: InputState::new(DurationInput(entry.duration_per_kg)),
            step_order: InputState::new(entry.step_order.to_string()),
            ..Default::default()
        }
    }

    pub fn create(order: f64) -> Self {
        let mut entry = RecipeStep::default();
        entry.step_order = order;
        entry.step_id = -1;
        Self {
            entry,
            fixed_duration: InputState::new("00:00:00".to_string()),
            duration_per_kg: InputState::new("00:00:00".to_string()),
            step_order: InputState::new(order.to_string()),
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: RecipeStepMessage) -> Option<ReturnMessage> {
        match message {
            RecipeStepMessage::Delete => Some(ReturnMessage::Delete),
            RecipeStepMessage::NameChanged(s) => {
                self.name.update(s);
                None
            },
            RecipeStepMessage::DescriptionChanged(s) => {
                self.description.update(s);
                None
            },
            RecipeStepMessage::FixedDurationChanged(d) => {
                self.fixed_duration.update(d);
                None
            },
            RecipeStepMessage::DurationPerKgChanged(d) => {
                self.duration_per_kg.update(d);
                None
            },
            RecipeStepMessage::StepOrderChanged(o) => {
                self.step_order.update(o);
                None
            },
        }
    }

    pub fn view(&mut self) -> Element<RecipeStepMessage> {
        let theme = crate::theme();

        fn render_input<'a, T: FromStr>(
            input: &'a mut InputState<T>,
            label: &'static str,
            message: impl Fn(String) -> RecipeStepMessage + 'a,
        ) -> Element<'a, RecipeStepMessage> {
            let text_theme = match input.valid() {
                true => style::TextInput::Normal,
                false => style::TextInput::Error,
            };

            Row::new()
                .push(Text::new(format!("{label}:")).size(14))
                .push(
                    TextInput::new(&mut input.state, label, "", message)
                        .size(14)
                        .style(text_theme)
                        .width(Length::Fill),
                )
                .into()
        }
        let name = render_input(&mut self.name, "Name", RecipeStepMessage::NameChanged);
        let description = render_input(
            &mut self.description,
            "Description",
            RecipeStepMessage::DescriptionChanged,
        );
        let fixed_duration = render_input(
            &mut self.fixed_duration,
            "Fixed Duration",
            RecipeStepMessage::FixedDurationChanged,
        );
        let duration_per_kg = render_input(
            &mut self.duration_per_kg,
            "Duration per kg",
            RecipeStepMessage::DurationPerKgChanged,
        );
        let step_order = render_input(&mut self.step_order, "Step Order", RecipeStepMessage::StepOrderChanged);

        let delete_button = Button::new(
            &mut self.delete_button,
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(RecipeStepMessage::Delete)
        .padding(10)
        .style(Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(name)
            .push(description)
            .push(fixed_duration)
            .push(duration_per_kg)
            .push(step_order)
            .push(delete_button)
            .into()
    }
}
