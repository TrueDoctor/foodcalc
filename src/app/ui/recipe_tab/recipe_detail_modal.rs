use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::{button, text_input, Alignment, Button, Column, Command, Element, Length, Row, Scrollable, Text, TextInput};

use super::RecipeTabMessage;
use crate::app::ui::{style, Icon};
use crate::db::{FoodBase, Recipe, RecipeEntry, RecipeMetaIngredient, RecipeStep, Unit};

mod recipe_ingredient_wrapper;
use recipe_ingredient_wrapper::{RecipeIngredientMessage, RecipeIngredientWrapper};
mod recipe_step_wrapper;
use recipe_step_wrapper::{RecipeStepMessage, RecipeStepWrapper};

#[derive(Debug, Clone)]
pub struct RecipeDetail {
    pub(crate) recipe: Recipe,
    pub(crate) recipe_description: text_input::State,
    database: Arc<FoodBase>,
    pub(crate) all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    pub(crate) all_units: Arc<Vec<Unit>>,
    pub(crate) ingredients: Vec<RecipeIngredientWrapper>,
    pub(crate) steps: Vec<RecipeStepWrapper>,
    pub(crate) scroll_ingredients: iced::scrollable::State,
    pub(crate) scroll_steps: iced::scrollable::State,
    pub(crate) cancel_state: button::State,
    pub(crate) ok_state: button::State,
    pub(crate) add_ingredient_button: button::State,
    pub(crate) add_step_button: button::State,
}

#[derive(Debug, Clone)]
pub enum RecipeDetailMessage {
    RecipeIngredientMessage(usize, RecipeIngredientMessage),
    RecipeStepMessage(usize, RecipeStepMessage),
    DescriptionEdited(String),
    SubmitDescription,
    AddIngredient,
    AddStep,
    Delete,
    Save,
    Cancel,
}

impl RecipeDetail {
    pub fn new(
        recipe: Recipe,
        all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
        all_units: Arc<Vec<Unit>>,
        database: Arc<FoodBase>,
        recipe_ingredients: Vec<RecipeEntry>,
        recipe_steps: Vec<RecipeStep>,
    ) -> Self {
        Self {
            recipe,
            all_ingredients: all_ingredients.clone(),
            all_units: all_units.clone(),
            database,
            ingredients: recipe_ingredients
                .into_iter()
                .map(|ingredient| RecipeIngredientWrapper::new(all_ingredients.clone(), all_units.clone(), ingredient))
                .collect(),
            steps: recipe_steps
                .into_iter()
                .map(|step| RecipeStepWrapper::edit(step))
                .collect(),
            recipe_description: Default::default(),
            scroll_ingredients: Default::default(),
            scroll_steps: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
            add_ingredient_button: Default::default(),
            add_step_button: Default::default(),
        }
    }

