use bigdecimal::ToPrimitive;
use foodlib_new::allergen::DietaryFlags;
use foodlib_new::ops::export::{EventAllergenInfo, RecipeInfo};
use serde::Serialize;
use time::format_description;
use typst::foundations::{Array, Dict, IntoValue, Value};

#[derive(Debug, Clone, Copy, Default)]
pub enum DishLabelLayout {
    #[default]
    Flat,
    Tent,
}
use typst::foundations::{Array, Dict, IntoValue, Value};
#[derive(Serialize, Debug, Clone)]
pub struct TypstRecipePage {
    pub name: String,
    pub date: String,
    pub ingredients: TypstIngredients,
    pub steps: Vec<TypstStep>,
    pub allergens: TypstAllergens,
    pub prep_for: Option<Vec<(String, f64)>>,
    pub weighed_in: bool,
    // Provide an empty array for now as water boil calculations are inline in Typst
    pub water: Vec<()>,
    pub root: Option<String>,
}
#[derive(Serialize, Debug, Clone)]
pub struct TypstIngredients {
    // Array of (name, weight_in_kg)
    pub ingredients: Vec<(String, f64)>,
    // Array of (name, weight_in_kg, is_from_prep)
    pub subrecipes: Vec<(String, f64, bool)>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TypstStep {
    pub title: String,
    pub desc: String,
    pub duration: TypstDuration,
}

#[derive(Serialize, Debug, Clone)]
pub struct TypstDuration {
    pub fix: f64, // Minutes
    pub var: f64, // Minutes
}

#[derive(Serialize, Debug, Clone)]
pub struct TypstAllergens {
    pub dietary: DietaryFlags,
    pub contains: Vec<String>,
    pub may_contain: Vec<String>,
    pub hinweise: Vec<String>,
}

impl TypstRecipePage {
    /// Flattens a `RecipeInfo` into an array of individual printable Typst pages.
    pub fn from_recipe_info(info: &RecipeInfo) -> Vec<Self> {
        let mut pages = Vec::with_capacity(info.subrecipes.len());
        let prepped_ids = &info.prepped_subrecipe_ids;

        for sub in &info.subrecipes {
            // Split ingredients and subrecipes as Typst expects them
            let mut leaf_ingredients = Vec::new();
            let mut meta_ingredients = Vec::new();

            for ing in &sub.ingredients {
                let weight = ing.weight.to_f64().unwrap_or(0.0);
                if ing.is_subrecipe {
                    let from_prep = prepped_ids.contains(&ing.subrecipe_id);
                    meta_ingredients.push((ing.subrecipe.clone(), weight, from_prep));
                } else {
                    leaf_ingredients.push((ing.ingredient.clone(), weight));
                }
            }

            // Convert intervals from microseconds to minutes
            let steps = sub
                .steps
                .iter()
                .map(|s| TypstStep {
                    title: s.name.clone(),
                    desc: s.description.clone(),
                    duration: TypstDuration {
                        fix: s.fixed_duration / 60.0,
                        var: s.duration_per_kg / 60.0,
                    },
                })
                .collect();

            // Map properties to string arrays for Typst
            let allergens = TypstAllergens {
                dietary: sub.dietary.clone(),
                contains: sub.properties.iter().map(|p| p.name.clone()).collect(),
                may_contain: vec![], // Populate if your DB maps trace properties
                hinweise: vec![],    // Populate if your DB maps non-EU allergen hints
            };

            pages.push(TypstRecipePage {
                name: info.name.clone(),
                date: info.date.clone(),
                ingredients: TypstIngredients {
                    ingredients: leaf_ingredients,
                    subrecipes: meta_ingredients,
                },
                steps,
                allergens,
                prep_for: if info.is_prep && !info.prep_for.is_empty() {
                    Some(info.prep_for.clone())
                } else {
                    None
                },
                weighed_in: info.weighed_in,
                water: vec![],
                root: Some(sub.root_recipe.clone()),
            });
        }
        pages
    }
}

// Typst expects a top-level dictionary with a single key "v" that contains the array of pages

#[derive(Debug, Clone)]
pub struct RecipePagesInput {
    pub v: Vec<TypstRecipePage>,
}

// Top-level dictionary wrapper that provides `inputs.v`
impl From<RecipePagesInput> for Dict {
    fn from(val: RecipePagesInput) -> Self {
        let mut dict = Dict::new();
        dict.insert("v".into(), val.v.into_value());
        dict
    }
}

// Implement IntoValue for the rest of your view models
impl IntoValue for TypstRecipePage {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        dict.insert("name".into(), self.name.into_value());
        dict.insert("date".into(), self.date.into_value());
        dict.insert("ingredients".into(), self.ingredients.into_value());
        dict.insert("steps".into(), self.steps.into_value());
        dict.insert("allergens".into(), self.allergens.into_value());

