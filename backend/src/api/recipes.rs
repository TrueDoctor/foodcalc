use std::{sync::Arc, time::Duration};

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post, MethodRouter},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;

use crate::db::{FoodBase, RecipeIngrdient, RecipeStep};

#[derive(Clone, Serialize, Deserialize)]
struct Recipe {
    name: String,
    comment: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SerializableRecipeStep {
    step_id: i32,
    step_order: f64,
    step_name: String,
    step_description: String,
    recipe_id: i32,
    fixed_duration: Duration,
    duration_per_kg: Duration,
}

pub fn create(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    post(|recipe: Json<Recipe>| async move {
        let recipe = crate::db::Recipe {
            recipe_id: 0,
            name: recipe.name.to_owned(),
            comment: recipe.comment.to_owned(),
        };
        let result = db.insert_recipe(&recipe).await;
        match result {
            Ok(r) => (StatusCode::CREATED, Json(Some(r.recipe_id))),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
        }
    })
}

pub fn update(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    post(
        |Path(recipe_id): Path<i32>, recipe: Json<Recipe>| async move {
            let recipe = crate::db::Recipe {
                recipe_id: recipe_id,
                name: recipe.name.to_owned(),
                comment: recipe.comment.to_owned(),
            };
            let result = db.update_recipe(&recipe).await;
            match result {
                Ok(r) => (StatusCode::CREATED, Json(Some(r.recipe_id))),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
            }
        },
    )
}

pub fn list(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    get(|| async move {
        let recipes = db.get_recipes().await;
        match recipes {
            Ok(recipes) => (StatusCode::OK, Json(recipes)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn list_meta_ingredients(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    get(|Path(recipe_id): Path<i32>| async move {
        let subrecipes = db.get_meta_ingredients(recipe_id).await;
        match subrecipes {
            Ok(subrecipes) => (StatusCode::OK, Json(subrecipes)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn list_all_meta_ingredients(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    get(|| async move {
        let subrecipes = db.get_all_meta_ingredients().await;
        match subrecipes {
            Ok(subrecipes) => (StatusCode::OK, Json(subrecipes)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn list_subrecipes(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    get(|Path(recipe_id): Path<i32>| async move {
        let subrecipes = db.get_recipe_meta_ingredients(recipe_id).await;
        match subrecipes {
            Ok(subrecipes) => (StatusCode::OK, Json(subrecipes)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn list_ingredients(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    get(|Path(recipe_id): Path<i32>| async move {
        let ingredients = db.get_recipe_ingredients(recipe_id).await;
        match ingredients {
            Ok(ingredients) => (StatusCode::OK, Json(ingredients)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}

pub fn update_entry(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    post(
        |Path(recipe_id): Path<i32>, Json(entries): Json<Vec<RecipeIngrdient>>| async move {
            let result = db
                .update_recipe_entries(recipe_id, entries.into_iter())
                .await;
            match result {
                Ok(_) => StatusCode::CREATED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        },
    )
}

pub fn update_steps(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    post(
        |Path(recipe_id): Path<i32>, Json(steps): Json<Vec<SerializableRecipeStep>>| async move {
            let steps = steps.into_iter().map(deserialize_step);
            let result = db.update_recipe_steps(recipe_id, steps).await;
            match result {
                Ok(_) => StatusCode::CREATED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        },
    )
}

fn deserialize_step(step: SerializableRecipeStep) -> RecipeStep {
    RecipeStep {
        step_id: step.step_id,
        step_order: step.step_order,
        step_name: step.step_name,
        step_description: step.step_description,
        recipe_id: step.recipe_id,
        fixed_duration: PgInterval::try_from(step.fixed_duration).unwrap(),
        duration_per_kg: PgInterval::try_from(step.duration_per_kg).unwrap(),
    }
}

fn serialize_step(step: RecipeStep) -> SerializableRecipeStep {
    SerializableRecipeStep {
        step_id: step.step_id,
        step_order: step.step_order,
        step_name: step.step_name,
        step_description: step.step_description,
        recipe_id: step.recipe_id,
        fixed_duration: Duration::default(),
        duration_per_kg: Duration::default(),
    }
}

pub fn get_steps(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    get(|Path(recipe_id): Path<i32>| async move {
        let steps = db.get_recipe_steps(recipe_id).await;
        match steps {
            Ok(steps) => (
                StatusCode::OK,
                Json(steps.into_iter().map(serialize_step).collect::<Vec<_>>()),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
        }
    })
}
