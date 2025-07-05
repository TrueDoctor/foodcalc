use ::time::OffsetDateTime;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use foodlib_new::meal::Meal;
use foodlib_new::FoodLib;
use http::header::CONTENT_TYPE;
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task;

use std::collections::HashMap;
use std::env;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    recipe_id: i32,
    place: String,
    meal_id: i32,
    event_id: i32, // Added event_id for cache invalidation
}

#[derive(Clone, Deserialize, Serialize)]
struct FullMealStatus {
    meal_id: i32,
    status: MealStatus,
}

#[derive(Clone, Deserialize, Serialize)]
struct FeedbackStatus {
    feedback_id: i32, // Unique ID for feedback
    feedback: String,
    event_id: i32,
    timestamp: u64,
    read: bool,
    assigned_to: Option<String>,
}

#[derive(Deserialize)]
struct FeedbackUpdate {
    read: Option<bool>,
    assigned_to: Option<String>,
}

#[derive(Deserialize)]
struct FeedbackQuery {
    include_read: Option<bool>,
}

// Define the cache structure
#[derive(Clone)]
struct EventCache {
    meals: HashMap<i32, Vec<FullMealStatus>>,
    last_updated: HashMap<i32, Instant>,
    upcoming_event_id: Option<i32>,
    last_upcoming_check: Instant,
    feedback: Vec<FeedbackStatus>,
}

impl EventCache {
    fn new() -> Self {
        Self {
            meals: HashMap::new(),
            last_updated: HashMap::new(),
            upcoming_event_id: None,
            last_upcoming_check: Instant::now(),
            feedback: Vec::new(),
        }
    }

    fn is_event_cache_valid(&self, event_id: i32, cache_ttl: Duration) -> bool {
        match self.last_updated.get(&event_id) {
            Some(time) => time.elapsed() < cache_ttl,
            None => false,
        }
    }

    fn is_upcoming_cache_valid(&self, cache_ttl: Duration) -> bool {
        self.last_upcoming_check.elapsed() < cache_ttl
    }

    fn update_event_cache(&mut self, event_id: i32, meals: Vec<FullMealStatus>) {
        self.meals.insert(event_id, meals);
        self.last_updated.insert(event_id, Instant::now());
    }

    fn update_upcoming_event(&mut self, event_id: Option<i32>) {
        self.upcoming_event_id = event_id;
        self.last_upcoming_check = Instant::now();
    }

    fn get_event_meals(&self, event_id: i32) -> Option<&Vec<FullMealStatus>> {
        self.meals.get(&event_id)
    }
}

// Query parameters struct
#[derive(serde::Deserialize)]
struct EventQuery {
    event: Option<i32>,
}

