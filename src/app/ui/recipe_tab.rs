use std::sync::Arc;

use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{alignment, alignment::Horizontal, button, Button, Column, Command, Container, Element, Length, Row, Text};
use log::debug;

use super::TabMessage;
use crate::db::{FoodBase, Recipe};

use crate::app::Error;

mod recipe;
pub use recipe::RecipeWrapper;

mod recipe_detail_modal;
use recipe_detail_modal::{RecipeDetail, RecipeDetailMessage};

#[derive(Debug, Clone, Default)]
pub struct ModalState {
    cancel_state: button::State,
    ok_state: button::State,
    recipe_detail_modal: Option<RecipeDetail>,
}

#[derive(Debug, Clone)]
pub struct RecipeTab {
    recipe_list: Vec<RecipeWrapper>,
    scroll: scrollable::State,
    input: text_input::State,
    input_value: String,
    database: Arc<FoodBase>,
    recipe_detail_modal: ModalState,
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
    SaveRecipe,
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
            recipe_detail_modal: ModalState::default(),
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
                if let Some(modal) = self.recipe_detail_modal.recipe_detail_modal.as_mut() {
                    modal.update(message);
                }
            },
            RecipeTabMessage::CancelButtonPressed => {
                self.recipe_detail_modal.recipe_detail_modal = None;
            },
            RecipeTabMessage::CloseModal => {
                self.recipe_detail_modal.recipe_detail_modal = None;
            },
            RecipeTabMessage::ShowModal(Ok(recipe_detail)) => {
                self.recipe_detail_modal.recipe_detail_modal = Some(recipe_detail);
            },
            RecipeTabMessage::OpenModal(recipe) => {
                let move_database = self.database.clone();
                return Command::perform(
                    async move {
                        let all_ingredients = move_database.get_all_meta_ingredients().await?;
                        let all_units = move_database.get_units().await?;
                        let entries = move_database.get_meta_ingredients(recipe.recipe_id).await?;
                        Ok(RecipeDetail::new(
                            recipe,
                            Arc::new(all_ingredients),
                            Arc::new(all_units),
                            entries,
                        ))
                    },
                    RecipeTabMessage::ShowModal,
                )
                .map(|message| TabMessage::RecipeTab(message.into()));
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
        let theme = crate::theme();

        let input = TextInput::new(
            &mut self.input,
            "Recipe Name",
            &self.input_value,
            RecipeTabMessage::InputChanged,
        )
        .padding(15)
        .style(theme)
        .size(30);
        let filtered_recipes = self
            .recipe_list
            .iter()
            .filter(|recipe| crate::similar(&recipe.recipe.name, &*self.input_value));

        let recipes: Element<_> = if filtered_recipes.count() > 0 {
            self.recipe_list
                .iter_mut()
                .enumerate()
                .filter(|(_, recipe)| crate::similar(&recipe.recipe.name, &self.input_value))
                .fold(Column::new().spacing(00), |column, (_i, recipe)| {
                    column.push(recipe.view())
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

        let modal_shown = self.recipe_detail_modal.recipe_detail_modal.is_some();
        let modal_state = &mut self.recipe_detail_modal.recipe_detail_modal;

        let modal: Element<'_, RecipeTabMessage> =
            Column::new().spacing(30)
            .push(
            Text::new(
                modal_state
                    .as_ref()
                    .map(|x| x.recipe.name.as_str())
                    .unwrap_or_default()
                    .to_string(),
                ).color(theme.foreground()).size(30))
            .push(
            modal_state
                .as_mut()
                .map(|recipe_detail| recipe_detail.view().map(RecipeTabMessage::RecipeDetailMessage))
                .unwrap_or_else(|| empty_message("No recipe selected...")))
        .push(
            Row::new()
                .spacing(10)
                .padding(5)
                .width(Length::Fill)
                .height(Length::Units(50))
                .push(
                    Button::new(

                        &mut self.recipe_detail_modal.cancel_state,
                        Text::new("Cancel").horizontal_alignment(Horizontal::Center),
                    )
                    .width(Length::Fill)
                    .style(theme)
                    .on_press(RecipeTabMessage::CancelButtonPressed),
                )
                .push(
                    Button::new(
                        &mut self.recipe_detail_modal.ok_state,
                        Text::new("Ok").horizontal_alignment(Horizontal::Center),
                    )
                    .width(Length::Fill)
                    .style(theme)
                    .on_press(RecipeTabMessage::SaveRecipe),
                ),
        ).max_width(1000).into()/*
        .max_width(1000)
        //.width(Length::Shrink)
        //.height(Length::Shrink)
        .on_close(RecipeTabMessage::CloseModal)
        .into()*/;
        let modal: Element<'_, RecipeTabMessage> = Container::new(modal)
            .center_x()
            .height(Length::Fill)
            .width(Length::Fill)
            .into();
        let element: Element<'_, RecipeTabMessage> = match modal_shown {
            true => modal,
            false => element,
        };

        element.map(|message| TabMessage::RecipeTab(message.into()))
    }

    /*fn modal(&self, content: Element<'_, RecipeTabMessage>) {

    }*/

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
