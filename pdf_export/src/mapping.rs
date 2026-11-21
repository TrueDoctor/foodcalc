use bigdecimal::ToPrimitive;
use foodlib_new::entities::allergen::DietaryFlags;
use foodlib_new::ops::export::{EventAllergenInfo, RecipeInfo};
use time::format_description;
use typst::foundations::{Array, Dict, IntoValue, Value}; // If you still pass BigDecimal from foodlib_new

#[derive(Debug, Clone, Copy, Default)]
pub enum DishLabelLayout {
    #[default]
    Flat,
    Tent,
}

pub fn map_recipes_to_typst(recipes: &[RecipeInfo]) -> Dict {
    let mut pages = Array::new();

    for info in recipes {
        let prepped_ids = &info.prepped_subrecipe_ids;

        for sub in &info.subrecipes {
            let mut dict = Dict::new();
            dict.insert("name".into(), info.name.clone().into_value());
            dict.insert("date".into(), info.date.clone().into_value());
            dict.insert("weighed_in".into(), info.weighed_in.into_value());
            dict.insert("root".into(), sub.root_recipe.clone().into_value());
            dict.insert("water".into(), Array::new().into_value());

            // Handle Prep For
            if info.is_prep && !info.prep_for.is_empty() {
                let mut prep_arr = Array::new();
                for (name, pct) in &info.prep_for {
                    prep_arr
                        .push(Array::from_iter([name.clone().into_value(), pct.into_value()]).into_value());
                }
                dict.insert("prep_for".into(), prep_arr.into_value());
            } else {
                dict.insert("prep_for".into(), Value::None);
            }

            // Map Ingredients
            let mut leaf_ingredients = Array::new();
            let mut meta_ingredients = Array::new();
            for ing in &sub.ingredients {
                let weight = ing.weight.to_f64().unwrap_or(0.0).into_value();
                if ing.is_subrecipe {
                    let from_prep = prepped_ids.contains(&ing.subrecipe_id).into_value();
                    meta_ingredients.push(
                        Array::from_iter([ing.subrecipe.clone().into_value(), weight, from_prep])
                            .into_value(),
                    );
                } else {
                    leaf_ingredients.push(
                        Array::from_iter([ing.ingredient.clone().into_value(), weight])
                            .into_value(),
                    );
                }
            }
            let mut ing_dict = Dict::new();
            ing_dict.insert("ingredients".into(), leaf_ingredients.into_value());
            ing_dict.insert("subrecipes".into(), meta_ingredients.into_value());
            dict.insert("ingredients".into(), ing_dict.into_value());

            // Map Steps (Now using standard f64 minutes)
            let mut steps_arr = Array::new();
            for s in &sub.steps {
                let mut s_dict = Dict::new();
                s_dict.insert("title".into(), s.name.clone().into_value());
                s_dict.insert("desc".into(), s.description.clone().into_value());

                let mut dur_dict = Dict::new();
                dur_dict.insert("fix".into(), s.fixed_duration.into_value());
                dur_dict.insert("var".into(), s.duration_per_kg.into_value());
                s_dict.insert("duration".into(), dur_dict.into_value());

                steps_arr.push(s_dict.into_value());
            }
            dict.insert("steps".into(), steps_arr.into_value());

            // Map Allergens
            let contains: Vec<String> = sub.properties.iter().map(|p| p.name.clone()).collect();
            dict.insert("allergens".into(), Value::Dict(map_allergens(&sub.dietary, &contains)));

            pages.push(dict.into_value());
        }
    }

    let mut out = Dict::new();
    out.insert("v".into(), pages.into_value());
    out
}

pub fn map_event_to_typst(info: &EventAllergenInfo) -> Dict {
    let format = format_description::parse("[day].[month]. [hour]:[minute]").unwrap();
    let mut dict = Dict::new();

    dict.insert("event_name".into(), info.event_name.clone().into_value());

    let mut meals_arr = Array::new();
    for m in &info.meals {
        let mut m_dict = Dict::new();
        m_dict.insert("name".into(), m.recipe_name.clone().into_value());
        m_dict.insert("place".into(), m.place.clone().into_value());
        m_dict.insert(
            "time".into(),
            m.start_time
                .format(&format)
                .unwrap_or_default()
                .into_value(),
        );
        m_dict.insert("servings".into(), m.servings.into_value());

        let contains: Vec<String> = m.properties.iter().map(|p| p.name.clone()).collect();
        m_dict.extend(map_allergens(&m.dietary, &contains).into_iter());

        meals_arr.push(m_dict.into_value());
    }
    dict.insert("meals".into(), meals_arr.into_value());

    dict
}

fn map_allergens(dietary: &DietaryFlags, contains: &[String]) -> Dict {
    let mut dict = Dict::new();
    let mut d_flags = Dict::new();

    d_flags.insert("vegan".into(), dietary.vegan.into_value());
    d_flags.insert("vegan_trace".into(), dietary.vegan_trace.into_value());
    d_flags.insert("vegetarian".into(), dietary.vegetarian.into_value());
    d_flags.insert(
        "vegetarian_trace".into(),
        dietary.vegetarian_trace.into_value(),
    );
    d_flags.insert("lactose_free".into(), dietary.lactose_free.into_value());
    d_flags.insert(
        "lactose_free_trace".into(),
        dietary.lactose_free_trace.into_value(),
    );
    d_flags.insert("gluten_free".into(), dietary.gluten_free.into_value());
    d_flags.insert(
        "gluten_free_trace".into(),
        dietary.gluten_free_trace.into_value(),
    );

    dict.insert("dietary".into(), d_flags.into_value());
    dict.insert("contains".into(), contains.to_vec().into_value());
    dict.insert("may_contain".into(), Array::new().into_value());
    dict.insert("hinweise".into(), Array::new().into_value());

    dict
}
