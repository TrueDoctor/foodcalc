use tui::widgets::TableState;

use super::db::{FoodBase, Ingredient};

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
}

#[derive(Clone)]
pub enum AppState {
    Init,
    IngredientView {
        popup: Option<PopUp>,
        ingredients: Vec<Ingredient>,
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
        if let Self::IngredientView {
            selection,
            ingredients,
            ..
        } = self
        {
            let i = match selection.selected() {
                Some(i) => i % (ingredients.len() - 1) + 1,
                None => 1,
            };
            selection.select(Some(i));
        }
    }

    pub(crate) fn previous_item(&mut self) {
        if let Self::IngredientView {
            selection,
            ingredients,
            ..
        } = self
        {
            let i = match selection.selected() {
                Some(i) => (i + ingredients.len() - 3) % (ingredients.len() - 1) + 1,
                None => 1,
            };
            selection.select(Some(i));
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

    pub(crate) fn popup(&self) -> Option<&PopUp> {
        if let Self::IngredientView { ref popup, .. } = self {
            return popup.as_ref();
        }
        None
    }

    pub(crate) async fn update(&mut self, database: &FoodBase) {
        if let Self::IngredientView { ingredients, .. } = self {
            *ingredients = database.get_ingredients().await.unwrap_or_default();
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}
