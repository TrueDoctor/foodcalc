use std::env;
use std::sync::Arc;

use db::{FoodBase, Ingredient};
use iced::alignment::{self, Alignment};
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{Application, Checkbox, Column, Command, Container, Element, Font, Length, Row, Settings, Text};
use log::{debug, error, warn};
use sqlx::postgres::types::PgMoney;
use sqlx::PgPool;

use self::ui::model_wrapper::{IngredientMessage, IngredientWrapper};
use self::ui::{TabBarExample, TabMessage};
pub use crate::db;

#[cfg(feature = "scraping")]
pub mod scraping;

pub mod ui;

static PRICE_PLACEHOLDER: PgMoney = PgMoney(-100i64);

#[derive(Debug)]
pub enum FoodCalc {
    ConnectingToDatabase,
    ErrorView(String),
    MainView(ui::TabBarExample),
    //MealView(MealState),
}

#[derive(Debug, Clone)]
pub enum Message {
    DatebaseConnected(Result<Arc<FoodBase>, Error>),
    MainMessage(TabMessage),
}

#[derive(Debug, Clone)]
pub enum Error {
    Database(String),
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::Database(format!("Database Error occurred {error}"))
    }
}

impl Application for FoodCalc {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let command = Command::perform(
            async move {
                dotenv::dotenv().ok();
                let pool =
                    PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL env var was not set")).await?;
                Ok(Arc::new(FoodBase::new(pool)))
            },
            Message::DatebaseConnected,
        );
        (FoodCalc::ConnectingToDatabase, command)
    }

    fn title(&self) -> String {
        "FoodCalc".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            FoodCalc::ConnectingToDatabase => match message {
                Message::DatebaseConnected(Ok(database)) => {
                    let (main_view, main_command) = ui::TabBarExample::new(database);
                    *self = FoodCalc::MainView(main_view);
                    main_command.map(Message::MainMessage)
                },
                Message::DatebaseConnected(Err(Error::Database(error))) => {
                    *self = FoodCalc::ErrorView(error);
                    Command::none()
                },
                _ => Command::none(),
            },
            FoodCalc::MainView(main_view) => {
                match message {
                    Message::MainMessage(message) => {
                        main_view.update(message);
                    },
                    _ => {
                        debug!("recieved message without handler: {message:?}")
                    },
                }
                Command::none()
            },

            FoodCalc::ErrorView(_) => Command::none(),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        match self {
            FoodCalc::ConnectingToDatabase => empty_message("Connecting To Database"),
            FoodCalc::ErrorView(error) => empty_message(error),
            FoodCalc::MainView(main_view) => main_view.view(),
        }
    }
}

fn empty_message<'a>(message: &str) -> Element<'a, Message> {
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
