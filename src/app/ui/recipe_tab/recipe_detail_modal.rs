use std::sync::Arc;

use iced::{
    alignment::Horizontal, button, text_input, Alignment, Button, Column, Command, Element, Length, Row, Scrollable,
    Text, TextInput,
};

use super::RecipeTabMessage;
use crate::app::ui::{style, Icon};
use crate::db::{FoodBase, Recipe, RecipeMetaIngredient};
use crate::db::{RecipeEntry, Unit};

mod recipe_ingredient_wrapper;
use recipe_ingredient_wrapper::{RecipeIngredientMessage, RecipeIngredientWrapper};

#[derive(Debug, Clone)]
pub struct RecipeDetail {
    pub(crate) recipe: Recipe,
    pub(crate) recipe_description: text_input::State,
    database: Arc<FoodBase>,
    pub(crate) all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    pub(crate) all_units: Arc<Vec<Unit>>,
    pub(crate) ingredients: Vec<RecipeIngredientWrapper>,
    pub(crate) scroll: iced::scrollable::State,
    pub(crate) cancel_state: button::State,
    pub(crate) ok_state: button::State,
    pub(crate) add_ingredient_button: button::State,
}

#[derive(Debug, Clone)]
pub enum RecipeDetailMessage {
    RecipeIngredientMessage(usize, RecipeIngredientMessage),
    DescriptionEdited(String),
    SubmitDescription,
    AddIngredient,
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
            recipe_description: Default::default(),
            scroll: Default::default(),
            cancel_state: Default::default(),
            ok_state: Default::default(),
            add_ingredient_button: Default::default(),
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
            RecipeDetailMessage::RecipeIngredientMessage(i, message) => {
                if let Some(recipe_ingredient) = self.ingredients.get_mut(i) {
                    recipe_ingredient.update(message);
                }
            },
            RecipeDetailMessage::Save => log::warn!("implement saving to database"),
            RecipeDetailMessage::AddIngredient => {
                self.ingredients.push(RecipeIngredientWrapper::new(
                    self.all_ingredients.clone(),
                    self.all_units.clone(),
                    RecipeEntry::default(),
                ));
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

        let ingredients = Scrollable::new(&mut self.scroll)
            .push(ingredients)
            .push(add_ingredient_button)
            .align_items(Alignment::Start)
            .spacing(20)
            .height(Length::Fill);

        let cancel_button = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(RecipeDetailMessage::Cancel);

        let ok_button = Button::new(
            &mut self.ok_state,
            Text::new("Ok").horizontal_alignment(Horizontal::Center),
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
            .push(footer)
            .into()
    }
}
