use std::sync::Arc;

use bigdecimal::BigDecimal;
use iced::widget::text_input::Id;
use iced::{widget::*, Command};
use iced::{Alignment, Element, Length};
use num::Num;

use crate::app::ui::Icon;
use crate::db::{RecipeIngredient, RecipeMetaIngredient, Unit};

#[derive(Debug, Clone)]
pub struct RecipeIngredientWrapper {
    pub(crate) entry: RecipeIngredient,
    all_ingredients: Arc<Vec<RecipeMetaIngredient>>,
    all_units: Arc<Vec<Unit>>,
    filtered_ingredients: Option<Vec<RecipeMetaIngredient>>,
    ingredient_filter: String,
    amount_text: String,
    amount_valid: bool,
    searchable_list_id: Id,
}
impl Default for RecipeIngredientWrapper {
    fn default() -> Self {
        Self {
            entry: RecipeIngredient::default(),
            all_ingredients: Arc::new(Vec::new()),
            all_units: Arc::new(Vec::new()),
            filtered_ingredients: None,
            ingredient_filter: String::new(),
            amount_text: String::new(),
            amount_valid: true,
            searchable_list_id: Id::unique(),
        }
    }
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
    pub fn new(
        ingredients: Arc<Vec<RecipeMetaIngredient>>,
        all_units: Arc<Vec<Unit>>,
        entry: RecipeIngredient,
    ) -> Self {
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

    pub fn update(&mut self, message: RecipeIngredientMessage) -> Command<()> {
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
                self.update(RecipeIngredientMessage::SubmitAmount);
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
                    self.entry.ingredient = elem.clone();
                }
            },
            RecipeIngredientMessage::Focus => return text_input::focus(self.searchable_list_id.clone()),
            RecipeIngredientMessage::Unfocus => (),
            RecipeIngredientMessage::Delete => (),
        };
        Command::none()
    }

    pub fn view(&self) -> Element<RecipeIngredientMessage> {
        let ingredient_list = iced_searchable_picklist::PickList::new(
            self.filtered_ingredients.as_ref().unwrap_or(&*self.all_ingredients),
            Some(self.entry.ingredient.clone()),
            RecipeIngredientMessage::PickIngredient,
            RecipeIngredientMessage::FilterChanged,
            &self.ingredient_filter,
        )
        .on_submit(RecipeIngredientMessage::SubmitFilter)
        .on_focus(RecipeIngredientMessage::Focus)
        .width(Length::FillPortion(3))
        .padding(10);

        let amount_input = TextInput::new("Amount…", &self.amount_text, RecipeIngredientMessage::AmountChanged)
            .on_submit(RecipeIngredientMessage::SubmitAmount)
            .width(Length::Units(60))
            .padding(10);

        let unit_list = match self.entry.ingredient {
            RecipeMetaIngredient::MetaRecipe(_) => &[Unit::KG][..],
            _ => &self.all_units[..],
        };

        let unit_list = PickList::new(
            unit_list,
            Some(self.entry.unit.clone()),
            RecipeIngredientMessage::PickUnit,
        )
        .padding(10)
        .width(Length::Shrink);

        let delete_button = Button::new(
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(RecipeIngredientMessage::Delete)
        .padding(10)
        .style(iced::theme::Button::Destructive);

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
