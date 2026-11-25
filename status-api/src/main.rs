use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use foodlib_new::entities::allergen::DietaryFlags;
use foodlib_new::entities::meal::Meal;
use foodlib_new::types::ids::{EventId, MealId, RecipeId};
use foodlib_new::AppState as FoodState;
use http::header::CONTENT_TYPE;
use http::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ::time::OffsetDateTime;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task;
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
    recipe_id: RecipeId,
    place: String,
    meal_id: MealId,
    event_id: EventId,
    #[serde(default)]
    allergens: Vec<String>,
    #[serde(default)]
    dietary: DietaryStatus,
}

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
struct DietaryStatus {
    vegan: bool,
    vegetarian: bool,
    lactose_free: bool,
    gluten_free: bool,
    #[serde(default)]
    vegan_trace: bool,
    #[serde(default)]
    vegetarian_trace: bool,
    #[serde(default)]
    lactose_free_trace: bool,
    #[serde(default)]
    gluten_free_trace: bool,
}

impl From<DietaryFlags> for DietaryStatus {
    fn from(d: DietaryFlags) -> Self {
        Self {
            vegan: d.vegan,
            vegetarian: d.vegetarian,
            lactose_free: d.lactose_free,
            gluten_free: d.gluten_free,
            vegan_trace: d.vegan_trace,
            vegetarian_trace: d.vegetarian_trace,
            lactose_free_trace: d.lactose_free_trace,
            gluten_free_trace: d.gluten_free_trace,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct FullMealStatus {
    meal_id: MealId,
    status: MealStatus,
}

#[derive(Clone, Deserialize, Serialize)]
struct FeedbackStatus {
    feedback_id: i32,
    feedback: String,
    event_id: EventId,
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

#[derive(Clone)]
struct EventCache {
    meals: HashMap<EventId, Vec<FullMealStatus>>,
    last_updated: HashMap<EventId, Instant>,
    upcoming_event_id: Option<EventId>,
    last_upcoming_check: Instant,
}

impl EventCache {
    fn new() -> Self {
        Self {
            meals: HashMap::new(),
            last_updated: HashMap::new(),
            upcoming_event_id: None,
            last_upcoming_check: Instant::now(),
        }
    }

    fn is_event_cache_valid(&self, event_id: EventId, cache_ttl: Duration) -> bool {
        match self.last_updated.get(&event_id) {
            Some(time) => time.elapsed() < cache_ttl,
            None => false,
        }
    }

    fn is_upcoming_cache_valid(&self, cache_ttl: Duration) -> bool {
        self.last_upcoming_check.elapsed() < cache_ttl
    }

    fn update_event_cache(&mut self, event_id: EventId, meals: Vec<FullMealStatus>) {
        self.meals.insert(event_id, meals);
        self.last_updated.insert(event_id, Instant::now());
    }

    fn update_upcoming_event(&mut self, event_id: Option<EventId>) {
        self.upcoming_event_id = event_id;
        self.last_upcoming_check = Instant::now();
    }

    fn get_event_meals(&self, event_id: EventId) -> Option<&Vec<FullMealStatus>> {
        self.meals.get(&event_id)
    }
}

#[derive(serde::Deserialize)]
struct EventQuery {
    event: Option<EventId>,
}

#[derive(Clone)]
struct AppState {
    meal_states: Arc<RwLock<HashMap<MealId, MealStatus>>>,
    feedback: Arc<RwLock<Vec<FeedbackStatus>>>,
    event_cache: Arc<RwLock<EventCache>>,
    food_lib: FoodState,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

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
    let upcoming_event_id = get_upcoming_event(&state.food_lib).await;

    {
        let mut cache = state.event_cache.write().await;
        cache.update_upcoming_event(upcoming_event_id);
    }

    if let Some(event_id) = upcoming_event_id {
        let event_meals = get_event_meals_from_db(&state.food_lib, event_id).await;
        let allergens = fetch_allergens_for_meals(&state.food_lib, &event_meals).await;

        let full_meals = process_meals(
            event_meals,
            state.meal_states.read().await.deref(),
            event_id,
            &allergens,
        );
        let mut cache = state.event_cache.write().await;
        cache.update_event_cache(event_id, full_meals);
    }

    println!("Cache refreshed at {}", OffsetDateTime::now_utc());
}

async fn get_upcoming_event(food_lib: &FoodState) -> Option<EventId> {
    match foodlib_new::events::get_current_or_upcoming_event(food_lib, OffsetDateTime::now_utc())
        .await
    {
        Ok(event) => event.map(|e| e.id),
        Err(e) => {
            eprintln!("Error fetching current/upcoming event: {}", e);
            None
        }
    }
}

async fn get_event_meals_from_db(food_lib: &FoodState, event_id: EventId) -> Vec<Meal> {
    match foodlib_new::events::meals::get_event_meals(food_lib, event_id).await {
        Ok(meals) => meals,
        Err(e) => {
            eprintln!("Error fetching meals for event {}: {}", event_id, e);
            Vec::new()
        }
    }
}

async fn fetch_allergens_for_meals(
    food_lib: &FoodState,
    meals: &[Meal],
) -> HashMap<RecipeId, (Vec<String>, DietaryStatus)> {
    let recipe_ids: std::collections::HashSet<_> =
        meals.iter().map(|meal| meal.recipe_id).collect();

    let mut out = HashMap::with_capacity(recipe_ids.len());

    for recipe_id in recipe_ids {
        let data = foodlib_new::recipes::gather_recipe_allergens(food_lib, recipe_id)
            .await
            .unwrap();

        let mut names = data.contains;
        names.extend(data.may_contain.into_iter().map(|name| format!("~spuren-{name}")));
        names.extend(data.hinweise.into_iter().map(|name| format!("~hinweis-{name}")));

        out.insert(recipe_id, (names, data.dietary.into()));
    }

    out
}


fn process_meals(
    meals: Vec<Meal>,
    meal_states: &HashMap<MealId, MealStatus>,
    event_id: EventId,
    allergens: &HashMap<RecipeId, (Vec<String>, DietaryStatus)>,
) -> Vec<FullMealStatus> {
    let current_time = now();

    meals
        .into_iter()
        .map(|meal| {
            let (recipe_allergens, recipe_dietary) = allergens
                .get(&meal.recipe_id)
                .cloned()
                .unwrap_or_default();
            let mut status = meal_states
                .get(&meal.meal_id)
                .cloned()
                .unwrap_or_else(|| MealStatus {
                    start: meal.start_time.unix_timestamp(),
                    end: meal.end_time.unix_timestamp(),
                    eta: current_time as i64 - 1,
                    last_modified: current_time,
                    over: false,
                    msg: None,
                    recipe: meal.name.clone(),
                    recipe_id: meal.recipe_id,
                    place: meal.place.clone(),
                    meal_id: meal.meal_id,
                    event_id,
                    allergens: recipe_allergens.clone(),
                    dietary: recipe_dietary,
                });

            status.start = meal.start_time.unix_timestamp();
            status.end = meal.end_time.unix_timestamp();
            status.recipe = meal.name.clone();
            status.recipe_id = meal.recipe_id;
            status.place = meal.place.clone();
            status.allergens = recipe_allergens;
            status.dietary = recipe_dietary;

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

    println!("Setting up Database");
    let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");
    let pg_pool = PgPool::connect(database_url)
        .await
        .expect("Failed to connect to database");
    let food_lib = FoodState::new(pg_pool);

    println!("Setting up Logging");
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE]);

    let meal_states = Arc::new(RwLock::new(HashMap::new()));
    let event_cache = Arc::new(RwLock::new(EventCache::new()));
    let feedback = Arc::new(RwLock::new(Vec::new()));

    let app_state = AppState {
        meal_states,
        event_cache,
        feedback,
        food_lib: food_lib.clone(),
    };

    start_cache_refresh_task(app_state.clone(), Duration::from_secs(300)).await;

    if let Some(next_event) = get_upcoming_event(&food_lib).await {
        let event_meals = get_event_meals_from_db(&food_lib, next_event).await;
        let allergens = fetch_allergens_for_meals(&food_lib, &event_meals).await;
        let current_time = now();

        for meal in &event_meals {
            let (recipe_allergens, recipe_dietary) = allergens
                .get(&meal.recipe_id)
                .cloned()
                .unwrap_or_default();
            app_state.meal_states.write().await.insert(
                meal.meal_id,
                MealStatus {
                    start: meal.start_time.unix_timestamp(),
                    end: meal.end_time.unix_timestamp(),
                    eta: current_time as i64 - 1,
                    last_modified: current_time,
                    over: false,
                    msg: None,
                    recipe: meal.name.clone(),
                    recipe_id: meal.recipe_id,
                    place: meal.place.clone(),
                    meal_id: meal.meal_id,
                    event_id: next_event,
                    allergens: recipe_allergens,
                    dietary: recipe_dietary,
                },
            );
        }

        let full_meals = process_meals(
            event_meals,
            app_state.meal_states.read().await.deref(),
            next_event,
            &allergens,
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

#[axum::debug_handler]
async fn get_status(
    State(state): State<AppState>,
    Query(params): Query<EventQuery>,
) -> impl IntoResponse {
    let event_id = match params.event {
        Some(id) => id,
        None => {
            let cache = state.event_cache.read().await;
            if let Some(id) = cache.upcoming_event_id {
                if cache.is_upcoming_cache_valid(Duration::from_secs(300)) {
                    id
                } else {
                    drop(cache);
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
                drop(cache);
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

    let cache_ttl = Duration::from_secs(60);
    let meals = {
        let cache = state.event_cache.read().await;
        if cache.is_event_cache_valid(event_id, cache_ttl) {
            match cache.get_event_meals(event_id) {
                Some(meals) => meals.clone(),
                None => Vec::new(),
            }
        } else {
            drop(cache);

            let event_meals = get_event_meals_from_db(&state.food_lib, event_id).await;
            if event_meals.is_empty() {
                return (
                    StatusCode::NOT_FOUND,
                    Json(Vec::<Vec<FullMealStatus>>::new()),
                );
            }

            let allergens = fetch_allergens_for_meals(&state.food_lib, &event_meals).await;
            let full_meals = process_meals(
                event_meals,
                state.meal_states.read().await.deref(),
                event_id,
                &allergens,
            );
            let mut cache = state.event_cache.write().await;
            cache.update_event_cache(event_id, full_meals.clone());
            full_meals
        }
    };

    let hour = 3600;
    let get_date_key = |time: i64| {
        let dt = OffsetDateTime::from_unix_timestamp(time - 3 * hour)
            .unwrap()
            .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC))
            .date();

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

async fn update_status(
    State(state): State<AppState>,
    Path(meal_id): Path<MealId>,
    Json(mut status): Json<MealStatus>,
) -> impl IntoResponse {
    let mut app_state = state.meal_states.write().await;
    status.last_modified = now();

    let event_id = status.event_id;
    app_state.insert(meal_id, status.clone());

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

async fn get_events(State(state): State<AppState>) -> impl IntoResponse {
    match foodlib_new::events::list_events(&state.food_lib).await {
        Ok(events) => {
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

async fn get_event_details(
    State(state): State<AppState>,
    Path(event_id): Path<EventId>,
) -> impl IntoResponse {
    let ctx = state.food_lib.status_api_ctx();

    match foodlib_new::events::get_event(&state.food_lib, &ctx, event_id).await {
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

async fn handle_feedback(
    State(state): State<AppState>,
    Json(mut feedback): Json<FeedbackStatus>,
) -> impl IntoResponse {
    println!("Received feedback: {}", feedback.feedback);
    let mut feedback_list = state.feedback.write().await;
    feedback.feedback_id = feedback_list.last().map_or(1, |f| f.feedback_id + 1);
    let log_entry = format!(
        "{} - Feedback ID: {}, Event ID: {}, Message: {}\n",
        OffsetDateTime::now_utc(),
        feedback.feedback_id,
        feedback.event_id,
        feedback.feedback
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("feedback.log")
        .expect("Failed to open feedback log file");
    write!(file, "{}", log_entry).expect("Failed to write to feedback log file");

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
