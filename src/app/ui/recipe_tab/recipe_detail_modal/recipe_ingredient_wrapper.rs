use std::sync::Arc;

use iced::{button, text_input, Alignment, Button, Element, Length, Row, Text, TextInput};
use num::Num;
use sqlx::types::BigDecimal;

use crate::app::ui::style::Button::Destructive;
use crate::app::ui::{style, Icon};
use crate::db::{RecipeIngrdient, RecipeMetaIngredient, Unit};

#[derive(Debug, Clone, Default)]
pub struct RecipeIngredientWrapper {
    pub(crate) entry: RecipeIngrdient,
    amount: text_input::State,
    unit_list: iced::pick_list::State<Unit>,
    ingredient_list: iced_searchable_picklist::State<RecipeMetaIngredient>,
    all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    all_units: Arc<Vec<Unit>>,
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
    Focus,
    Unfocus,
    PickIngredient(RecipeMetaIngredient),
    SubmitFilter,
    Delete,
}

impl RecipeIngredientWrapper {
    pub fn new(ingredients: Arc<Vec<RecipeMetaIngredient>>, all_units: Arc<Vec<Unit>>, entry: RecipeIngrdient) -> Self {
        Self {
            all_ingredients: ingredients,
            all_units,
            amount_text: entry.amount.to_string(),
            amount_valid: true,
            entry,
            ..Default::default()
        }
    }

    pub fn valid(&self) -> bool {
        self.amount_valid
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
            RecipeIngredientMessage::AmountChanged(amount) => {
                self.amount_text = amount;
                self.update(RecipeIngredientMessage::SubmitAmount)
            },
            RecipeIngredientMessage::PickUnit(unit) => self.entry.unit = unit,
            RecipeIngredientMessage::SubmitAmount => {
                if let Ok(num) = BigDecimal::from_str_radix(&self.amount_text, 10) {
                    self.entry.amount = num;
                    self.amount_valid = true;
                } else {
                    log::error!("Invalid amount: {}", self.amount_text);
                    self.amount_valid = false;
                }
            },
            RecipeIngredientMessage::PickIngredient(ingredient) => {
                self.entry.ingredient = ingredient;
                if let RecipeMetaIngredient::MetaRecipe(_) = self.entry.ingredient {
                    self.entry.unit = Unit::default();
                }
            },
            RecipeIngredientMessage::SubmitFilter => {
                if let Some([elem]) = self.filtered_ingredients.as_deref() {
                    self.ingredient_list.unfocus();
                    self.entry.ingredient = elem.clone();
                }
            },
            RecipeIngredientMessage::Focus => self.ingredient_list.focus(),
            RecipeIngredientMessage::Unfocus => self.ingredient_list.unfocus(),
            RecipeIngredientMessage::Delete => (),
        }
    }

    pub fn view(&mut self) -> Element<RecipeIngredientMessage> {
        let theme = crate::theme();
        let ingredient_list = iced_searchable_picklist::PickList::new(
            &mut self.ingredient_list,
            self.filtered_ingredients.as_ref().unwrap_or(&*self.all_ingredients),
            Some(self.entry.ingredient.clone()),
            RecipeIngredientMessage::PickIngredient,
            RecipeIngredientMessage::FilterChanged,
            &self.ingredient_filter,
        )
        .on_submit(RecipeIngredientMessage::SubmitFilter)
        .on_focus(RecipeIngredientMessage::Focus)
        .width(Length::FillPortion(3))
        .text_style(theme)
        .style(theme)
        .padding(10);

        let text_theme = match self.amount_valid {
            true => style::TextInput::Normal,
            false => style::TextInput::Error,
        };
        let amount_input = TextInput::new(
            &mut self.amount,
            "Amount…",
            &self.amount_text,
            RecipeIngredientMessage::AmountChanged,
        )
        .on_submit(RecipeIngredientMessage::SubmitAmount)
        .width(Length::Units(60))
        .style(text_theme)
        .padding(10);

        let unit_list = match self.entry.ingredient {
            RecipeMetaIngredient::MetaRecipe(_) => &[Unit::KG][..],
            _ => &self.all_units[..],
        };

        let unit_list = iced::PickList::new(
            &mut self.unit_list,
            unit_list,
            Some(self.entry.unit.clone()),
            RecipeIngredientMessage::PickUnit,
        )
        .padding(10)
        .style(theme)
        .width(Length::Shrink);

        let delete_button = Button::new(
            &mut self.delete_button,
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(RecipeIngredientMessage::Delete)
        .padding(10)
        .style(Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(ingredient_list)
            .push(amount_input)
            .push(unit_list)
            .push(delete_button)
            .into()
    }
}