    pub fn update(&mut self, message: RecipeDetailMessage) -> Command<RecipeTabMessage> {
        match message {
            RecipeDetailMessage::DescriptionEdited(new_description) => {
                self.recipe.comment = Some(new_description);
            },
            RecipeDetailMessage::Delete => {},
            RecipeDetailMessage::SubmitDescription => {},
            RecipeDetailMessage::RecipeIngredientMessage(i, RecipeIngredientMessage::Focus) => {
                for (j, ingredient) in self.ingredients.iter_mut().enumerate() {
                    if j != i {
                        ingredient.update(RecipeIngredientMessage::Unfocus);
                    }
                }
            },
            RecipeDetailMessage::RecipeIngredientMessage(i, RecipeIngredientMessage::Delete) => {
                log::trace!("Deleted recipe entry: {:?}", self.ingredients.remove(i).entry);
            },
            RecipeDetailMessage::RecipeStepMessage(i, RecipeStepMessage::Delete) => {
                log::trace!("Deleted recipe entry: {:?}", self.ingredients.remove(i).entry);
            },
            RecipeDetailMessage::RecipeIngredientMessage(i, message) => {
                if let Some(recipe_ingredient) = self.ingredients.get_mut(i) {
                    recipe_ingredient.update(message);
                }
            },
            RecipeDetailMessage::RecipeStepMessage(i, message) => {
                if let Some(recipe_step) = self.steps.get_mut(i) {
                    recipe_step.update(message);
                }
            },
            RecipeDetailMessage::Save => {
                let move_database = self.database.clone();
                let recipe = self.recipe.clone();
                let ingredients: Vec<_> = self
                    .ingredients
                    .iter()
                    .map(|entry_wrapper| entry_wrapper.entry.clone())
                    .collect();

                return Command::perform(
                    async move {
                        move_database.update_recipe(&recipe).await?;
                        move_database
                            .update_recipe_entries(&recipe, ingredients.into_iter())
                            .await?;
                        Ok(())
                    },
                    RecipeTabMessage::SaveRecipe,
                );
            },
            RecipeDetailMessage::AddIngredient => {
                self.ingredients.push(RecipeIngredientWrapper::new(
                    self.all_ingredients.clone(),
                    self.all_units.clone(),
                    RecipeEntry::default(),
                ));
            },
            RecipeDetailMessage::AddStep => {
                let max_order = self.steps.last().map(|step| step.entry.step_order).unwrap_or(0.);
                self.steps.push(RecipeStepWrapper::create(max_order + 1.));
            },
            RecipeDetailMessage::Cancel => {
                println!("Cancel");
                return Command::perform(async {}, |_| RecipeTabMessage::CancelButtonPressed);
            },
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<RecipeDetailMessage> {
        let theme = crate::theme();
        let description_input = TextInput::new(
            &mut self.recipe_description,
            "Recipe Description…",
            self.recipe.comment.as_deref().unwrap_or(""),
            RecipeDetailMessage::DescriptionEdited,
        )
        .on_submit(RecipeDetailMessage::SubmitDescription)
        .style(theme)
        .padding(10);

        let ingredients: Element<'_, RecipeDetailMessage> = self
            .ingredients
            .iter_mut()
            .enumerate()
            .fold(Column::new().spacing(10), |column, (i, recipe)| {
                column.push(
                    recipe
                        .view()
                        .map(move |message| RecipeDetailMessage::RecipeIngredientMessage(i, message)),
                )
            })
            .into();

        let title = Text::new(&self.recipe.name).color(theme.foreground()).size(30);

        let add_ingredient_button = Button::new(
            &mut self.add_ingredient_button,
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(Text::new("Add Ingredient")),
        )
        .on_press(RecipeDetailMessage::AddIngredient)
        .padding(10)
        .style(style::Button::Add);

        let ingredients = Scrollable::new(&mut self.scroll_ingredients)
            .push(ingredients)
            .push(add_ingredient_button)
            .align_items(Alignment::Start)
            .spacing(20)
            .height(Length::FillPortion(4));

        let steps: Element<'_, RecipeDetailMessage> = self
            .steps
            .iter_mut()
            .enumerate()
            .fold(Column::new().spacing(10), |column, (i, recipe)| {
                column.push(
                    recipe
                        .view()
                        .map(move |message| RecipeDetailMessage::RecipeStepMessage(i, message)),
                )
            })
            .into();

        let add_step_button = Button::new(
            &mut self.add_step_button,
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(Text::new("Add Step")),
        )
        .on_press(RecipeDetailMessage::AddStep)
        .padding(10)
        .style(style::Button::Add);

        let steps = Scrollable::new(&mut self.scroll_steps)
            .push(steps)
            .push(add_step_button)
            .align_items(Alignment::Start)
            .spacing(20)
            .height(Length::FillPortion(4));

        let cancel_button = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(RecipeDetailMessage::Cancel);

        let ok_button = Button::new(
            &mut self.ok_state,
            Text::new("Save").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(RecipeDetailMessage::Save);

        let footer = Row::new()
            .spacing(10)
            .padding(5)
            .width(Length::Fill)
            .height(Length::Units(50))
            .push(cancel_button)
            .push(ok_button);

        Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(title)
            .push(description_input)
            .push(ingredients)
            .push(steps)
            .push(footer)
            .into()
    }
}
