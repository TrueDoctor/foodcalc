use std::sync::Arc;

use axum::Router;

use crate::db::FoodBase;

mod events;
mod ingredients;
mod recipes;
mod stores;
mod utils;

pub fn foodbase(db: FoodBase) -> Router {
    let db = Arc::new(db);
    Router::new()
        .nest("/ingredients", ingredients_router(db.clone()))
        .nest("/recipes", recipes_router(db.clone()))
        .nest("/events", events_router(db.clone()))
        .nest("/stores", stores_router(db.clone()))
        .nest("/utils", utils_router(db.clone()))
        
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
        .route("/:recipe_id/steps/list", recipes::get_steps(db.clone()))

}

pub fn events_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/create", events::create_empty(db.clone()))
        .route("/:event_id/update", events::update_event(db.clone()))
        .route("/:event_id/meals/update", events::update_meals(db.clone()))
        .route("/:event_id/meals/list", events::get_meals(db.clone()))
        .route("/:event_id/cost", events::get_event_cost(db.clone()))
        .route("/meal/ingredients/list", events::get_recipe_ingredients(db.clone()))
        .route("/list", events::list(db.clone()))
        .route("/meal/update", events::update_single_meal(db.clone()))
}

pub fn stores_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/ingredient/add", stores::add_ingredient_source(db.clone()))
        .route("/ingredient/:id/fetch", stores::fetch_metro_prices(db.clone()))
        .route("/ingredient/fetch", stores::fetch_all_metro_prices(db.clone()))
        .route("/ingredient/:id/get", stores::get_metro_ingredient_sources(db.clone()))
        .route("/ingredient/list", stores::get_all_metro_ingredient_sources(db.clone()))
        .route("/ingredient/update", stores::update_ingredient_source_price(db.clone()))

}

pub fn utils_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/units", utils::get_units(db.clone()))
        .route("/places", utils::get_places(db.clone()))
}