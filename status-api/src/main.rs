use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use http::header::CONTENT_TYPE;
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use foodlib::*;
use std::collections::HashMap;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone, Deserialize, Serialize)]
struct MealStatus {
    eta: u32,
    msg: Option<String>,
    recipe: String,
}

#[derive(Clone)]
struct ApiState {
    food_base: FoodBase,
    meal_states: HashMap<i32, MealStatus>,
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
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    let mut meal_states = HashMap::new();

    let meal_id = 136;
    println!("Adding Meal {meal_id}");
    meal_states.insert(
        meal_id,
        MealStatus {
            eta: 10,
            msg: None,
            recipe: "Lots of Love".to_string(),
        },
    );
    let meal_id = 137;
    println!("Adding Meal {meal_id}");
    meal_states.insert(
        meal_id,
        MealStatus {
            eta: 0,
            msg: Some("This is an optional custom Status".to_string()),
            recipe: "Pures MSG".to_string(),
        },
    );
    let meal_id = 138;
    println!("Adding Meal {meal_id}");
    meal_states.insert(
        meal_id,
        MealStatus {
            eta: 0,
            msg: None,
            recipe: "Pizza".to_string(),
        },
    );

    println!("Loading Routes");
    let app = Router::<ApiState>::new()
        .route("/", get(get_status))
        .route("/:meal_id", post(update_status))
        .with_state(ApiState {
            food_base,
            meal_states,
        })
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

#[derive(Clone, Deserialize, Serialize)]
struct FullMealStatus {
    meal_id: i32,
    status: MealStatus
}
async fn get_status(State(state): State<ApiState>) -> impl IntoResponse {
    let data = state.meal_states.iter().map(|(meal_id, status)| FullMealStatus {
        meal_id: *meal_id,
        status: status.clone()
    }).collect::<Vec<FullMealStatus>>();
    (StatusCode::OK, Json(data))
}

async fn update_status(
    State(mut state): State<ApiState>,
    Path(meal_id): Path<i32>,
    Json(status): Json<MealStatus>,
) -> impl IntoResponse {
    state.meal_states.insert(meal_id, status);
    (StatusCode::OK, Json(state.meal_states))
}
