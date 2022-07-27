use std::borrow::Cow;
use std::sync::Arc;

use iced::{button, text_input, Alignment, Button, Element, Length, Row, Text, TextInput};
use sqlx::types::BigDecimal;

use crate::app::ui::style;
use crate::app::ui::Icon;
use crate::db::RecipeEntry;
use crate::db::RecipeMetaIngredient;
use crate::db::Unit;

#[derive(Debug, Clone, Default)]
pub struct RecipeIngredientWrapper {
    entry: RecipeEntry,
    amount: text_input::State,
    unit_list: iced::pick_list::State<Unit>,
    ingredient_list: iced_searchable_picklist::State<RecipeMetaIngredient>,
    all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    filtered_ingredients: Option<Vec<RecipeMetaIngredient>>,
    ingredient_filter: String,
    amount_text: String,
    delete_button: button::State,
    amount_valid: bool,
}

#[derive(Debug, Clone)]
pub enum RecipeIngredientMessage {
    FilterChanged(String),
    AmountChanged(String),
    PickUnit(Unit),
    SubmitAmount,
    PickIngredient(RecipeMetaIngredient),
    SubmitFilter,
}

impl RecipeIngredientWrapper {
    pub fn new(ingredients: Arc<Vec<RecipeMetaIngredient>>, entry: RecipeEntry) -> Self {
        Self {
            all_ingredients: ingredients,
            entry,
            ..Default::default()
        }
    }

    pub fn update(&mut self, message: RecipeIngredientMessage) {
        match message {
            RecipeIngredientMessage::FilterChanged(name) => {
                self.ingredient_filter = name;
                self.filtered_ingredients = (!self.ingredient_filter.is_empty()).then(|| {
                    self.all_ingredients
                        .iter()
                        .filter(|ingredient| crate::similar(ingredient.name(), &self.ingredient_filter))
                        .cloned()
                        .collect::<Vec<_>>()
                })
            },
            RecipeIngredientMessage::AmountChanged(amount) => self.amount_text = amount,
            RecipeIngredientMessage::PickUnit(unit) => self.entry.unit = unit,
            RecipeIngredientMessage::SubmitAmount => (),
            RecipeIngredientMessage::PickIngredient(ingredient) => {
                self.entry.ingredient = ingredient;
                self.filtered_ingredients = None
            },
            RecipeIngredientMessage::SubmitFilter => (),
        }
    }

    pub fn view(&mut self) -> Element<RecipeIngredientMessage> {
        let ingredient_list = iced_searchable_picklist::PickList::new(
            &mut self.ingredient_list,
            dbg!(self.filtered_ingredients.as_ref().unwrap_or(&*self.all_ingredients)),
            Some(self.entry.ingredient.clone()),
            RecipeIngredientMessage::PickIngredient,
            RecipeIngredientMessage::FilterChanged,
            &self.ingredient_filter,
        )
        .on_submit(RecipeIngredientMessage::SubmitFilter)
        .padding(10);

        let amount_input = TextInput::new(
            &mut self.amount,
            "Ingredient Amount…",
            &self.amount_text,
            RecipeIngredientMessage::AmountChanged,
        )
        .on_submit(RecipeIngredientMessage::SubmitAmount)
        .width(Length::Units(30))
        .padding(10);

        static units: &[Unit] = &[Unit {
            unit_id: 0,
            name: Cow::Borrowed("Kilo"),
        }];
        let unit_list =
            iced::PickList::new(&mut self.unit_list, units, None, RecipeIngredientMessage::PickUnit).padding(10);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(ingredient_list)
            .push(amount_input)
            .push(unit_list)
            .into()
    }
}
