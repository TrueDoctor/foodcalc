use std::env;
use std::sync::Arc;

use db::FoodBase;
use fern::colors::{Color, ColoredLevelConfig};
use iced::alignment::{self};
use iced::{Application, Column, Command, Container, Element, Length, Text};
use log::debug;
use sqlx::PgPool;

use self::ui::TabMessage;
pub use crate::db;

#[cfg(feature = "scraping")]
pub mod scraping;

pub mod ui;

#[derive(Debug)]
pub struct FoodCalc {
    state: FoodCalcState,
    errors: Vec<String>,
    receiver: std::sync::mpsc::Receiver<String>,
    ok_state: iced::button::State,
}

#[derive(Debug)]
pub enum FoodCalcState {
    ConnectingToDatabase,
    ErrorView(String),
    MainView(Box<ui::TabBarExample>),
    //MealView(MealState),
}

#[derive(Debug, Clone)]
pub enum Message {
    DatebaseConnected(Result<Arc<FoodBase>, Error>),
    MainMessage(TabMessage),
    ErrorClosed,
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
        let colors = ColoredLevelConfig::new()
            .debug(Color::Magenta)
            .info(Color::Green)
            .error(Color::Red);

        let (sender, receiver) = std::sync::mpsc::channel();
        fern::Dispatch::new()
            .chain(std::io::stdout())
            .chain(sender)
            .level_for("foodcalc", log::LevelFilter::Trace)
            .level_for("sqlx", log::LevelFilter::Trace)
            .level_for("iced", log::LevelFilter::Trace)
            .level(log::LevelFilter::Warn)
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "[{}]{} {}",
                    // This will color the log level only, not the whole line. Just a touch.
                    colors.color(record.level()),
                    chrono::Utc::now().format("[%Y-%m-%d %H:%M:%S]"),
                    message
                ))
            })
            .apply()
            .unwrap();
        let command = Command::perform(
            async move {
                dotenv::dotenv().ok();
                let pool =
                    PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL env var was not set")).await?;
                Ok(Arc::new(FoodBase::new(pool)))
            },
            Message::DatebaseConnected,
        );
        let state = FoodCalcState::ConnectingToDatabase;
        (
            FoodCalc {
                state,
                errors: vec![],
                receiver,
                ok_state: iced::button::State::new(),
            },
            command,
        )
    }

    fn title(&self) -> String {
        "FoodCalc".to_string()
    }

    fn background_color(&self) -> iced::Color {
        crate::theme().background()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match &mut self.state {
            FoodCalcState::ConnectingToDatabase => match message {
                Message::DatebaseConnected(Ok(database)) => {
                    let (main_view, main_command) = ui::TabBarExample::new(database);
                    self.state = FoodCalcState::MainView(main_view.into());
                    main_command.map(Message::MainMessage)
                },
                Message::DatebaseConnected(Err(Error::Database(error))) => {
                    self.state = FoodCalcState::ErrorView(error);
                    Command::none()
                },
                _ => Command::none(),
            },
            FoodCalcState::MainView(main_view) => match message {
                Message::ErrorClosed => {
                    self.errors.clear();
                    Command::none()
                },
                Message::MainMessage(message) => main_view.update(message).map(Message::MainMessage),
                _ => {
                    log::error!("recieved message without handler: {message:?}");
                    Command::none()
                },
            },

            FoodCalcState::ErrorView(_) => Command::none(),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let theme = crate::theme();
        self.receiver.try_iter().for_each(|message| {
            if message.contains("[31mERROR") && !message.contains("egl") {
                self.errors.push(message);
            }
        });
        if !self.errors.is_empty() {
            let error_view: Column<Self::Message> = Column::new().push(Text::new("Errors:"));
            let view: Column<Self::Message> = self
                .errors
                .iter()
                .fold(error_view, |view, error| view.push(Text::new(error).size(20)));
            let view = view.push(
                iced::Button::new(&mut self.ok_state, Text::new("Ok"))
                    .on_press(Message::ErrorClosed)
                    .style(theme),
            );
            return view.into();
        }
        match &mut self.state {
            FoodCalcState::ConnectingToDatabase => empty_message("Connecting To Database"),
            FoodCalcState::ErrorView(error) => empty_message(error),
            FoodCalcState::MainView(main_view) => main_view.view(),
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