// Modified AppState to include cache
#[derive(Clone)]
struct AppState {
    meal_states: Arc<RwLock<HashMap<i32, MealStatus>>>,
    event_cache: Arc<RwLock<EventCache>>,
    food_lib: Arc<FoodLib>,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Background task to refresh the cache
async fn start_cache_refresh_task(state: AppState, refresh_interval: Duration) {
    task::spawn(async move {
        let mut interval = tokio::time::interval(refresh_interval);
        loop {
            interval.tick().await;
            refresh_cache(&state).await;
        }
    });
}

async fn refresh_cache(state: &AppState) {
    // Refresh the upcoming event
    let upcoming_event_id = get_upcoming_event(&state.food_lib).await;

    // Update the upcoming event in cache
    {
        let mut cache = state.event_cache.write().await;
        cache.update_upcoming_event(upcoming_event_id);
    }

    // If we have an upcoming event, make sure its meals are cached
    if let Some(event_id) = upcoming_event_id {
        let event_meals = get_event_meals_from_db(&state.food_lib, event_id).await;

        // Process meals with current state and update cache
        let full_meals = process_meals(
            event_meals,
            state.meal_states.write().await.deref(),
            event_id,
        );
        let mut cache = state.event_cache.write().await;
        cache.update_event_cache(event_id, full_meals);
    }

    println!("Cache refreshed at {}", OffsetDateTime::now_utc());
}

// Get the next upcoming event
async fn get_upcoming_event(food_lib: &FoodLib) -> Option<i32> {
    match food_lib
        .events()
        .get_upcoming_events(OffsetDateTime::now_utc())
        .await
    {
        Ok(events) => events.first().map(|e| e.id),
        Err(e) => {
            eprintln!("Error fetching upcoming events: {}", e);
            None
        }
    }
}

// Get event meals from database
async fn get_event_meals_from_db(food_lib: &FoodLib, event_id: i32) -> Vec<Meal> {
    match get_event_meals_internal(food_lib, event_id).await {
        Ok(meals) => meals,
        Err(e) => {
            eprintln!("Error fetching meals for event {}: {}", event_id, e);
            Vec::new()
        }
    }
}

// Helper to fetch meals from database
async fn get_event_meals_internal(
    food_lib: &FoodLib,
    event_id: i32,
) -> Result<Vec<Meal>, foodlib_new::Error> {
    food_lib.meals().get_event_meals(event_id).await
}

// Process meals with current state
fn process_meals(
    meals: Vec<Meal>,
    meal_states: &HashMap<i32, MealStatus>,
    event_id: i32,
) -> Vec<FullMealStatus> {
    let current_time = now();

    meals
        .into_iter()
        .map(|meal| {
            let status = meal_states.get(&meal.meal_id).cloned().unwrap_or_else(|| {
                // Default meal status if not in state
                MealStatus {
                    start: meal.start_time.unix_timestamp(),
                    end: meal.end_time.unix_timestamp(),
                    eta: current_time as i64 - 1, // Default to 1 second before current time
                    last_modified: current_time,
                    over: false,
                    msg: None,
                    recipe: meal.name.clone(),
                    recipe_id: meal.recipe_id,
                    place: meal.place.clone(),
                    meal_id: meal.meal_id,
                    event_id,
                }
            });

            FullMealStatus {
                meal_id: meal.meal_id,
                status,
            }
        })
        .collect()
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Get Database Connection
    println!("Setting up Database");
    let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");
    let pg_pool = PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database");
    let food_lib = FoodLib::new(pg_pool);

    println!("Setting up Logging");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    let meal_states = Arc::new(RwLock::new(HashMap::new()));
    let event_cache = Arc::new(RwLock::new(EventCache::new()));
    let feedback = Arc::new(RwLock::new(Vec::new()));
    let food_lib = Arc::new(food_lib);

    // Initialize the app state
    let app_state = AppState {
        meal_states,
        event_cache,
        food_lib: food_lib.clone(),
    };

    // Start the cache refresh task
    start_cache_refresh_task(app_state.clone(), Duration::from_secs(300)).await;

    // Pre-populate meal states from the next event
    if let Some(next_event) = get_upcoming_event(&food_lib).await {
        let event_meals = get_event_meals_from_db(&food_lib, next_event).await;
        let current_time = now();

        for meal in &event_meals {
            app_state.meal_states.write().await.insert(
                meal.meal_id,
                MealStatus {
                    start: meal.start_time.unix_timestamp(),
                    end: meal.end_time.unix_timestamp(),
                    eta: current_time as i64 - 1, // Default to 1 second before current time
                    last_modified: current_time,
                    over: false,
                    msg: None,
                    recipe: meal.name.clone(),
                    recipe_id: meal.recipe_id,
                    place: meal.place.clone(),
                    meal_id: meal.meal_id,
                    event_id: next_event,
                },
            );
        }

        // Initialize the cache with the upcoming event's meals
        let full_meals = process_meals(
            event_meals,
            app_state.meal_states.read().await.deref(),
            next_event,
        );
        app_state
            .event_cache
            .write()
            .await
            .update_event_cache(next_event, full_meals);
        app_state
            .event_cache
            .write()
            .await
            .update_upcoming_event(Some(next_event));
    }

    println!("Loading Routes");
    let app = Router::new()
        .route("/", get(get_status))
        .route("/{meal_id}", post(update_status))
        .route("/events", get(get_events))
        .route("/events/{event_id}", get(get_event_details))
        .route("/feedback", get(get_feedback))
        .route("/feedback", post(handle_feedback))
        .route("/feedback/{id}", post(update_feedback))
        .route("/feedback/{id}", delete(delete_feedback))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    println!("Setting up Webserver");
    let interface = &env::var("API_INTERFACE").expect("API_INTERFACE env var was not set");
    let port = &env::var("API_PORT").expect("API_PORT env var was not set");
    let socket_str = format!("{}:{}", interface, port);
    println!("Starting Server on '{}'", &socket_str);
    let listener = TcpListener::bind(socket_str).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Modified handler for getting status
#[axum::debug_handler]
async fn get_status(
    State(state): State<AppState>,
    Query(params): Query<EventQuery>,
) -> impl IntoResponse {
    // Determine which event to display
    let event_id = match params.event {
        Some(id) => id,
        None => {
            // Use cached upcoming event or fetch if cache is stale
            let cache = state.event_cache.read().await;
            if let Some(id) = cache.upcoming_event_id {
                if cache.is_upcoming_cache_valid(Duration::from_secs(300)) {
                    id
                } else {
                    drop(cache); // Release lock before async operation
                    match get_upcoming_event(&state.food_lib).await {
                        Some(id) => {
                            let mut cache = state.event_cache.write().await;
                            cache.update_upcoming_event(Some(id));
                            id
                        }
                        None => {
                            return (
                                StatusCode::NOT_FOUND,
                                Json(Vec::<Vec<FullMealStatus>>::new()),
                            );
                        }
                    }
                }
            } else {
                drop(cache); // Release lock before async operation
                match get_upcoming_event(&state.food_lib).await {
                    Some(id) => {
                        let mut cache = state.event_cache.write().await;
                        cache.update_upcoming_event(Some(id));
                        id
                    }
                    None => {
                        return (
                            StatusCode::NOT_FOUND,
                            Json(Vec::<Vec<FullMealStatus>>::new()),
                        );
                    }
                }
            }
        }
    };

    // Check if we have a valid cache for this event
    let cache_ttl = Duration::from_secs(60); // 1 minute TTL
    let meals = {
        let cache = state.event_cache.read().await;
        if cache.is_event_cache_valid(event_id, cache_ttl) {
            match cache.get_event_meals(event_id) {
                Some(meals) => meals.clone(),
                None => Vec::new(),
            }
        } else {
            drop(cache); // Release lock before async operation

            // Fetch meals from database
            let event_meals = get_event_meals_from_db(&state.food_lib, event_id).await;
            if event_meals.is_empty() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(Vec::<Vec<FullMealStatus>>::new()),
                );
            }

            // Process and cache meals
            let full_meals = process_meals(
                event_meals,
                state.meal_states.read().await.deref(),
                event_id,
            );
            let mut cache = state.event_cache.write().await;
            cache.update_event_cache(event_id, full_meals.clone());
            full_meals
        }
    };

