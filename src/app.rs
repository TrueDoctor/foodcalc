use std::env;
use std::sync::Arc;

use db::FoodBase;
use iced::alignment::{self};

use iced::{Application, Command, Container, Element, Length, Text};
use log::debug;
use sqlx::PgPool;

use self::ui::TabMessage;
pub use crate::db;

#[cfg(feature = "scraping")]
pub mod scraping;

pub mod ui;

#[derive(Debug)]
pub enum FoodCalc {
    ConnectingToDatabase,
    ErrorView(String),
    MainView(Box<ui::TabBarExample>),
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
    Misc(String),
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::Database(format!("Database Error occurred {error}"))
    }
}
impl From<eyre::ErrReport> for Error {
    fn from(error: eyre::ErrReport) -> Self {
        Error::Misc(format!("Error occurred {error}"))
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

    fn background_color(&self) -> iced::Color {
        crate::theme().background()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            FoodCalc::ConnectingToDatabase => match message {
                Message::DatebaseConnected(Ok(database)) => {
                    let (main_view, main_command) = ui::TabBarExample::new(database);
                    *self = FoodCalc::MainView(main_view.into());
                    main_command.map(Message::MainMessage)
                },
                Message::DatebaseConnected(Err(Error::Database(error))) => {
                    *self = FoodCalc::ErrorView(error);
                    Command::none()
                },
                _ => Command::none(),
            },
            FoodCalc::MainView(main_view) => match message {
                Message::MainMessage(message) => main_view.update(message).map(Message::MainMessage),
                _ => {
                    debug!("recieved message without handler: {message:?}");
                    Command::none()
                },
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
