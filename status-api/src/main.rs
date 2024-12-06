use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use http::header::CONTENT_TYPE;
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::net::TcpListener;

use foodlib::*;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone, Deserialize, Serialize)]
struct MealStatus {
    start: i64,
    end: i64,
    eta: i64,
    last_modified: u64,
    over: bool,
    msg: Option<String>,
    recipe: String,
    place: String,
    meal_id: i32,
}

#[derive(Clone)]
struct ApiState {
    meal_states: HashMap<i32, MealStatus>,
}

type AppState = Arc<Mutex<ApiState>>;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
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

    let current_time = now();

    let event_meals = get_event_meals(&food_base, 38).await;

    for meal in event_meals {
        println!("Adding Meal {:?}", meal);
        let to_utc = |time: OffsetDateTime| time.unix_timestamp();
        println!("{}", to_utc(meal.start));
        meal_states.insert(
            meal.meal_id,
            MealStatus {
                start: to_utc(meal.start),
                end: to_utc(meal.end),
                last_modified: current_time,
                eta: to_utc(meal.start),
                msg: None,
                over: false,
                recipe: meal.recipe,
                place: meal.place,
                meal_id: meal.meal_id,
            },
        );
    }

    println!("Loading Routes");
    let app = Router::<AppState>::new()
        .route("/", get(get_status))
        .route("/:meal_id", post(update_status))
        .with_state(Arc::new(Mutex::new(ApiState { meal_states })))
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
    status: MealStatus,
}
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let mut data = state
        .lock()
        .unwrap()
        .meal_states
        .iter()
        .map(|(meal_id, status)| FullMealStatus {
            meal_id: *meal_id,
            status: status.clone(),
        })
        .collect::<Vec<FullMealStatus>>();
    data.sort_unstable_by_key(|m| m.status.start);

    // create vec of days with meals
    let hour = 3600;

    let day = |time| {
        OffsetDateTime::from_unix_timestamp(time - 3 * hour)
            .unwrap()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
            .date()
            .day()
    };
    let Some(first) = data.first() else {
        return (StatusCode::OK, Json(vec![]));
    };
    let mut current_day = day(first.status.start);
    let mut days = vec![vec![]];
    for meal in data {
        let new_day = day(meal.status.start);
        if new_day == current_day {
            days.last_mut().unwrap().push(meal);
        } else {
            days.push(vec![meal]); // Clone the meal variable
            current_day = new_day;
        }
    }
    (StatusCode::OK, Json(days))
}

async fn update_status(
    State(state): State<AppState>,
    Path(meal_id): Path<i32>,
    Json(mut status): Json<MealStatus>,
) -> impl IntoResponse {
    let mut state = state.lock().unwrap();
    status.last_modified = now();
    state.meal_states.insert(meal_id, status);
    let mut meals: Vec<_> = state.meal_states.values().cloned().collect();
    meals.sort_unstable_by_key(|m| m.start);
    (StatusCode::OK, Json(meals))
}

#[derive(Debug)]
struct EventMeal {
    meal_id: i32,
    #[allow(unused)]
    event_id: i32,
    #[allow(unused)]
    recipe_id: i32,
    start: OffsetDateTime,
    end: OffsetDateTime,
    recipe: String,
    place: String,
}

async fn get_event_meals(database: &FoodBase, event_id: i32) -> Vec<EventMeal> {
    let records = sqlx::query_as!(
        EventMeal,
        r#" SELECT
            event_id,
            meal_id,
            recipe_id,
            recipes.name as "recipe",
            places.name as "place",
            start_time as start,
            end_time as end

            FROM event_meals
            INNER JOIN recipes USING(recipe_id)
            INNER JOIN places USING(place_id)

            WHERE event_meals.event_id = $1
            ORDER BY event_meals.start_time 
        "#,
        event_id
    )
    .fetch_all(database.pool())
    .await;
    records.unwrap()
}
