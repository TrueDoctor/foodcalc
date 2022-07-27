use std::sync::Arc;

use iced::{button, text_input, Alignment, Button, Column, Element, Length, Row, Text, TextInput};

use crate::db::RecipeEntry;
use crate::db::{Recipe, RecipeMetaIngredient};

mod recipe_ingredient_wrapper;
use recipe_ingredient_wrapper::{RecipeIngredientMessage, RecipeIngredientWrapper};

#[derive(Debug, Clone, Default)]
pub struct RecipeDetail {
    pub(crate) recipe: Recipe,
    pub(crate) recipe_name: text_input::State,
    pub(crate) recipe_description: text_input::State,
    pub(crate) all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    pub(crate) ingredients: Vec<RecipeIngredientWrapper>,
    //pub(crate) steps: Vec<RecipeStepWrapper>,
}

#[derive(Debug, Clone)]
pub enum RecipeDetailMessage {
    RecipeIngredientMessage(usize, RecipeIngredientMessage),
    DescriptionEdited(String),
    SubmitDescription,
    Delete,
    //Save,
    //Cancel,
}

impl RecipeDetail {
    pub fn new(
        recipe: Recipe,
        all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
        recipe_ingredients: Vec<RecipeEntry>,
    ) -> Self {
        Self {
            recipe,
            all_ingredients: all_ingredients.clone(),
            ingredients: recipe_ingredients
                .into_iter()
                .map(|ingredient| RecipeIngredientWrapper::new(all_ingredients.clone(), ingredient))
                .collect(),
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: RecipeDetailMessage) {
        match message {
            RecipeDetailMessage::DescriptionEdited(new_description) => {
                self.recipe.comment = Some(new_description);
            },
            RecipeDetailMessage::Delete => {},
            RecipeDetailMessage::SubmitDescription => {},
            RecipeDetailMessage::RecipeIngredientMessage(i, message) => {
                if let Some(recipe_ingredient) = self.ingredients.get_mut(i) {
                    recipe_ingredient.update(message);
                }
            },
        }
    }

    pub fn view(&mut self) -> Element<RecipeDetailMessage> {
        let description_input = TextInput::new(
            &mut self.recipe_description,
            "Recipe Description…",
            self.recipe.comment.as_deref().unwrap_or(""),
            RecipeDetailMessage::DescriptionEdited,
        )
        .on_submit(RecipeDetailMessage::SubmitDescription)
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
        Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(description_input)
            .push(ingredients)
            .into()
    }
}
