use std::sync::Arc;

use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{alignment, Column, Command, Container, Element, Length, Text};

mod ingredient;
pub use ingredient::{IngredientMessage, IngredientWrapper};

use super::TabMessage;
use crate::db::FoodBase;

#[derive(Clone, Debug)]
pub struct IngredientTab {
    ingredient_list: Vec<IngredientWrapper>,
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    database: Arc<FoodBase>,
}

#[derive(Debug, Clone)]
pub enum IngredientTabMessage {
    InputChanged(String),
    IngredientMessage(usize, IngredientMessage),
    UpdateData(Result<Vec<IngredientWrapper>, Error>),
}

#[derive(Debug, Clone)]
pub enum Error {
    Misc(String),
    Database(String),
}

impl From<eyre::ErrReport> for Error {
    fn from(error: eyre::ErrReport) -> Self {
        Error::Misc(format!("Error occurred {error}"))
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::Database(format!("Database Error occurred {error}"))
    }
}

impl IngredientTab {
    pub fn new(database: Arc<FoodBase>) -> (Self, Command<TabMessage>) {
        let move_database = database.clone();
        let command = Command::perform(
            async move {
                let ingredients = move_database
                    .get_ingredients()
                    .await?
                    .into_iter()
                    .map(IngredientWrapper::new)
                    .collect();
                Ok(ingredients)
            },
            IngredientTabMessage::UpdateData,
        );
        let ingredients = IngredientTab {
            database,
            scroll: scrollable::State::default(),
            input: text_input::State::default(),
            input_value: String::new(),
            ingredient_list: Vec::new(),
        };
        (
            ingredients,
            command.map(|message| TabMessage::IngredientTab(message.into())),
        )
    }

    pub fn update(&mut self, message: IngredientTabMessage) -> Command<TabMessage> {
        match message {
            IngredientTabMessage::UpdateData(Ok(ingredients)) => {
                self.ingredient_list = ingredients;
                Command::none()
            },
            IngredientTabMessage::InputChanged(input) => {
                self.input_value = input;
                Command::none()
            },
            IngredientTabMessage::IngredientMessage(i, message) => {
                if let Some(ingredient) = self.ingredient_list.get_mut(i) {
                    ingredient
                        .update(message, &self.database)
                        .map(move |message| IngredientTabMessage::IngredientMessage(i, message))
                        .map(|message| TabMessage::IngredientTab(message.into()))
                } else {
                    Command::none()
                }
            },
            _ => {
                log::warn!("recieved message without handler: {message:?}");
                Command::none()
            },
        }
    }
}

impl super::Tab for IngredientTab {
    type Message = TabMessage;

    fn title(&self) -> String {
        "Ingredients".to_string()
    }

    fn content(&mut self) -> Element<'_, Self::Message> {
        let theme = crate::theme();

        let input = TextInput::new(
            &mut self.input,
            "Ingredient Name",
            &self.input_value,
            IngredientTabMessage::InputChanged,
        )
        .padding(15)
        .style(theme)
        .size(30);
        let filtered_ingredients = self.ingredient_list.iter().filter(|ingredient| {
            ingredient
                .ingredient
                .name
                .to_lowercase()
                .contains(&*self.input_value.to_lowercase())
        });

        let ingredients: Element<_> = if filtered_ingredients.count() > 0 {
            self.ingredient_list
                .iter_mut()
                .enumerate()
                .filter(|(_, ingredient)| crate::similar(&ingredient.ingredient.name, &self.input_value))
                .fold(Column::new().spacing(00), |column, (i, ingredient)| {
                    column.push(
                        ingredient
                            .view()
                            .map(move |message| IngredientTabMessage::IngredientMessage(i, message)),
                    )
                })
                .into()
        } else {
            empty_message("No matching ingredient...")
        };

        let scroll: Element<'_, IngredientTabMessage> = Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(Container::new(ingredients).width(Length::Fill))
            .into();

        let element: Element<'_, IngredientTabMessage> =
            Column::new().max_width(800).spacing(20).push(input).push(scroll).into();
        let element: Element<'_, IngredientTabMessage> = Container::new(element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into();

        element.map(|message| TabMessage::IngredientTab(message.into()))
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        super::TabLabel::IconText(super::Icon::Apple.into(), self.title())
    }
}

fn empty_message<'a>(message: &str) -> Element<'a, IngredientTabMessage> {
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
