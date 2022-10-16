use std::sync::Arc;

use iced::{alignment::Horizontal, button, Alignment, Button, Column, Command, Element, Length, Row, Text, TextInput};
use sqlx::types::time::PrimitiveDateTime;

use crate::db::{Meal, Place, Recipe, FoodBase};

use super::EventDetailMessage;

#[derive(Debug, Clone, Default)]
pub struct InputState {
    state: iced::text_input::State,
    valid: bool,
    value: String,
}

#[derive(Debug, Clone)]
pub struct MealDetail {
    pub(crate) new_meal: Meal,
    old_meal: Option<Meal>,
    all_recipes: Arc<Vec<Recipe>>,
    all_places: Arc<Vec<Place>>,
    filtered_recipes: Option<Vec<Recipe>>,
    filtered_places: Option<Vec<Place>>,
    recipe_filter: String,
    place_filter: String,

    recipe_picker: iced_searchable_picklist::State<Recipe>,
    place_picker: iced_searchable_picklist::State<Place>,
    start_time: InputState,
    end_time: InputState,
    servings: InputState,
    energy: InputState,
    comment: InputState,

    pub(crate) ok_state: button::State,
    pub(crate) cancel_state: button::State,
    database: Arc<FoodBase>
}

#[derive(Debug, Clone)]
pub enum InputField {
    StartTime,
    EndTime,
    Servings,
    Energy,
    Comment,
}

#[derive(Debug, Clone)]
pub enum MealDetailMessage {
    PickRecipe(Recipe),
    PickPlace(Place),
    RecipeFilterChanged(String),
    PlaceFilterChanged(String),
    ValueChanged(InputField, String),
    SubmitValue(InputField),
    FocusRecipe,
    FocusPlace,
    Unfocus,
    SubmitFilter,
    Delete,
    Cancel,
    Save,
}

impl MealDetail {
    pub fn new(meal: Option<Meal>, all_recipes: Arc<Vec<Recipe>>, all_places: Arc<Vec<Place>>, database: Arc<FoodBase>) -> Self {
        Self {
            new_meal: meal.clone().unwrap_or_default(),
            old_meal: meal.clone(),
            all_recipes: all_recipes.clone(),
            all_places: all_places.clone(),
            filtered_recipes: None,
            filtered_places: None,
            recipe_filter: Default::default(),
            place_filter: Default::default(),
            recipe_picker: Default::default(),
            place_picker: Default::default(),
            start_time: InputState {
                value: meal.clone().unwrap_or_default().start_time.to_string(),
                valid: true,
                ..Default::default()
            },
            end_time: InputState {
                value: meal.clone().unwrap_or_default().end_time.to_string(),
                valid: true,
                ..Default::default()
            },
            servings: InputState {
                value: meal.clone().unwrap_or_default().servings.to_string(),
                valid: true,
                ..Default::default()
            },
            energy: InputState {
                value: meal.clone().unwrap_or_default().energy.to_string(),
                valid: true,
                ..Default::default()
            },
            comment: InputState {
                value: meal.clone().unwrap_or_default().comment.unwrap_or_default(),
                valid: true,
                ..Default::default()
            },
            ok_state: Default::default(),
            cancel_state: Default::default(),
            database
        }
    }

    pub fn update(&mut self, message: MealDetailMessage) -> Command<EventDetailMessage> {
        match message {
            MealDetailMessage::PickRecipe(recipe) => self.new_meal.recipe_id = recipe.recipe_id,
            MealDetailMessage::PickPlace(place) => self.new_meal.place_id = place.place_id,
            MealDetailMessage::RecipeFilterChanged(name) => {
                self.recipe_filter = name;
                self.filtered_recipes = (!self.recipe_filter.is_empty()).then(|| {
                    self.all_recipes
                        .iter()
                        .filter(|recipe| crate::similar(&recipe.name, &self.recipe_filter))
                        .cloned()
                        .collect::<Vec<_>>()
                });
            },
            MealDetailMessage::PlaceFilterChanged(name) => {
                self.place_filter = name;
                self.filtered_places = (!self.place_filter.is_empty()).then(|| {
                    self.all_places
                        .iter()
                        .filter(|place| crate::similar(&place.name, &self.place_filter))
                        .cloned()
                        .collect::<Vec<_>>()
                });
            },
            MealDetailMessage::FocusRecipe => {
                self.place_picker.unfocus();
                self.recipe_picker.focus();
            },
            MealDetailMessage::FocusPlace => {
                self.recipe_picker.unfocus();
                self.place_picker.focus();
            },
            MealDetailMessage::Unfocus => {
                self.place_picker.unfocus();
                self.recipe_picker.unfocus();
            },
            MealDetailMessage::SubmitFilter => {
                self.place_picker.unfocus();
                self.recipe_picker.unfocus();
                if let Some([recipe]) = self.filtered_recipes.as_deref() {
                    self.new_meal.recipe_id = recipe.recipe_id;
                }
                if let Some([place]) = self.filtered_places.as_deref() {
                    self.new_meal.place_id = place.place_id;
                }
            },
            MealDetailMessage::Delete => (),
            MealDetailMessage::ValueChanged(field, s) => match field {
                InputField::StartTime => self.start_time.value = s,
                InputField::EndTime => self.end_time.value = s,
                InputField::Servings => self.servings.value = s,
                InputField::Energy => self.energy.value = s,
                InputField::Comment => self.new_meal.comment = Some(s),
            },
            MealDetailMessage::SubmitValue(input) => match input {
                InputField::StartTime => {
                    if let Ok(time) = PrimitiveDateTime::parse(&self.start_time.value, "%F %T") {
                        self.start_time.valid = true;
                        self.new_meal.start_time = time;
                    } else {
                        self.start_time.valid = false;
                    }
                },
                InputField::EndTime => {
                    if let Ok(time) = PrimitiveDateTime::parse(&self.end_time.value, "%F %T") {
                        self.end_time.valid = true;
                        self.new_meal.end_time = time;
                    } else {
                        self.servings.valid = false;
                    }
                },
                InputField::Servings => {
                    if let Ok(n) = self.servings.value.parse() {
                        self.servings.valid = true;
                        self.new_meal.servings = n;
                    } else {
                        self.servings.valid = false;
                    }
                },
                InputField::Energy => {
                    if let Ok(n) = self.servings.value.parse() {
                        self.servings.valid = true;
                        self.new_meal.servings = n;
                    } else {
                        self.servings.valid = false;
                    }
                },
                InputField::Comment => self.comment.valid = true,
            },
            MealDetailMessage::Cancel => {
                println!("Cancel");
                return Command::perform(async {}, |_| EventDetailMessage::CloseModal(Ok(())));
            },
            MealDetailMessage::Save => {
                
                let move_database = self.database.clone();
                let meal = self.new_meal.clone();
                let old_meal = self.old_meal.clone();
                return Command::perform(async move { 
                        move_database.update_single_meal(old_meal, Some(meal)).await?;
                        Ok(())
                 }, EventDetailMessage::CloseModal);
            },
        }
        Command::none()
    }

