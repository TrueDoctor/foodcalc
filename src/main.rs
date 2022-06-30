use std::sync::Arc;

use eyre::Result;
use foodcalc::app::db::{FoodBase, Ingredient, TaskMessage};
use iced::alignment::{self, Alignment};
use iced::button::{self, Button};
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{Application, Checkbox, Column, Command, Container, Element, Font, Length, Row, Settings, Text};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

fn main() -> iced::Result {
    foodcalc::app::FoodCalc::run(Settings::default())
}
