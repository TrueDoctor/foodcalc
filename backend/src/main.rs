use std::env;

use fern::colors::{Color, ColoredLevelConfig};
use sqlx::postgres::PgPool;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use foodlib::{AuthContext, Credenitals, FoodBase, User};

mod api;
mod frontend;

use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::get, routing::post,
    Extension, Json,
};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    AuthLayer, PostgresStore, RequireAuthorizationLayer,
};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct MyAppState {
    db_connection: FoodBase,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool =
        PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL env var was not set"))
            .await
            .unwrap();

    let colors = ColoredLevelConfig::new()
        .debug(Color::Magenta)
        .info(Color::Green)
        .error(Color::Red);

    fern::Dispatch::new()
        .chain(std::io::stdout())
        .level_for("axum", log::LevelFilter::Info)
        .level_for("mio", log::LevelFilter::Info)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("backend", log::LevelFilter::Trace)
        .level_for("sqlx", log::LevelFilter::Trace)
        .level(log::LevelFilter::Debug)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}]{}{} {}",
                // This will color the log level only, not the whole line. Just a touch.
                colors.color(record.level()),
                chrono::Utc::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.module_path().unwrap_or("<unnamed>"),
                message
            ))
        })
        .apply()
        .unwrap();

    let secret = rand::thread_rng().gen::<[u8; 64]>();

    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret).with_secure(false);

    let user_store = PostgresStore::<User>::new(pool.clone());
    let auth_layer = AuthLayer::new(user_store, &secret);

    let state = MyAppState {
        db_connection: FoodBase::new_with_pool(pool),
    };

    async fn protected_handler(Extension(user): Extension<User>) -> impl IntoResponse {
        format!("Logged in as: {}", user.username)
    }

    // build our application with a route
    let app = axum::Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(RequireAuthorizationLayer::<i64, User>::login())
        .nest("/api", api::foodbase())
        .nest("/", frontend::frontend_router())
        .layer(auth_layer)
        .layer(session_layer)
        .with_state(state)
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http());

    println!("Listening on http://localhost:3000");

    // run it
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