    pub fn view(&mut self) -> Element<MealDetailMessage> {
        let theme = crate::theme();

        let selected_recipe = self
            .all_recipes
            .iter()
            .find(|recipe| recipe.recipe_id == self.new_meal.recipe_id)
            .cloned();
        let recipe_list = iced_searchable_picklist::PickList::new(
            &mut self.recipe_picker,
            self.filtered_recipes.as_ref().unwrap_or(&*self.all_recipes),
            selected_recipe,
            MealDetailMessage::PickRecipe,
            MealDetailMessage::RecipeFilterChanged,
            &self.recipe_filter,
        )
        .on_submit(MealDetailMessage::SubmitFilter)
        .on_focus(MealDetailMessage::FocusRecipe)
        .width(Length::FillPortion(3))
        .text_style(theme)
        .style(theme)
        .padding(10);

        let selected_place = self
            .all_places
            .iter()
            .find(|place| place.place_id == self.new_meal.place_id)
            .cloned();
        let place_list = iced_searchable_picklist::PickList::new(
            &mut self.place_picker,
            self.filtered_places.as_ref().unwrap_or(&*self.all_places),
            selected_place,
            MealDetailMessage::PickPlace,
            MealDetailMessage::PlaceFilterChanged,
            &self.place_filter,
        )
        .on_submit(MealDetailMessage::SubmitFilter)
        .on_focus(MealDetailMessage::FocusPlace)
        .width(Length::FillPortion(3))
        .text_style(theme)
        .style(theme)
        .padding(10);

        let start_input = TextInput::new(
            &mut self.start_time.state,
            "Start Time…",
            &self.start_time.value,
            |value| MealDetailMessage::ValueChanged(InputField::StartTime, value),
        )
        .on_submit(MealDetailMessage::SubmitValue(InputField::StartTime))
        .width(Length::FillPortion(1))
        .style(theme)
        .padding(10);

        let end_input = TextInput::new(&mut self.end_time.state, "End Time…", &self.end_time.value, |value| {
            MealDetailMessage::ValueChanged(InputField::EndTime, value)
        })
        .on_submit(MealDetailMessage::SubmitValue(InputField::EndTime))
        .width(Length::FillPortion(1))
        .style(theme)
        .padding(10);

        let comment_input = TextInput::new(&mut self.comment.state, "Comment…", &self.comment.value, |value| {
            MealDetailMessage::ValueChanged(InputField::Comment, value)
        })
        .on_submit(MealDetailMessage::SubmitValue(InputField::Comment))
        .width(Length::Fill)
        .style(theme)
        .padding(10);

        let servings_input = TextInput::new(&mut self.servings.state, "Servings…", &self.servings.value, |value| {
            MealDetailMessage::ValueChanged(InputField::Servings, value)
        })
        .on_submit(MealDetailMessage::SubmitValue(InputField::StartTime))
        .width(Length::FillPortion(1))
        .style(theme)
        .padding(10);

        let energy_input = TextInput::new(&mut self.energy.state, "Energy…", &self.energy.value, |value| {
            MealDetailMessage::ValueChanged(InputField::Energy, value)
        })
        .on_submit(MealDetailMessage::SubmitValue(InputField::StartTime))
        .width(Length::FillPortion(1))
        .style(theme)
        .padding(10);

        let cancel_button = Button::new(
            &mut self.cancel_state,
            Text::new("Cancel").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(MealDetailMessage::Cancel);

        let ok_button = Button::new(
            &mut self.ok_state,
            Text::new("Save").horizontal_alignment(Horizontal::Center),
        )
        .width(Length::Fill)
        .style(theme)
        .on_press(MealDetailMessage::Save);

        let row1 = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(recipe_list)
            .push(place_list);

        let row2 = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(start_input)
            .push(end_input);

        let row3 = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(servings_input)
            .push(energy_input);

        let row4 = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(comment_input);

        let row5 = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(ok_button)
            .push(cancel_button);

        Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(row1)
            .push(row2)
            .push(row3)
            .push(row4)
            .push(row5)
            .into()
    }
}