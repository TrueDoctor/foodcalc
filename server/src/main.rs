use std::{env, net::SocketAddr, ops::Deref, sync::Arc, time::Duration};

use axum::{Extension, Router};
use axum_login::{
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use fern::colors::{Color, ColoredLevelConfig};
use foodlib::FoodBase;
use sqlx::postgres::PgPool;
use time::format_description;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::cookie::Key;
use tower_sessions_sqlx_store::PostgresStore;

use foodlib_new::auth::AuthBackend;
mod frontend;
mod htmx_middleware;

type FoodLib = Extension<foodlib_new::FoodLib>;

#[derive(Clone)]
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
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");
    let pool = PgPool::connect(&database_url).await.unwrap();

    let port = &env::var("PORT").unwrap_or("3000".to_string());
    let colors = ColoredLevelConfig::new()
        .debug(Color::Magenta)
        .info(Color::Green)
        .error(Color::Red);

    let format = format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap();
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
                colors.color(record.level()),
                time::OffsetDateTime::now_utc().format(&format).unwrap(),
                record.module_path().unwrap_or("<unnamed>"),
                message
            ))
        })
        .apply()
        .unwrap();

    // Set up session store
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.unwrap();

    // Set up session deletion task
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(Duration::from_secs(60)),
    );

    // Generate a signing key for cookies
    let key = Key::generate();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)))
        .with_signed(key);

    // Create auth backend and layer
    let backend = AuthBackend::new(Arc::new(pool.clone()));
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let state = MyAppState {
        db_connection: FoodBase::new_with_pool(pool),
    };
    let new_lib = foodlib_new::FoodLib::from_shared(state.pool_arc());

    // Combine routes with middleware
    let app = Router::new()
        .merge(frontend::frontend_router())
        .with_state(state)
        .layer(Extension(new_lib))
        .layer(axum::middleware::from_fn(htmx_middleware::htmx_middleware))
        .layer(auth_layer)
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http());

    println!("Listening on http://localhost:{port}");

    // Start server with graceful shutdown for session cleanup
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await
        .unwrap();

    deletion_task.await.unwrap().unwrap();
}

async fn shutdown_signal(deletion_task_abort_handle: tokio::task::AbortHandle) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            deletion_task_abort_handle.abort();
        },
        _ = terminate => {
            deletion_task_abort_handle.abort();
        },
    }
}
