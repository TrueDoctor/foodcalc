use std::sync::Arc;

use axum::{
    Router, routing::{get}, http::{StatusCode, header, Uri}, response::{IntoResponse, Response}, body::{boxed, Full},
};
use rust_embed::RustEmbed;

use crate::db::FoodBase;

mod events;
mod ingredients;
mod recipes;
mod stores;
mod utils;

pub fn foodbase(db: FoodBase) -> Router {
    let db = Arc::new(db);
    Router::new()
        .fallback_service(get(static_handler))
        .route("/", get(index_handler))
        .nest("/ingredients", ingredients_router(db.clone()))
        .nest("/recipes", recipes_router(db.clone()))
        .nest("/events", events_router(db.clone()))
        .nest("/stores/ingredients", stores_router(db.clone()))
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
        .route(
            "/meta_ingredients/list",
            recipes::list_all_meta_ingredients(db.clone()),
        )
        .route(
            "/:recipe_id/meta_ingredients/list",
            recipes::list_meta_ingredients(db.clone()),
        )
        .route(
            "/:recipe_id/subrecipes/list",
            recipes::list_subrecipes(db.clone()),
        )
        .route(
            "/:recipe_id/ingredients/list",
            recipes::list_ingredients(db.clone()),
        )
        .route(
            "/:recipe_id/meta_ingredients/update",
            recipes::update_entry(db.clone()),
        )
        .route(
            "/:recipe_id/steps/update",
            recipes::update_steps(db.clone()),
        )
        .route("/:recipe_id/steps/list", recipes::get_steps(db.clone()))
}

pub fn events_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/create", events::create_empty(db.clone()))
        .route("/:event_id/update", events::update_event(db.clone()))
        .route("/:event_id/meals/update", events::update_meals(db.clone()))
        .route("/:event_id/meals/list", events::get_meals(db.clone()))
        .route("/:event_id/cost", events::get_event_cost(db.clone()))
        .route(
            "/meal/ingredients/list",
            events::get_recipe_ingredients(db.clone()),
        )
        .route("/list", events::list(db.clone()))
        .route("/meal/update", events::update_single_meal(db.clone()))
}

pub fn stores_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/add", stores::add_ingredient_source(db.clone()))
        .route("/:id/fetch", stores::fetch_metro_prices(db.clone()))
        .route("/fetch", stores::fetch_all_metro_prices(db.clone()))
        .route("/:id/get", stores::get_metro_ingredient_sources(db.clone()))
        .route(
            "/list",
            stores::get_all_metro_ingredient_sources(db.clone()),
        )
        .route(
            "/update",
            stores::update_ingredient_source_price(db.clone()),
        )
}

pub fn utils_router(db: Arc<FoodBase>) -> Router {
    Router::new()
        .route("/units", utils::get_units(db.clone()))
        .route("/places", utils::get_places(db.clone()))
}

async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
  }
  
  // We use a wildcard matcher ("/dist/*file") to match against everything
  // within our defined assets directory. This is the directory on our Asset
  // struct below, where folder = "examples/public/".
  async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
  
    if path.starts_with("dist/") {
      path = path.replace("dist/", "");
    }
  
    StaticFile(path)
  }

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
  T: Into<String>,
{
  fn into_response(self) -> Response {
    let path = self.0.into();

    match Asset::get(path.as_str()) {
      Some(content) => {
        let body = boxed(Full::from(content.data));
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        Response::builder().header(header::CONTENT_TYPE, mime.as_ref()).body(body).unwrap()
      }
      None => Response::builder().status(StatusCode::NOT_FOUND).body(boxed(Full::from("404"))).unwrap(),
    }
  }
}