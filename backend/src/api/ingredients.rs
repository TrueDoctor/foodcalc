use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post, MethodRouter},
    Json,
};
use serde::Deserialize;
use sqlx::types::BigDecimal;

use crate::db::FoodBase;

#[derive(Clone, Deserialize)]
struct Ingredient {
    name: String,
    energy: BigDecimal,
    comment: Option<String>,
}

pub fn create(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    post(|ingredient: Json<Ingredient>| async move {
        let id = db
            .add_ingredient(
                ingredient.name.to_owned(),
                ingredient.energy.to_owned(),
                ingredient.comment.to_owned(),
            )
            .await;
        match id {
            Ok(id) => (StatusCode::OK, Json(id)),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(-1)),
        }
    })
}

pub fn update(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    post(
        |Path(ingredient_id): Path<i32>, ingredient: Json<Ingredient>| async move {
            db.update_ingredient(crate::db::Ingredient::new(
                ingredient_id,
                ingredient.name.to_owned(),
                ingredient.energy.to_owned(),
                ingredient.comment.to_owned(),
            ))
            .await
            .unwrap_or_default();
        },
    )
}

pub fn list(foodbase: Arc<FoodBase>) -> MethodRouter {
    let db = foodbase.clone();
    let list = || async move {
        let ingredients = db.get_ingredients().await.unwrap_or_default();
        Json(ingredients)
    };
    get(list)
}
