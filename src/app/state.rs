use std::fmt::Display;

use tui::widgets::TableState;

use super::db::{FoodBase, Ingredient, Meal, RecipeIngredient};

const CURRENT_EVENT: i32 = 0;

#[derive(Clone)]
pub enum PopUp {
    Delete {
        id: isize,
    },
    AddSourceUrl {
        ingredient: Ingredient,
        url: String,
    },
    AddSourceWeight {
        ingredient: Ingredient,
        weight: String,
        url: String,
    },
    ViewMealIngredients {
        meal: Meal,
        ingredients: Vec<RecipeIngredient>,
        selection: TableState,
    },
}

#[derive(Clone)]
pub enum AppState {
    Init,
    IngredientView {
        popup: Option<PopUp>,
        ingredients: Vec<Ingredient>,
        selection: TableState,
    },
    RecipeIngredientView {
        popup: Option<PopUp>,
        meals: Vec<Meal>,
        selection: TableState,
    },
}

impl AppState {
    pub fn initialized() -> Self {
        let ingredients = Vec::new();
        let selection = TableState::default();
        let popup = None;
        Self::IngredientView {
            ingredients,
            selection,
            popup,
        }
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::IngredientView { .. })
    }

    pub(crate) fn next_item(&mut self) {
        let len = self.list_len();
        let selection = self.selection();
        match (selection, len) {
            (Some(selection), Some(len)) => {
                let i = match selection.selected() {
                    Some(i) => i % len + 1,
                    None => 1,
                };
                selection.select(Some(i));
            }
            _ => {}
        }
    }

    pub(crate) fn previous_item(&mut self) {
        let len = self.list_len();
        let selection = self.selection();
        match (selection, len) {
            (Some(selection), Some(len)) => {
                let i = match selection.selected() {
                    Some(i) => (i + len - 2) % len + 1,
                    None => 1,
                };
                selection.select(Some(i));
            }
            _ => {}
        }
    }

    pub(crate) fn ingredient(&self) -> Option<&Ingredient> {
        if let AppState::IngredientView {
            ingredients,
            selection,
            ..
        } = self
        {
            if let Some(i) = selection.selected() {
                return ingredients.get(i - 1);
            }
        }
        None
    }

    pub fn add_ingredient_source_url(&mut self) {
        if let Some(ingredient) = self.ingredient().map(Clone::clone) {
            if let AppState::IngredientView { popup, .. } = self {
                *popup = Some(PopUp::AddSourceUrl {
                    ingredient,
                    url: String::new(),
                })
            }
        }
    }

    pub fn close_popup(&mut self) {
        if let AppState::IngredientView { popup, .. } = self {
            *popup = None
        }
    }

    pub fn add_ingredient_source_weight(&mut self) {
        if let AppState::IngredientView { popup, .. } = self {
            if let Some(PopUp::AddSourceUrl { ingredient, url }) = popup.take() {
                *popup = Some(PopUp::AddSourceWeight {
                    ingredient,
                    url,
                    weight: String::new(),
                })
            }
        }
    }

    pub(crate) fn input(&mut self) -> Option<&mut String> {
        match self {
            AppState::IngredientView { ref mut popup, .. } => match popup {
                Some(PopUp::AddSourceUrl { url, .. }) => Some(url),
                Some(PopUp::AddSourceWeight { weight, .. }) => Some(weight),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn selection(&mut self) -> Option<&mut TableState> {
        match self {
            AppState::IngredientView {
                ref mut selection, ..
            } => Some(selection),
            AppState::RecipeIngredientView {
                ref mut selection, ..
            } => Some(selection),
            _ => None,
        }
    }

    pub(crate) fn list_len(&self) -> Option<usize> {
        match self {
            AppState::IngredientView { ingredients, .. } => Some(ingredients.len()),
            AppState::RecipeIngredientView { meals, .. } => Some(meals.len()),
            _ => None,
        }
    }

    pub(crate) fn popup(&self) -> Option<&PopUp> {
        if let Self::IngredientView { ref popup, .. } = self {
            return popup.as_ref();
        }
        None
    }

    pub(crate) async fn update(&mut self, database: &FoodBase) {
        match self {
            Self::IngredientView { ingredients, .. } => {
                *ingredients = database.get_ingredients().await.unwrap_or_default()
            }
            Self::RecipeIngredientView {
                popup:
                    Some(PopUp::ViewMealIngredients {
                        meal, ingredients, ..
                    }),
                ..
            } => {
                *ingredients = database
                    .get_recipe_ingredients(
                        meal.event_id,
                        meal.recipe_id,
                        meal.place_id,
                        meal.start_time,
                    )
                    .await
                    .unwrap_or_default()
            }
            Self::RecipeIngredientView { meals, .. } => {
                *meals = database
                    .get_event_meals(CURRENT_EVENT)
                    .await
                    .unwrap_or_default()
            }
            _ => (),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}
