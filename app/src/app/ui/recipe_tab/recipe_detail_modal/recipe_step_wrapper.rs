use std::str::FromStr;
use std::sync::Arc;

use chrono::Duration;
use iced::widget::*;
use iced::{Alignment, Element, Length};
use num::Num;
use sqlx::postgres::types::PgInterval;
use sqlx::types::BigDecimal;

use crate::app::ui::util::{DurationInput, InputState};
use crate::app::ui::Icon;
use crate::db::{RecipeIngrdient, RecipeMetaIngredient, RecipeStep, Unit};

#[derive(Debug, Clone, Default)]
pub struct RecipeStepWrapper {
    pub(crate) entry: RecipeStep,
    name: InputState<String>,
    description: InputState<String>,
    fixed_duration: InputState<DurationInput>,
    duration_per_kg: InputState<DurationInput>,
    step_order: InputState<f64>,
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
        let entry = RecipeStep {
            step_order: order,
            step_id: -1,
            ..Default::default()
        };
        Self {
            entry,
            fixed_duration: InputState::new("0 min".to_string()),
            duration_per_kg: InputState::new("0 min".to_string()),
            step_order: InputState::new(order.to_string()),
            ..Default::default()
        }
    }

    pub fn valid(&self) -> bool {
        self.name.valid()
            && self.step_order.valid()
            && self.fixed_duration.valid()
            && self.duration_per_kg.valid()
            && self.description.valid()
    }

    pub fn update(&mut self, message: RecipeStepMessage) -> Option<ReturnMessage> {
        match message {
            RecipeStepMessage::Delete => Some(ReturnMessage::Delete),
            RecipeStepMessage::NameChanged(s) => {
                self.name.update(s);
                if let Some(value) = self.name.value_type.clone() {
                    self.entry.step_name = value;
                }
                None
            },
            RecipeStepMessage::DescriptionChanged(s) => {
                self.description.update(s);
                if let Some(value) = self.description.value_type.clone() {
                    self.entry.step_description = value;
                }
                None
            },
            RecipeStepMessage::FixedDurationChanged(d) => {
                self.fixed_duration.update(d);
                if let Some(value) = self.fixed_duration.value_type.clone() {
                    self.entry.fixed_duration = value.0;
                }
                None
            },
            RecipeStepMessage::DurationPerKgChanged(d) => {
                self.duration_per_kg.update(d);
                if let Some(value) = self.duration_per_kg.value_type.clone() {
                    self.entry.duration_per_kg = value.0;
                }
                None
            },
            RecipeStepMessage::StepOrderChanged(o) => {
                self.step_order.update(o);
                if let Some(value) = self.step_order.value_type {
                    self.entry.step_order = value;
                }
                None
            },
        }
    }

    pub fn view(&mut self) -> Element<RecipeStepMessage> {
        let text_theme = self.name.text_color();
        let name = Row::new()
            .push(Text::new("Name:").size(20))
            .spacing(5)
            .push(
                text_input("Name", &self.name.value, RecipeStepMessage::NameChanged)
                    .size(20)
                    .style(text_theme),
            )
            .width(Length::Fill);

        let text_theme = self.step_order.text_color();
        let step_order = Row::new()
            .push(Text::new("Step Order:").size(20))
            .spacing(5)
            // TODO: find replacement
            //.max_width(100)
            .push(
                text_input(
                    "Step Order",
                    &self.step_order.value,
                    RecipeStepMessage::StepOrderChanged,
                )
                .size(20)
                .style(text_theme),
            )
            .width(Length::Units(200));

        let text_theme = self.description.text_color();
        let description = Column::new().push(Text::new("Description:").size(20)).spacing(5).push(
            text_input(
                "Description",
                &self.description.value,
                RecipeStepMessage::DescriptionChanged,
            )
            .size(20)
            .style(text_theme),
        );
        let text_theme = self.fixed_duration.text_color();
        let fixed_duration = Row::new()
            .push(Text::new("Fixed Duration:").size(20))
            .spacing(5)
            .push(
                text_input(
                    "Fixed Duration",
                    &self.fixed_duration.value,
                    RecipeStepMessage::FixedDurationChanged,
                )
                .size(20)
                .style(text_theme),
            )
            .width(Length::FillPortion(2));

        let text_theme = self.duration_per_kg.text_color();
        let duration_per_kg = Row::new()
            .push(Text::new("Duration per kg:").size(20))
            .spacing(5)
            .push(
                text_input(
                    "Fixed Duration",
                    &self.duration_per_kg.value,
                    RecipeStepMessage::DurationPerKgChanged,
                )
                .size(20)
                .style(text_theme),
            )
            .width(Length::FillPortion(2));

        let delete_button = Button::new(
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(RecipeStepMessage::Delete)
        .padding(10)
        .width(Length::Shrink)
        .style(iced::theme::Button::Destructive);

        let row1 = Row::new().spacing(20).push(name).push(step_order);
        let row2 = Row::new().spacing(20).push(fixed_duration).push(duration_per_kg);

        let fields = Column::new()
            .push(row1)
            .push(description)
            .push(row2)
            .spacing(10)
            .width(Length::FillPortion(8));

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(fields)
            .push(delete_button)
            .into()
    }
}
