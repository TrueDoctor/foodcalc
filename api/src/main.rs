use axum::Router;
use http::header::CONTENT_TYPE;
use http::Method;
use tokio::net::TcpListener;

use foodlib::*;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod events;
mod ingredients;
mod mealcalc;
mod places;
mod reciepes;

#[derive(Clone)]
struct ApiState {
    food_base: FoodBase,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Get Database Connection
    println!("Setting up Database");
    let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");
    let food_base = FoodBase::new(database_url)
        .await
        .expect("Failed to connect to database");

    println!("Setting up Logging");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    println!("Loading Routes");
    let app = Router::<ApiState>::new()
        .nest("/events", events::router())
        .nest("/ingredients", ingredients::router())
        .nest("/places", places::router())
        .nest("/reciepes", reciepes::router())
        .nest("/calc", mealcalc::router())
        .with_state(ApiState { food_base })
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    println!("Setting up Webserver");
    let interface = &env::var("API_INTERFACE").expect("API_INTERFACE env var was not set");
    let port = &env::var("API_PORT").expect("API_PORT env var was not set");
    let socket_str = format!("{}:{}", interface, port);
    println!("Starting Server on '{}'", &socket_str);
    let listener = TcpListener::bind(socket_str).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
