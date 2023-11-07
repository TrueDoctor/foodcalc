use std::cell::RefCell;
use std::env;
use std::sync::Arc;

use fern::colors::{Color, ColoredLevelConfig};
use foodlib::FoodBase;
use iced::widget::{button, column, container, scrollable, text};
use iced::{Application, Command, Element, Length};

use self::ui::TabMessage;

//#[cfg(feature = "scraping")]
//pub mod scraping;

pub mod ui;

pub struct FoodCalc {
    state: FoodCalcState,
    errors: RefCell<Vec<String>>,
    receiver: std::sync::mpsc::Receiver<String>,
}

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
    type Theme = iced::theme::Theme;

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
                let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");
                Ok(Arc::new(FoodBase::new(database_url).await?))
            },
            Message::DatebaseConnected,
        );
        let state = FoodCalcState::ConnectingToDatabase;
        (
            FoodCalc {
                state,
                errors: RefCell::new(vec![]),
                receiver,
            },
            command,
        )
    }

    fn title(&self) -> String {
        "FoodCalc".to_string()
    }

    fn theme(&self) -> Self::Theme {
        ui::theme::theme()
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
                    self.errors.borrow_mut().clear();
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

    fn view(&self) -> Element<'_, Self::Message> {
        let mut errors = self.errors.borrow_mut();
        self.receiver.try_iter().for_each(|message| {
            if message.contains("[31mERROR") && !message.contains("egl") {
                errors.push(message.splitn(2, ' ').last().unwrap().to_string());
            }
        });
        let main_window = match self.state {
            FoodCalcState::ConnectingToDatabase => empty_message("Connecting To Database"),
            FoodCalcState::ErrorView(ref error) => empty_message(error),
            FoodCalcState::MainView(ref main_view) => main_view.view(),
        };
        if !errors.is_empty() {
            let error_view = column![text("Errors:").size(40)];
            let view = errors.iter().fold(error_view, |view, error| {
                view.push(
                    text(error)
                        .size(20)
                        .style(iced::theme::Text::Color(iced::color!(255, 0, 0))),
                )
            });
            let view = column![view, button("Ok").on_press(Message::ErrorClosed)];
            let view = scrollable(view);

            view.into()
        } else {
            main_window
        }
    }
}

fn empty_message<'a>(message: &str) -> Element<'a, Message> {
    container(
        text(message)
            .width(Length::Fill)
            .size(25)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}
