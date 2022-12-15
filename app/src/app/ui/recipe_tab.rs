use std::sync::Arc;

use iced::widget::*;
use iced::{alignment, Command, Element, Length};
use log::debug;

use super::TabMessage;
use crate::app::Error;
use crate::db::{FoodBase, Recipe};

mod recipe;
pub use recipe::RecipeWrapper;

mod recipe_detail_modal;
use recipe_detail_modal::{RecipeDetail, RecipeDetailMessage};

#[derive(Debug, Clone)]
pub struct RecipeTab {
    recipe_list: Vec<RecipeWrapper>,
    input_value: String,
    database: Arc<FoodBase>,
    recipe_detail_modal: Option<RecipeDetail>,
}

#[derive(Debug, Clone)]
pub enum RecipeTabMessage {
    InputChanged(String),
    RecipeDetailMessage(RecipeDetailMessage),
    UpdateData(Result<Vec<RecipeWrapper>, Error>),
    OpenModal(Recipe),
    ShowModal(Result<RecipeDetail, Error>),
    CloseModal,
    CancelButtonPressed,
    SaveRecipe(Result<(), Error>),
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
            input_value: String::new(),
            recipe_list: Vec::new(),
            recipe_detail_modal: None,
        };
        (recipes, command.map(|message| TabMessage::RecipeTab(message.into())))
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
            RecipeTabMessage::RecipeDetailMessage(message) => {
                if let Some(modal) = self.recipe_detail_modal.as_mut() {
                    return modal
                        .update(message)
                        .map(|message| TabMessage::RecipeTab(message.into()));
                }
            },
            RecipeTabMessage::CancelButtonPressed => {
                println!("Cancel");
                self.recipe_detail_modal = None;
            },
            RecipeTabMessage::CloseModal => {
                self.recipe_detail_modal = None;
            },
            RecipeTabMessage::ShowModal(Ok(recipe_detail)) => {
                self.recipe_detail_modal = Some(recipe_detail);
            },
            RecipeTabMessage::OpenModal(recipe) => {
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let all_ingredients = move_database.get_all_meta_ingredients().await?;
                        let all_units = move_database.get_units().await?;
                        let entries = move_database.get_meta_ingredients(recipe.recipe_id).await?;
                        let steps = move_database.get_recipe_steps(recipe.recipe_id).await?;
                        Ok(RecipeDetail::new(
                            recipe,
                            Arc::new(all_ingredients),
                            Arc::new(all_units),
                            move_database.clone(),
                            entries,
                            steps,
                        ))
                    },
                    RecipeTabMessage::ShowModal,
                )
                .map(|message| TabMessage::RecipeTab(message.into()));
            },
            RecipeTabMessage::SaveRecipe(Ok(())) => {
                if crate::app::ui::settings::close_on_save() {
                    self.recipe_detail_modal = None;
                }
                let move_database = self.database.clone();
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
                return command.map(|message| TabMessage::RecipeTab(message.into()));
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

    fn content(&self) -> Element<'_, Self::Message> {
        let theme = crate::theme();

        let input = TextInput::new("Recipe Name", &self.input_value, RecipeTabMessage::InputChanged)
            .padding(15)
            .size(30);
        let filtered_recipes = self
            .recipe_list
            .iter()
            .filter(|recipe| crate::similar(&recipe.recipe.name, &*self.input_value));

        let recipes: Element<_> = if filtered_recipes.count() > 0 {
            self.recipe_list
                .iter()
                .enumerate()
                .filter(|(_, recipe)| crate::similar(&recipe.recipe.name, &self.input_value))
                .fold(Column::new().spacing(00), |column, (_i, recipe)| {
                    column.push(recipe.view())
                })
                .into()
        } else {
            empty_message("No matching recipe...")
        };

        let scroll: Element<'_, RecipeTabMessage> =
            Scrollable::new(Container::new(recipes).width(Length::Fill).padding(40)).into();

        let element: Element<'_, RecipeTabMessage> =
            Column::new().max_width(800).spacing(20).push(input).push(scroll).into();

        let element: Element<'_, RecipeTabMessage> = Container::new(element)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into();

        let element: Element<'_, RecipeTabMessage> = match self.recipe_detail_modal.as_ref() {
            Some(modal) => modal.view().map(RecipeTabMessage::RecipeDetailMessage),
            None => element,
        };

        element.map(|message| TabMessage::RecipeTab(message.into()))
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        super::TabLabel::IconText(super::Icon::Burger.into(), self.title())
    }
}

fn empty_message<'a>(message: &str) -> Element<'a, RecipeTabMessage> {
    Container::new(
        text(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
    )
    .width(Length::Fill)
    .height(Length::Units(200))
    .center_y()
    .into()
}
