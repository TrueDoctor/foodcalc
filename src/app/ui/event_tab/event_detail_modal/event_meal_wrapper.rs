use std::sync::Arc;

use iced::{text_input, button, Element, Length, TextInput, Button, Row, Text, Alignment};

use crate::{db::{Meal, Recipe}, app::ui::Icon};

use crate::app::ui::style::Button::Destructive;

#[derive(Debug, Clone)]
pub struct MealWrapper {
    pub(crate) meal: Meal,
    servings: text_input::State,
    energy: text_input::State,
    all_recipes: Arc<Vec<Recipe>>,
    recipe_list: iced_searchable_picklist::State<Recipe>,
    filtered_recipes: Option<Vec<Recipe>>,
    recipe_filter: String,
    servings_text: String,
    energy_text: String,
    delete_button: button::State
}

#[derive(Clone, Debug)]
pub enum MealMessage {
    FilterChanged(String),
    ServingsChanged(String),
    EnergyChanged(String),
    PickRecipe(Recipe),
    Submit,
    Focus,
    Unfocus,
    SubmitFilter,
    Delete,
}

impl MealWrapper {
    pub fn new(meal: Meal, recipes: Arc<Vec<Recipe>>,) -> Self {
        Self {
            meal: meal.clone(),
            all_recipes: recipes,
            servings_text: meal.clone().servings.to_string(),
            energy_text: meal.clone().energy.to_string(),
            recipe_list: Default::default(),
            filtered_recipes: None,
            recipe_filter: "".to_string(),
            servings: Default::default(),
            energy: Default::default(),
            delete_button: Default::default(),
        }
    }

    pub fn update(&mut self, message: MealMessage) {
        match message {
            MealMessage::FilterChanged(name) => {
                self.recipe_filter = name;
                self.filtered_recipes = (!self.recipe_filter.is_empty()).then(|| {
                    self.all_recipes
                        .iter()
                        .filter(|recipe| crate::similar(&recipe.name, &self.recipe_filter))
                        .cloned()
                        .collect::<Vec<_>>()
                })
            },
            MealMessage::ServingsChanged(amount) => self.servings_text = amount,
            MealMessage::EnergyChanged(amount) => self.energy_text = amount,
            MealMessage::PickRecipe(recipe) => self.meal.recipe_id = recipe.recipe_id,
            MealMessage::Submit => {

            },
            MealMessage::Focus => self.recipe_list.focus(),
            MealMessage::Unfocus => self.recipe_list.unfocus(),
            MealMessage::SubmitFilter => {
                if let Some([recipe]) = self.filtered_recipes.as_deref() {
                    self.recipe_list.unfocus();
                    self.meal.recipe_id = recipe.recipe_id;
                }
            },
            MealMessage::Delete => (),
            
        }
    }

    pub fn view(&mut self) -> Element<MealMessage>{
        let theme = crate::theme();
        let recipe_list = iced_searchable_picklist::PickList::new(
            &mut self.recipe_list, 
            self.filtered_recipes.as_ref().unwrap_or(&*self.all_recipes), 
            Some(self.all_recipes
                .clone()
                .to_vec()
                .into_iter()
                .filter(|recipe| recipe.recipe_id == self.meal.recipe_id)
                .next()
                .unwrap_or(Default::default())), 
                MealMessage::PickRecipe, 
                MealMessage::FilterChanged, 
                &self.recipe_filter
            )
            .on_submit(MealMessage::SubmitFilter)
            .on_focus(MealMessage::Focus)
            .width(Length::FillPortion(3))
            .text_style(theme)
            .style(theme)
            .padding(10);

        let servings_input = TextInput::new(
            &mut self.servings, 
            "Servings…", 
            &self.servings_text, 
            MealMessage::ServingsChanged
        )
        .on_submit(MealMessage::Submit)
        .width(Length::Units(60))
        .padding(10);

        let energy_input = TextInput::new(
            &mut self.energy, 
            "Energy", 
            &self.energy_text, 
            MealMessage::EnergyChanged
        )
        .on_submit(MealMessage::Submit)
        .width(Length::Units(60))
        .padding(10);

        let delete_button = Button::new(
            &mut self.delete_button,
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        ).on_press(MealMessage::Delete)
        .padding(10)
        .style(Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(recipe_list)
            .push(servings_input)
            .push(energy_input)
            .push(delete_button)
            .into()
    }
}