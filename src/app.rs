use std::env;

use db::{FoodBase, Ingredient};
use iced::alignment::{self, Alignment};
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{Application, Checkbox, Column, Command, Container, Element, Font, Length, Row, Settings, Text};
use log::{debug, error, warn};
use once_cell::sync::OnceCell;
use sqlx::postgres::types::PgMoney;
use sqlx::PgPool;

use self::model_wrapper::{IngredientMessage, IngredientWrapper};

pub mod db;
pub mod model_wrapper;
#[cfg(feature = "scraping")]
pub mod scraping;
//pub mod state;

static PRICE_PLACEHOLDER: PgMoney = PgMoney(-100i64);

pub static DATABASE: OnceCell<db::FoodBase> = OnceCell::new();

pub fn database() -> &'static db::FoodBase {
    DATABASE.get().unwrap()
}

#[derive(Debug)]
pub enum FoodCalc {
    ConnectingToDatabase,
    AppInitialized,
    ErrorView(String),
    IngredientView(IngredientsState),
    //MealView(MealState),
}

#[derive(Debug, Default)]
pub struct IngredientsState {
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    //filter: Filter,
    ingredients: Vec<IngredientWrapper>,
    //controls: Controls,
    dirty: bool,
    saving: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    DatebaseConnected(Result<(), Error>),
    Loaded(Option<Vec<IngredientWrapper>>),
    Saved(Option<()>),
    InputChanged(String),
    CreateIngredient,
    //FilterChanged(Filter),
    IngredientMessage(usize, IngredientMessage),
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
                DATABASE.set(FoodBase::new(pool)).unwrap();
                Ok(())
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
                Message::DatebaseConnected(Ok(_)) => {
                    *self = FoodCalc::AppInitialized;
                    Command::perform(
                        async {
                            database()
                                .get_ingredients_option()
                                .await
                                .map(|x| x.into_iter().map(IngredientWrapper::new).collect())
                        },
                        Message::Loaded,
                    )
                },
                Message::DatebaseConnected(Err(Error::Database(error))) => {
                    *self = FoodCalc::ErrorView(error);
                    Command::none()
                },
                _ => Command::none(),
            },
            FoodCalc::AppInitialized => {
                match message {
                    Message::Loaded(Some(ingredients)) => {
                        *self = FoodCalc::IngredientView(IngredientsState {
                            input_value: String::new(),
                            //filter: state.filter,
                            ingredients,
                            ..IngredientsState::default()
                        });
                    },
                    Message::Loaded(None) => {
                        *self = FoodCalc::IngredientView(IngredientsState::default());
                    },
                    _ => {},
                }

                Command::none()
            },
            FoodCalc::IngredientView(IngredientsState {
                input_value,
                ingredients,
                ..
            }) => {
                match message {
                    Message::InputChanged(input) => {
                        *input_value = input;
                    },
                    Message::IngredientMessage(i, message) => {
                        if let Some(ingredient) = ingredients.get_mut(i) {
                            ingredient.update(message);
                        }
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
            FoodCalc::AppInitialized => empty_message("Connection to Database successful"),
            FoodCalc::ErrorView(error) => empty_message(error),
            FoodCalc::IngredientView(IngredientsState {
                scroll,
                input,
                input_value,
                ingredients,
                dirty,
                saving,
            }) => {
                let title = Text::new("Ingredients")
                    .width(Length::Fill)
                    .size(100)
                    .color([0.5, 0.5, 0.5])
                    .horizontal_alignment(alignment::Horizontal::Center);

                let input = TextInput::new(input, "Ingredient Name", input_value, Message::InputChanged)
                    .padding(15)
                    .size(30)
                    .on_submit(Message::CreateIngredient);
                let filtered_ingredients = ingredients.iter().filter(|ingredient| {
                    ingredient
                        .ingredient
                        .name
                        .to_lowercase()
                        .contains(&*input_value.to_lowercase())
                });

                let ingredients: Element<_> = if filtered_ingredients.count() > 0 {
                    ingredients
                        .iter_mut()
                        .enumerate()
                        .filter(|(_, ingredient)| {
                            ingredient
                                .ingredient
                                .name
                                .to_lowercase()
                                .contains(&input_value.to_lowercase())
                        })
                        .fold(Column::new().spacing(20), |column, (i, ingredient)| {
                            column.push(
                                ingredient
                                    .view()
                                    .map(move |message| Message::IngredientMessage(i, message)),
                            )
                        })
                        .into()
                } else {
                    empty_message("No matching ingredient...")
                };

                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(title)
                    .push(input)
                    .push(ingredients);

                Scrollable::new(scroll)
                    .padding(40)
                    .push(Container::new(content).width(Length::Fill).center_x())
                    .into()
            },
        }
    }
}

fn loading_message<'a>() -> Element<'a, Message> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(alignment::Horizontal::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
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
