use std::sync::Arc;

use axum::Router;

use crate::db::FoodBase;

mod events;
mod ingredients;
mod recipes;

pub fn foodbase(db: FoodBase) -> Router {
    let db = Arc::new(db);
    Router::new()
        .nest("/ingredients", ingredients_router(db.clone()))
        .nest("/recipes", recipes_router(db.clone()))
}

pub fn ingredients_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/create", ingredients::create(db.clone()))
        .route("/update/:ingredient_id", ingredients::update(db.clone()))
        .route("/list", ingredients::list(db.clone()))
}

pub fn recipes_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/create", recipes::create(db.clone()))
        .route("/:recipe_id/update", recipes::update(db.clone()))
        .route("/list", recipes::list(db.clone()))
        .route("/meta_ingredients/list", recipes::list_all_meta_ingredients(db.clone()))
        .route("/:recipe_id/meta_ingredients/list", recipes::list_meta_ingredients(db.clone()))
        .route("/:recipe_id/subrecipes/list", recipes::list_subrecipes(db.clone()))
        .route("/:recipe_id/ingredients/list", recipes::list_ingredients(db.clone()))
        .route("/:recipe_id/meta_ingredients/update", recipes::update_entry(db.clone()))
        .route("/:recipe_id/steps/update", recipes::update_steps(db.clone()))

}