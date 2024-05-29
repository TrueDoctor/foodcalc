use std::{env, ops::Deref};

use fern::colors::{Color, ColoredLevelConfig};
use sqlx::postgres::PgPool;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use foodlib::{FoodBase, User};

mod frontend;

use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    AuthLayer, PostgresStore,
};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct MyAppState {
    db_connection: FoodBase,
}

impl Deref for MyAppState {
    type Target = FoodBase;

    fn deref(&self) -> &Self::Target {
        &self.db_connection
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool =
        PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL env var was not set"))
            .await
            .unwrap();

    let port = &env::var("PORT").unwrap_or("3000".to_string());
    let colors = ColoredLevelConfig::new()
        .debug(Color::Magenta)
        .info(Color::Green)
        .error(Color::Red);

    fern::Dispatch::new()
        .chain(std::io::stdout())
        .level_for("axum", log::LevelFilter::Trace)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("mio", log::LevelFilter::Info)
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

    // build our application with a route
    let app = axum::Router::new()
        .nest("/", frontend::frontend_router())
        .layer(auth_layer)
        .layer(session_layer)
        .with_state(state)
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http());

    println!("Listening on http://localhost:{port}");

    // run it
    axum::Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
