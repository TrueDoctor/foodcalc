use axum::routing::{get, post};

use crate::MyAppState;

mod events;
mod ingredients;
mod recipes;
mod stores;
mod utils;

pub type Router = axum::Router<MyAppState>;

pub fn foodbase() -> Router {
    Router::new()
        .nest("/ingredients", ingredients_router())
        .nest("/recipes", recipes_router())
        .nest("/events", events_router())
        .nest("/stores/ingredients", stores_router())
        .nest("/utils", utils_router())
}

pub fn ingredients_router() -> Router {
    Router::new()
        .route("/create", post(ingredients::create))
        .route("/update/:ingredient_id", post(ingredients::update))
        .route("/list", get(ingredients::list))
}

pub fn recipes_router() -> Router {
    Router::new()
        .route("/create", post(recipes::create))
        .route("/:recipe_id/update", post(recipes::update))
        .route("/list", get(recipes::list))
        .route(
            "/meta_ingredients/list",
            get(recipes::list_all_meta_ingredients),
        )
        .route(
            "/:recipe_id/meta_ingredients/list",
            get(recipes::list_meta_ingredients),
        )
        .route("/:recipe_id/subrecipes/list", get(recipes::list_subrecipes))
        .route(
            "/:recipe_id/ingredients/list",
            get(recipes::list_ingredients),
        )
        .route(
            "/:recipe_id/meta_ingredients/update",
            post(recipes::update_entry),
        )
        .route("/:recipe_id/steps/update", post(recipes::update_steps))
        .route("/:recipe_id/steps/list", get(recipes::get_steps))
}

pub fn events_router() -> Router {
    Router::new()
        .route("/create", post(events::create_empty))
        .route("/:event_id/update", post(events::update_event))
        .route("/:event_id/meals/update", post(events::update_meals))
        .route("/:event_id/meals/list", get(events::get_meals))
        .route("/:event_id/cost", get(events::get_event_cost))
        .route(
            "/meal/ingredients/list",
            get(events::get_recipe_ingredients),
        )
        .route("/list", get(events::list))
        .route("/meal/update", post(events::update_single_meal))
}

pub fn stores_router() -> Router {
    Router::new()
        .route("/add", post(stores::add_ingredient_source))
        .route("/:id/fetch", get(stores::fetch_metro_prices))
        .route("/fetch", get(stores::fetch_all_metro_prices))
        .route("/:id/get", get(stores::get_metro_ingredient_sources))
        .route("/list", get(stores::get_all_metro_ingredient_sources))
        .route("/update", post(stores::update_ingredient_source_price))
}

pub fn utils_router() -> Router {
    Router::new()
        .route("/units", get(utils::get_units))
        .route("/places", get(utils::get_places))
}
