use std::sync::Arc;

use iced::button::Button;
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{alignment, button, Alignment, Column, Command, Container, Element, Length, Row, Space, Text};

mod ingredient;
pub use ingredient::{IngredientMessage, IngredientWrapper};

mod ingredient_create;
use ingredient_create::IngredientCreationDialog;

use self::ingredient_create::IngredientCreateMessage;
use super::{style, Icon, TabMessage};
use crate::db::{FoodBase, Ingredient, IngredientCreate};

#[derive(Clone, Debug)]
pub struct IngredientTab {
    ingredient_list: Vec<IngredientWrapper>,
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    database: Arc<FoodBase>,
    pub(crate) add_ingredient_button: button::State,
    ingredient_create: Option<IngredientCreationDialog>,
}

#[derive(Debug, Clone)]
pub enum IngredientTabMessage {
    InputChanged(String),
    IngredientMessage(usize, IngredientMessage),
    IngredientDetailMessage(IngredientCreateMessage),
    UpdateData(Result<Vec<IngredientWrapper>, Error>),
    AddIngredient,
    EditIngredient(i32),
    CloseCreateIngredient,
    UpdateIngredient(IngredientCreate),
    Refresh,
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
        let mut ingredients = IngredientTab {
            database,
            scroll: scrollable::State::default(),
            input: text_input::State::default(),
            input_value: String::new(),
            ingredient_list: Vec::new(),
            add_ingredient_button: button::State::default(),
            ingredient_create: None,
        };
        let command = ingredients.update(IngredientTabMessage::Refresh);
        (ingredients, command)
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
                        .update(message)
                        .map(|message| TabMessage::IngredientTab(message.into()))
                } else {
                    Command::none()
                }
            },
            IngredientTabMessage::IngredientDetailMessage(message) => {
                if let Some(message) = self.ingredient_create.as_mut().and_then(|cd| cd.update(message)) {
                    Command::perform(async move { Box::new(message.clone()) }, TabMessage::IngredientTab)
                } else {
                    Command::none()
                }
            },
            IngredientTabMessage::AddIngredient => {
                self.ingredient_create = Some(IngredientCreationDialog::default());
                Command::none()
            },
            IngredientTabMessage::EditIngredient(id) => {
                let Some(ingredient) = self.ingredient_list.iter().find(|i| i.ingredient.ingredient_id == id)
                    else { log::error!("Tried to edit non existing ingredient"); return Command::none() };
                self.ingredient_create = Some(IngredientCreationDialog::edit(ingredient.ingredient.clone()));
                Command::none()
            },
            IngredientTabMessage::CloseCreateIngredient => {
                self.ingredient_create = None;
                Command::none()
            },
            IngredientTabMessage::Refresh => {
                log::debug!("Refreshing ingredient list");
                let move_database = self.database.clone();
                Command::perform(
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
                )
                .map(|message| TabMessage::IngredientTab(message.into()))
            },
            IngredientTabMessage::UpdateIngredient(ingredient) => {
                self.ingredient_create = None;
                let move_database = self.database.clone();
                Command::perform(
                    async move {
                        if ingredient.id.is_some() {
                            move_database.update_ingredient(ingredient.to_ingredient()?).await?;
                        } else {
                            move_database
                                .add_ingredient(ingredient.name, ingredient.energy, ingredient.comment)
                                .await?;
                        }
                        Ok(())
                    },
                    |_: Result<(), Error>| Box::new(IngredientTabMessage::Refresh),
                )
                .map(TabMessage::IngredientTab)
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
            .push(Container::new(ingredients).width(Length::Fill).height(Length::Shrink))
            .height(Length::Fill)
            .into();

        let add_ingredient_button = Button::new(&mut self.add_ingredient_button, Icon::Plus.text())
            .on_press(IngredientTabMessage::AddIngredient)
            .padding(10)
            .height(Length::Units(40))
            .width(Length::Units(60))
            .style(style::Button::Add);

        let element: Element<'_, IngredientTabMessage> = if let Some(ingredient_create) = &mut self.ingredient_create {
            ingredient_create
                .view()
                .map(IngredientTabMessage::IngredientDetailMessage)
        } else {
            Column::new()
                .max_width(800)
                .spacing(20)
                .push(input)
                .push(scroll)
                .push(add_ingredient_button)
                .push(Space::with_height(Length::Units(10)))
                .into()
        };
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
