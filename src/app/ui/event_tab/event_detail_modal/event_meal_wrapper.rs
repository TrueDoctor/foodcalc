use std::sync::Arc;

use iced::{button, Alignment, Button, Element, Row, Text, Command};


use crate::{
    app::{ui::{Icon, style}},
    db::{Meal, Recipe, Place},
};

use crate::app::ui::style::Button::Destructive;

use super::EventDetailMessage;

#[derive(Debug, Clone)]
pub struct MealWrapper {
    pub(crate) meal: Meal,
    all_recipes: Arc<Vec<Recipe>>,
    all_places: Arc<Vec<Place>>,
    recipe_list: iced_searchable_picklist::State<Recipe>,
    filtered_recipes: Option<Vec<Recipe>>,
    recipe_filter: String,
    servings_text: String,
    energy_text: String,
    delete_button: button::State,
    edit_button: button::State,
}

#[derive(Clone, Debug)]
pub enum MealMessage {
    FilterChanged(String),
    ServingsChanged(String),
    EnergyChanged(String),
    PickRecipe(Recipe),
    OpenModal(Meal),
    Submit,
    Focus,
    Unfocus,
    SubmitFilter,
    Delete,
}

impl MealWrapper {
    pub fn new(meal: Meal) -> Self {
        Self {
            meal: meal.clone(),
            all_recipes: Default::default(),
            all_places: Default::default(),
            servings_text: meal.clone().servings.to_string(),
            energy_text: meal.clone().energy.to_string(),
            recipe_list: Default::default(),
            filtered_recipes: None,
            recipe_filter: "".to_string(),
            delete_button: Default::default(),
            edit_button: Default::default(),
        }
    }

    pub fn update(&mut self, message: MealMessage) -> Command<EventDetailMessage> {
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
            MealMessage::Submit => {},
            MealMessage::Focus => self.recipe_list.focus(),
            MealMessage::Unfocus => self.recipe_list.unfocus(),
            MealMessage::SubmitFilter => {
                if let Some([recipe]) = self.filtered_recipes.as_deref() {
                    self.recipe_list.unfocus();
                    self.meal.recipe_id = recipe.recipe_id;
                }
            },
            // handeled in outer event updae
            MealMessage::OpenModal(_) => (),
            MealMessage::Delete => (),
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<MealMessage> {
        let theme = crate::theme();
        
        let recipe = iced::Text::new(
            self.all_recipes
                .iter()
                .find(|recipe| recipe.recipe_id == self.meal.recipe_id)
                .cloned()
                .unwrap_or(Default::default())
                .name,
        )
        .color(theme.foreground());


        let place = iced::Text::new(
            self.all_places
                .iter()
                .find(|place| place.place_id == self.meal.place_id)
                .cloned()
                .unwrap_or(Default::default())
                .name,
        )
        .color(theme.foreground());

        let start = iced::Text::new(self.meal.start_time.to_string()).color(theme.foreground());

        let delete_button = Button::new(
            &mut self.delete_button,
            Row::new()
                .spacing(10)
                .push(Icon::Delete.text())
                .push(Text::new("Delete")),
        )
        .on_press(MealMessage::Delete)
        .padding(10)
        .style(Destructive);

        Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(recipe)
            .push(place)
            .push(start)
            .push(
                Button::new(&mut self.edit_button, Icon::Edit.text())
                    .on_press(MealMessage::OpenModal(self.meal.clone()))
                    .padding(10)
                    .style(style::Button::Icon),
            )
            .push(delete_button)
            .into()
    }
}
