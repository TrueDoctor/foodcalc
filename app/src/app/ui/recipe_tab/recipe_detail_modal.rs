use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::widget::*;
use iced::{Alignment, Command, Element, Length};

use super::RecipeTabMessage;
use crate::app::ui::Icon;
use crate::db::{FoodBase, Recipe, RecipeIngredient, RecipeMetaIngredient, RecipeStep, Unit};

mod recipe_ingredient_wrapper;
use recipe_ingredient_wrapper::{RecipeIngredientMessage, RecipeIngredientWrapper};
mod recipe_step_wrapper;
use recipe_step_wrapper::{RecipeStepMessage, RecipeStepWrapper};

#[derive(Debug, Clone)]
pub struct RecipeDetail {
    pub(crate) recipe: Recipe,
    database: Arc<FoodBase>,
    pub(crate) all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    pub(crate) all_units: Arc<Vec<Unit>>,
    pub(crate) ingredients: Vec<RecipeIngredientWrapper>,
    pub(crate) steps: Vec<RecipeStepWrapper>,
    is_new: bool,
}

#[derive(Debug, Clone)]
pub enum RecipeDetailMessage {
    RecipeIngredientMessage(usize, RecipeIngredientMessage),
    RecipeStepMessage(usize, RecipeStepMessage),
    NameEdited(String),
    DescriptionEdited(String),
    SubmitName,
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
        recipe_ingredients: Vec<RecipeIngredient>,
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
            steps: recipe_steps.into_iter().map(RecipeStepWrapper::edit).collect(),
            is_new: false,
        }
    }

    pub fn create(
        all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
        all_units: Arc<Vec<Unit>>,
        database: Arc<FoodBase>,
    ) -> Self {
        Self {
            recipe: Recipe::default(),
            all_ingredients,
            all_units,
            database,
            ingredients: Vec::new(),
            steps: Vec::new(),
            is_new: true,
        }
    }

    pub fn valid(&self) -> bool {
        self.ingredients.iter().all(|ingredient| ingredient.valid()) && self.steps.iter().all(|step| step.valid())
    }

    pub fn update(&mut self, message: RecipeDetailMessage) -> Command<RecipeTabMessage> {
        match message {
            RecipeDetailMessage::NameEdited(new_name) => {
                self.recipe.name = new_name;
            },
            RecipeDetailMessage::DescriptionEdited(new_description) => {
                self.recipe.comment = Some(new_description);
            },
            RecipeDetailMessage::Delete => {},
            RecipeDetailMessage::SubmitName => {},
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
                log::trace!("Deleted recipe entry: {:?}", self.steps.remove(i).entry);
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
                let steps: Vec<_> = self
                    .steps
                    .iter()
                    .map(|entry_wrapper| entry_wrapper.entry.clone())
                    .collect();

                if self.valid() {
                    let is_new = self.is_new;
                    return Command::perform(
                        async move {
                            if is_new {
                                move_database.insert_recipe(&recipe).await?;
                            } else {
                                move_database.update_recipe(&recipe).await?;
                            }
                            move_database.update_recipe(&recipe).await?;
                            move_database
                                .update_recipe_entries(recipe.recipe_id, ingredients.into_iter())
                                .await?;
                            move_database
                                .update_recipe_steps(recipe.recipe_id, steps.into_iter())
                                .await?;
                            Ok(())
                        },
                        RecipeTabMessage::SaveRecipe,
                    );
                } else {
                    log::error!("Recipe is not valid");
                }
            },
            RecipeDetailMessage::AddIngredient => {
                self.ingredients.push(RecipeIngredientWrapper::new(
                    self.all_ingredients.clone(),
                    self.all_units.clone(),
                    RecipeIngredient::default(),
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

    pub fn view(&self) -> Element<RecipeDetailMessage> {
        let name_input = TextInput::new(
            "Recipe name…",
            self.recipe.name.as_str(),
            RecipeDetailMessage::NameEdited,
        )
        .on_submit(RecipeDetailMessage::SubmitName)
        .padding(10);

        let description_input = TextInput::new(
            "Recipe Description…",
            self.recipe.comment.as_deref().unwrap_or(""),
            RecipeDetailMessage::DescriptionEdited,
        )
        .on_submit(RecipeDetailMessage::SubmitDescription)
        .padding(10);

        let ingredients: Element<'_, RecipeDetailMessage> = self
            .ingredients
            .iter()
            .enumerate()
            .fold(Column::new().spacing(10), |column, (i, recipe)| {
                column.push(
                    recipe
                        .view()
                        .map(move |message| RecipeDetailMessage::RecipeIngredientMessage(i, message)),
                )
            })
            .into();

        let title = text(&self.recipe.name).size(30);

        let add_ingredient_button = Button::new(
            Row::new()
                .spacing(10)
                .push(Icon::Plus.text())
                .push(text("Add Ingredient")),
        )
        .on_press(RecipeDetailMessage::AddIngredient)
        .padding(10)
        .style(iced::theme::Button::Positive);

        let ingredients = Scrollable::new(
            iced::widget::column![ingredients, add_ingredient_button]
                .align_items(Alignment::Start)
                .spacing(20),
        )
        .height(Length::FillPortion(4));

        let steps: Element<'_, RecipeDetailMessage> = self
            .steps
            .iter()
            .enumerate()
            .fold(Column::new().spacing(40), |column, (i, recipe)| {
                column.push(
                    recipe
                        .view()
                        .map(move |message| RecipeDetailMessage::RecipeStepMessage(i, message)),
                )
            })
            .into();

        let add_step_button = Button::new(Row::new().spacing(10).push(Icon::Plus.text()).push(text("Add Step")))
            .on_press(RecipeDetailMessage::AddStep)
            .padding(10)
            .style(iced::theme::Button::Positive);

        let steps = Scrollable::new(
            iced::widget::column![steps, add_step_button]
                .align_items(Alignment::Start)
                .spacing(20),
        )
        .height(Length::FillPortion(4));

        let cancel_button = Button::new(text("Cancel").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
            .on_press(RecipeDetailMessage::Cancel);

        let ok_button = Button::new(text("Save").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
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
            .push(name_input)
            .push(description_input)
            .push(ingredients)
            .push(steps)
            .push(footer)
            .into()
    }
}