    // Group meals by full date (year, month, day)
    let hour = 3600;
    let get_date_key = |time: i64| {
        let dt = OffsetDateTime::from_unix_timestamp(time - 3 * hour)
            .unwrap()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
            .date();

        // Return tuple of (year, month, day) for comparison
        (dt.year(), dt.month() as i32, dt.day())
    };

    if meals.is_empty() {
        return (StatusCode::OK, Json(Vec::<Vec<FullMealStatus>>::new()));
    }

    let mut current_date = get_date_key(meals[0].status.start);
    let mut days = vec![vec![]];

    for meal in meals {
        let new_date = get_date_key(meal.status.start);
        if new_date == current_date {
            days.last_mut().unwrap().push(meal);
        } else {
            days.push(vec![meal]);
            current_date = new_date;
        }
    }

    (StatusCode::OK, Json(days))
}

// Modified update_status to also update the cache
async fn update_status(
    State(state): State<AppState>,
    Path(meal_id): Path<i32>,
    Json(mut status): Json<MealStatus>,
) -> impl IntoResponse {
    let mut app_state = state.meal_states.write().await;
    status.last_modified = now();

    // Get the event_id of the meal
    let event_id = status.event_id;

    app_state.insert(meal_id, status.clone());

    // Update cache for this event if it exists
    let mut cache = state.event_cache.write().await;
    if let Some(meals) = cache.get_event_meals(event_id).cloned() {
        let updated_meals = meals
            .into_iter()
            .map(|mut meal| {
                if meal.meal_id == meal_id {
                    meal.status = status.clone();
                }
                meal
            })
            .collect();

        cache.update_event_cache(event_id, updated_meals);
    }

    let mut meals: Vec<_> = app_state.values().cloned().collect();
    meals.sort_unstable_by_key(|m| m.start);
    (StatusCode::OK, Json(meals))
}

