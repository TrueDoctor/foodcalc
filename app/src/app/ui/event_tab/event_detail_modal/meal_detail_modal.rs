use std::sync::Arc;

use iced::alignment::Horizontal;
use iced::widget::*;
use iced::{Alignment, Command, Element, Length};
use num::Zero;
use sqlx::types::BigDecimal;

use super::EventDetailMessage;
use crate::app::ui::util::{DateInput, InputState, OptionString};
use crate::db::{FoodBase, Meal, Place, Recipe};

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

    start_time: InputState<DateInput>,
    end_time: InputState<DateInput>,
    servings: InputState<i32>,
    energy: InputState<BigDecimal>,
    comment: InputState<OptionString>,

    database: Arc<FoodBase>,
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
    FocusRecipe,
    FocusPlace,
    Unfocus,
    SubmitFilter,
    Delete,
    Cancel,
    Save,
}

impl MealDetail {
    pub fn new(
        meal: Option<Meal>,
        all_recipes: Arc<Vec<Recipe>>,
        all_places: Arc<Vec<Place>>,
        database: Arc<FoodBase>,
        event_id: i32,
    ) -> Self {
        let mut new_meal = meal.clone().unwrap_or_default();
        new_meal.event_id = event_id;
        let start_time = new_meal.start_time.to_string();
        let end_time = new_meal.end_time.to_string();
        Self {
            new_meal: new_meal.clone(),
            old_meal: meal,
            all_recipes,
            all_places,
            filtered_recipes: None,
            filtered_places: None,
            recipe_filter: Default::default(),
            place_filter: Default::default(),
            start_time: InputState::new(start_time[..(start_time.len() - 3)].to_string()),
            end_time: InputState::new(end_time[..(end_time.len() - 3)].to_string()),
            servings: InputState::new(new_meal.servings.to_string()),
            energy: InputState::new(new_meal.energy.to_string()),
            comment: InputState::new(new_meal.comment.unwrap_or_default()),
            database,
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
                // TODO:

                //self.recipe_picker.focus();
            },
            MealDetailMessage::FocusPlace => {
                // TODO:
                //self.place_picker.focus();
            },
            MealDetailMessage::Unfocus => {},
            MealDetailMessage::SubmitFilter => {
                if let Some([recipe]) = self.filtered_recipes.as_deref() {
                    self.new_meal.recipe_id = recipe.recipe_id;
                }
                if let Some([place]) = self.filtered_places.as_deref() {
                    self.new_meal.place_id = place.place_id;
                }
            },
            MealDetailMessage::Delete => (),
            MealDetailMessage::ValueChanged(field, s) => match field {
                InputField::StartTime => self.start_time.update(s),
                InputField::EndTime => self.end_time.update(s),
                InputField::Servings => self.servings.update(s),
                InputField::Energy => self.energy.update(s),
                InputField::Comment => self.comment.update(s),
            },
            MealDetailMessage::Cancel => {
                println!("Cancel");
                return Command::perform(async { Ok(()) }, EventDetailMessage::CloseModal);
            },
            MealDetailMessage::Save => {
                let move_database = self.database.clone();

                let old_meal = self.old_meal.clone();
                if [
                    dbg!(self.start_time.valid()),
                    dbg!(self.end_time.valid()),
                    dbg!(self.comment.valid()),
                    dbg!(self.energy.valid() && self.energy.value_type.as_ref().unwrap() > &BigDecimal::zero()),
                    dbg!(self.servings.valid() && self.servings.value_type.unwrap() > 0),
                ]
                .iter()
                .all(|input| *input)
                {
                    self.new_meal.start_time = self.start_time.value_type.clone().unwrap().0;
                    self.new_meal.end_time = self.end_time.value_type.clone().unwrap().0;
                    self.new_meal.servings = self.servings.value_type.unwrap();
                    self.new_meal.energy = self.energy.value_type.clone().unwrap();
                    self.new_meal.comment = self.comment.value_type.clone().unwrap().0;
                    let meal = self.new_meal.clone();

                    return Command::perform(
                        async move {
                            log::trace!("Saving meal: {:?}", meal);
                            if let Some(old) = old_meal {
                                move_database.update_single_meal(
                                    old.meal_id,
                                    meal.recipe_id,
                                    meal.place_id,
                                    meal.start_time,
                                    meal.end_time,
                                    meal.energy,
                                    meal.servings,
                                    meal.comment,
                                );
                            } else {
                                move_database
                                    .add_meal(
                                        meal.event_id,
                                        meal.recipe_id,
                                        meal.place_id,
                                        meal.start_time,
                                        meal.end_time,
                                        meal.energy,
                                        meal.servings,
                                        meal.comment,
                                    )
                                    .await?;
                            }
                            Ok(())
                        },
                        EventDetailMessage::CloseModal,
                    );
                } else {
                    log::error!("Invalid input {:#?}", self.new_meal);
                }
            },
        }
        Command::none()
    }

    pub fn view(&self) -> Element<MealDetailMessage> {
        let selected_recipe = self
            .all_recipes
            .iter()
            .find(|recipe| recipe.recipe_id == self.new_meal.recipe_id)
            .cloned();
        let recipe_list = iced_searchable_picklist::PickList::new(
            self.filtered_recipes.as_ref().unwrap_or(&*self.all_recipes),
            selected_recipe,
            MealDetailMessage::PickRecipe,
            MealDetailMessage::RecipeFilterChanged,
            &self.recipe_filter,
        )
        .on_submit(MealDetailMessage::SubmitFilter)
        .on_focus(MealDetailMessage::FocusRecipe)
        .width(Length::FillPortion(3))
        .padding(10);

        let selected_place = self
            .all_places
            .iter()
            .find(|place| place.place_id == self.new_meal.place_id)
            .cloned();
        let place_list = iced_searchable_picklist::PickList::new(
            self.filtered_places.as_ref().unwrap_or(&*self.all_places),
            selected_place,
            MealDetailMessage::PickPlace,
            MealDetailMessage::PlaceFilterChanged,
            &self.place_filter,
        )
        .on_submit(MealDetailMessage::SubmitFilter)
        .on_focus(MealDetailMessage::FocusPlace)
        .width(Length::FillPortion(3))
        .padding(10);

        let _text_theme = self.start_time.text_color();
        let start_input = TextInput::new("Start Time…", &self.start_time.value, |value| {
            MealDetailMessage::ValueChanged(InputField::StartTime, value)
        })
        .width(Length::FillPortion(1))
        .padding(10);

        let _text_theme = self.end_time.text_color();

        let end_input = TextInput::new("End Time…", &self.end_time.value, |value| {
            MealDetailMessage::ValueChanged(InputField::EndTime, value)
        })
        .width(Length::FillPortion(1))
        .padding(10);

        let _text_theme = self.comment.text_color();
        let comment_input = TextInput::new("Comment…", &self.comment.value, |value| {
            MealDetailMessage::ValueChanged(InputField::Comment, value)
        })
        .width(Length::Fill)
        .padding(10);

        let _text_theme = self.servings.text_color();
        let servings_input = TextInput::new("Servings…", &self.servings.value, |value| {
            MealDetailMessage::ValueChanged(InputField::Servings, value)
        })
        .width(Length::FillPortion(1))
        .padding(10);

        let _text_theme = self.energy.text_color();

        let energy_input = TextInput::new("Energy…", &self.energy.value, |value| {
            MealDetailMessage::ValueChanged(InputField::Energy, value)
        })
        .width(Length::FillPortion(1))
        .padding(10);

        let cancel_button = Button::new(Text::new("Cancel").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
            .on_press(MealDetailMessage::Cancel);

        let ok_button = Button::new(Text::new("Save").horizontal_alignment(Horizontal::Center))
            .width(Length::Fill)
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
            .max_width(800)
            .align_items(Alignment::Center)
            .push(row1)
            .push(row2)
            .push(row3)
            .push(row4)
            .push(row5)
            .into()
    }
}