        // Handle Optionals
        if let Some(prep) = self.prep_for {
            let mut prep_arr = Array::new();
            for (name, pct) in prep {
                let mut tuple = Array::new();
                tuple.push(name.into_value());
                tuple.push(pct.into_value());
                prep_arr.push(Value::Array(tuple));
            }
            dict.insert("prep_for".into(), Value::Array(prep_arr));
        } else {
            dict.insert("prep_for".into(), Value::None);
        }

        dict.insert("weighed_in".into(), self.weighed_in.into_value());
        dict.insert("water".into(), Array::new().into_value());

        if let Some(root) = self.root {
            dict.insert("root".into(), root.into_value());
        } else {
            dict.insert("root".into(), Value::None);
        }

        Value::Dict(dict)
    }
}

impl IntoValue for TypstIngredients {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();

        // Typst expects an array of tuples (arrays in Typst): ((name, amount))
        let mut ing_array = Array::new();
        for (name, weight) in self.ingredients {
            let mut tuple = Array::new();
            tuple.push(name.into_value());
            tuple.push(weight.into_value());
            ing_array.push(Value::Array(tuple));
        }
        dict.insert("ingredients".into(), Value::Array(ing_array));

        // Typst expects ((name, amount, from_prep))
        let mut sub_array = Array::new();
        for (name, weight, is_prep) in self.subrecipes {
            let mut tuple = Array::new();
            tuple.push(name.into_value());
            tuple.push(weight.into_value());
            tuple.push(is_prep.into_value());
            sub_array.push(Value::Array(tuple));
        }
        dict.insert("subrecipes".into(), Value::Array(sub_array));

        Value::Dict(dict)
    }
}

impl IntoValue for TypstStep {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        dict.insert("title".into(), self.title.into_value());
        dict.insert("desc".into(), self.desc.into_value());
        dict.insert("duration".into(), self.duration.into_value());
        Value::Dict(dict)
    }
}

impl IntoValue for TypstDuration {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        dict.insert("fix".into(), self.fix.into_value());
        dict.insert("var".into(), self.var.into_value());
        Value::Dict(dict)
    }
}

impl IntoValue for TypstAllergens {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();

        let mut dietary = Dict::new();
        dietary.insert("vegan".into(), self.dietary.vegan.into_value());
        dietary.insert("vegan_trace".into(), self.dietary.vegan_trace.into_value());
        dietary.insert("vegetarian".into(), self.dietary.vegetarian.into_value());
        dietary.insert(
            "vegetarian_trace".into(),
            self.dietary.vegetarian_trace.into_value(),
        );
        dietary.insert(
            "lactose_free".into(),
            self.dietary.lactose_free.into_value(),
        );
        dietary.insert(
            "lactose_free_trace".into(),
            self.dietary.lactose_free_trace.into_value(),
        );
        dietary.insert("gluten_free".into(), self.dietary.gluten_free.into_value());
        dietary.insert(
            "gluten_free_trace".into(),
            self.dietary.gluten_free_trace.into_value(),
        );

        dict.insert("dietary".into(), Value::Dict(dietary));
        dict.insert("contains".into(), self.contains.into_value());
        dict.insert("may_contain".into(), self.may_contain.into_value());
        dict.insert("hinweise".into(), self.hinweise.into_value());

        Value::Dict(dict)
    }
}

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

        let meals = info
            .meals
            .iter()
            .map(|m| TypstMealAllergen {
                name: m.recipe_name.clone(),
                place: m.place.clone(),
                time: m.start_time.format(&format).unwrap_or_default(),
                servings: m.servings,
                dietary: m.dietary.clone(),
                contains: m.properties.iter().map(|p| p.name.clone()).collect(),
                may_contain: vec![],
                hinweise: vec![],
            })
            .collect();

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
    pub layout: String,
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
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        dict.insert("event_name".into(), self.event_name.into_value());
        dict.insert("meals".into(), self.meals.into_value());
        dict.into_value()
    }
}

impl IntoValue for TypstMealAllergen {
    fn into_value(self) -> Value {
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
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        dict.insert("vegan".into(), self.vegan.into_value());
        dict.insert("vegetarian".into(), self.vegetarian.into_value());
        dict.insert("gluten_free".into(), self.gluten_free.into_value());
        dict.insert("lactose_free".into(), self.lactose_free.into_value());
        // Trace flags
        dict.insert("vegan_trace".into(), self.vegan_trace.into_value());
        dict.insert(
            "vegetarian_trace".into(),
            self.vegetarian_trace.into_value(),
        );
        dict.insert(
            "gluten_free_trace".into(),
            self.gluten_free_trace.into_value(),
        );
        dict.insert(
            "lactose_free_trace".into(),
            self.lactose_free_trace.into_value(),
        );
        dict.into_value()
    }
}
