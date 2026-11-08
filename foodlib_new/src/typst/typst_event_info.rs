use std::alloc::Layout;
use crate::entities::allergen::DietaryFlags;
use crate::ops::export::EventAllergenInfo;
use serde::Serialize;
use time::format_description;
use typst::foundations::{Dict, IntoValue};

#[derive(Serialize, Debug, Clone)]
pub struct TypstMealAllergen {
    pub name: String,
    pub place: String,
    pub time: String,
    pub servings: i32,
    pub dietary: DietaryFlags,
    pub contains: Vec<String>,
    pub may_contain: Vec<String>,
    pub hinweise: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TypstDishLabel {
    pub event_name: String,
    pub meals: Vec<TypstMealAllergen>,
}

impl TypstDishLabel {
    pub fn from_event_info(info: &EventAllergenInfo) -> Self {
        let format = format_description::parse("[day].[month]. [hour]:[minute]").unwrap();

        let meals = info.meals.iter().map(|m| {
            TypstMealAllergen {
                name: m.recipe_name.clone(),
                place: m.place.clone(),
                time: m.start_time.format(&format).unwrap_or_default(),
                servings: m.servings,
                dietary: m.dietary.clone(),
                contains: m.properties.iter().map(|p| p.name.clone()).collect(),
                may_contain: vec![],
                hinweise: vec![],
            }
        }).collect();

        Self {
            event_name: info.event_name.clone(),
            meals,
        }
    }
}

// Dish label layout options
#[derive(Debug, Clone)]
pub struct DishLabelLayoutInput {
    pub v: Vec<TypstDishLabel>,
    pub layout: String
}

// Top-level dictionary wrapper that provides `inputs.v`
impl From<DishLabelLayoutInput> for Dict {
    fn from(val: DishLabelLayoutInput) -> Self {
        let mut dict = Dict::new();
        dict.insert("v".into(), val.v.into_value());
        dict.insert("layout".into(), val.layout.into_value());
        dict
    }
}

impl IntoValue for TypstDishLabel {
    fn into_value(self) -> typst::foundations::Value {
        let mut dict = Dict::new();
        dict.insert("event_name".into(), self.event_name.into_value());
        dict.insert("meals".into(), self.meals.into_value());
        dict.into_value()
    }
}

impl IntoValue for TypstMealAllergen {
    fn into_value(self) -> typst::foundations::Value {
        let mut dict = Dict::new();
        dict.insert("name".into(), self.name.into_value());
        dict.insert("place".into(), self.place.into_value());
        dict.insert("time".into(), self.time.into_value());
        dict.insert("servings".into(), self.servings.into_value());
        dict.insert("dietary".into(), self.dietary.into_value());
        dict.insert("contains".into(), self.contains.into_value());
        dict.insert("may_contain".into(), self.may_contain.into_value());
        dict.insert("hinweise".into(), self.hinweise.into_value());
        dict.into_value()
    }
}

impl IntoValue for DietaryFlags {
    fn into_value(self) -> typst::foundations::Value {
        let mut dict = Dict::new();
        dict.insert("vegan".into(), self.vegan.into_value());
        dict.insert("vegetarian".into(), self.vegetarian.into_value());
        dict.insert("gluten_free".into(), self.gluten_free.into_value());
        dict.insert("lactose_free".into(), self.lactose_free.into_value());
        // Trace flags
        dict.insert("vegan_trace".into(), self.vegan_trace.into_value());
        dict.insert("vegetarian_trace".into(), self.vegetarian_trace.into_value());
        dict.insert("gluten_free_trace".into(), self.gluten_free_trace.into_value());
        dict.insert("lactose_free_trace".into(), self.lactose_free_trace.into_value());
        dict.into_value()
    }
}