// New handler for listing events
async fn get_events(State(state): State<AppState>) -> impl IntoResponse {
    match state.food_lib.events().list().await {
        Ok(events) => {
            // Map to a simpler format for the frontend
            let event_list: Vec<serde_json::Value> = events
                .into_iter()
                .map(|event| {
                    serde_json::json!({
                        "id": event.id,
                        "name": event.name,
                    })
                })
                .collect();

            (StatusCode::OK, Json(event_list))
        }
        Err(e) => {
            eprintln!("Error fetching events: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<serde_json::Value>::new()),
            )
        }
    }
}

// New handler for getting event details
async fn get_event_details(
    State(state): State<AppState>,
    Path(event_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_lib.events().get(event_id).await {
        Ok(event) => {
            let event_details = serde_json::json!({
                "id": event.id,
                "name": event.name,
                "comment": event.comment,
            });

            (StatusCode::OK, Json(event_details))
        }
        Err(e) => {
            eprintln!("Error fetching event {}: {}", event_id, e);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Event not found"})),
            )
        }
    }
}

// New handler for processing feedback
async fn handle_feedback(
    State(state): State<AppState>,
    Json(mut feedback): Json<FeedbackStatus>,
) -> impl IntoResponse {
    // Placeholder for feedback handling logic
    println!("Received feedback: {}", feedback.feedback);
    let mut feedback_list = state.feedback.write().await;
    feedback.feedback_id = feedback_list.last().map_or(1, |f| f.feedback_id + 1);
    feedback_list.push(feedback);
    (StatusCode::OK, Json("Feedback received"))
}

// New handler for getting feedback
async fn get_feedback(
    State(state): State<AppState>,
    Query(query): Query<FeedbackQuery>,
) -> impl IntoResponse {
    let feedback_list = state.feedback.read().await;
    if !query.include_read.unwrap_or(true) {
        // If include_read is false, filter out read feedback
        let unread_feedback: Vec<FeedbackStatus> =
            feedback_list.iter().filter(|f| !f.read).cloned().collect();
        return (StatusCode::OK, Json(unread_feedback));
    }
    (StatusCode::OK, Json(feedback_list.clone()))
}

async fn update_feedback(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(update): Json<FeedbackUpdate>,
) -> impl IntoResponse {
    let mut feedback_list = state.feedback.write().await;
    match feedback_list.binary_search_by_key(&id, |f| f.feedback_id) {
        Ok(idx) => {
            let feedback = &mut feedback_list[idx];
            let mut changed = false;
            if let Some(read) = update.read {
                feedback.read = read;
                changed = true;
            }
            if let Some(assigned_to) = update.assigned_to {
                feedback.assigned_to = Some(assigned_to);
                changed = true;
            }
            if changed {
                (StatusCode::OK, Json("Feedback updated"))
            } else {
                (StatusCode::NO_CONTENT, Json("No changes made"))
            }
        }
        Err(_) => (StatusCode::NOT_FOUND, Json("Feedback not found")),
    }
}

async fn delete_feedback(Path(id): Path<i32>, State(state): State<AppState>) -> impl IntoResponse {
    let mut feedback_list = state.feedback.write().await;
    match feedback_list.binary_search_by_key(&id, |f| f.feedback_id) {
        Ok(idx) => {
            feedback_list.remove(idx);
            (StatusCode::OK, Json("Feedback deleted"))
        }
        Err(_) => (StatusCode::NOT_FOUND, Json("Feedback not found")),
    }
}
