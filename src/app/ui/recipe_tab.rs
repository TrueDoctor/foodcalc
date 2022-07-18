use std::sync::Arc;

use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{alignment, Application, Column, Command, Container, Element, Length, Sandbox, Text};
use log::debug;

use super::model_wrapper::{RecipeMessage, RecipeWrapper};
use super::TabMessage;
use crate::db::FoodBase;

use crate::app::Error;

//pub mod state;

#[derive(Clone, Debug)]
pub struct RecipeTab {
    recipe_list: Vec<RecipeWrapper>,
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    database: Arc<FoodBase>,
    dirty: bool,
}

#[derive(Debug, Clone)]
pub enum RecipeTabMessage {
    InputChanged(String),
    RecipeMessage(usize, RecipeMessage),
    UpdateData(Result<Vec<RecipeWrapper>, Error>),
}

impl RecipeTab {
    pub fn new(database: Arc<FoodBase>) -> (Self, Command<TabMessage>) {
        let move_database = database.clone();
        let command = Command::perform(
            async move {
                let recipes = move_database
                    .get_recipes()
                    .await?
                    .into_iter()
                    .map(RecipeWrapper::new)
                    .collect();
                Ok(recipes)
            },
            RecipeTabMessage::UpdateData,
        );
        let recipes = RecipeTab {
            database,
            scroll: scrollable::State::default(),
            input: text_input::State::default(),
            input_value: String::new(),
            recipe_list: Vec::new(),
            dirty: false,
        };
        (recipes, command.map(TabMessage::RecipeTab))
    }

    pub fn update(&mut self, message: RecipeTabMessage) -> Command<TabMessage> {
        match message {
            RecipeTabMessage::UpdateData(Ok(recipes)) => {
                self.recipe_list = recipes;
            },
            RecipeTabMessage::UpdateData(Err(error)) => {
                log::error!("{error:?}");
            },
            RecipeTabMessage::InputChanged(input) => {
                self.input_value = input;
            },
            RecipeTabMessage::RecipeMessage(i, message) => {
                if let Some(recipe) = self.recipe_list.get_mut(i) {
                    recipe.update(message);
                }
            },
            _ => {
                debug!("recieved message without handler: {message:?}")
            },
        }
        Command::none()
    }
}

impl super::Tab for RecipeTab {
    type Message = TabMessage;

    fn title(&self) -> String {
        "Recipes".to_string()
    }

    fn content(&mut self) -> Element<'_, Self::Message> {
        let input = TextInput::new(
            &mut self.input,
            "Recipe Name",
            &self.input_value,
            RecipeTabMessage::InputChanged,
        )
        .padding(15)
        .size(30);
        let filtered_recipes = self.recipe_list.iter().filter(|recipe| {
            recipe
                .recipe
                .name
                .to_lowercase()
                .contains(&*self.input_value.to_lowercase())
        });

        let recipes: Element<_> = if filtered_recipes.count() > 0 {
            self.recipe_list
                .iter_mut()
                .enumerate()
                .filter(|(_, recipe)| {
                    recipe
                        .recipe
                        .name
                        .to_lowercase()
                        .contains(&self.input_value.to_lowercase())
                })
                .fold(Column::new().spacing(00), |column, (i, recipe)| {
                    column.push(
                        recipe
                            .view()
                            .map(move |message| RecipeTabMessage::RecipeMessage(i, message)),
                    )
                })
                .into()
        } else {
            empty_message("No matching recipe...")
        };

        let scroll: Element<'_, RecipeTabMessage> = Scrollable::new(&mut self.scroll)
            .padding(40)
            .push(Container::new(recipes).width(Length::Fill))
            .into();

        let element: Element<'_, RecipeTabMessage> =
            Column::new().max_width(800).spacing(20).push(input).push(scroll).into();
        let element: Element<'_, RecipeTabMessage> = Container::new(element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into();

        element.map(TabMessage::RecipeTab)
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        super::TabLabel::IconText(super::Icon::Burger.into(), self.title())
    }
}

fn empty_message<'a>(message: &str) -> Element<'a, RecipeTabMessage> {
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
