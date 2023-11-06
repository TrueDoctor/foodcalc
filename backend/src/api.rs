use axum::{
    body::{boxed, Full},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rust_embed::RustEmbed;

use crate::MyAppState;

mod events;
mod ingredients;
mod recipes;
mod stores;
mod utils;

pub type Router = axum::Router<MyAppState>;
type MethodRouter = axum::routing::MethodRouter<MyAppState>;

pub fn foodbase() -> Router {
    Router::new()
        .fallback_service(get(static_handler))
        .route("/", get(index_handler))
        .nest("/ingredients", ingredients_router())
        .nest("/recipes", recipes_router())
        .nest("/events", events_router())
        .nest("/stores/ingredients", stores_router())
        .nest("/utils", utils_router())
}

pub fn ingredients_router() -> Router {
    Router::new()
        .route("/html", get(ingredients::list_html))
        .route("/create", post(ingredients::create))
        .route("/search", post(ingredients::search))
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
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}
