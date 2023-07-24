use std::env;

use chrono::{DateTime, Utc, NaiveDateTime};
use fern::colors::{Color, ColoredLevelConfig};
use db::FoodBase;
use serde::{Serialize, Deserialize};
use sqlx::postgres::PgPool;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod api;
mod db;

use axum::{extract::State, response::IntoResponse, routing::get, routing::post, Extension, http::StatusCode, Json};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    secrecy::SecretVec,
    AuthLayer, AuthUser, PostgresStore, RequireAuthorizationLayer,
};
use rand::Rng;

impl AuthUser<i64> for User {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }
}

type AuthContext = axum_login::extractors::AuthContext<i64, User, PostgresStore<User>>;

#[derive(Debug, Clone)]
pub struct MyAppState {
    db_connection: FoodBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Credenitals {
    username: String,
    password: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
    is_admin: bool,
    created_at: NaiveDateTime,
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
        db_connection: FoodBase::new(pool),
    };

    async fn login_handler(mut auth: AuthContext, State(state): State<MyAppState>, Json(credentials): Json<Credenitals>) -> Result<(), StatusCode> {
        log::info!("Logging in user: {}", &credentials.username);
        let pool = &state.db_connection;
        let mut conn = pool.pg_pool.acquire().await.unwrap();
        let user = sqlx::query_as!(User, "select * from users where username = $1", credentials.username)
            .fetch_one(&mut conn)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        auth.login(&user).await.unwrap();
        Ok(())
    }

    async fn logout_handler(mut auth: AuthContext) {
        dbg!("Logging out user: {}", &auth.current_user);
        auth.logout().await;
    }

    async fn protected_handler(Extension(user): Extension<User>) -> impl IntoResponse {
        format!("Logged in as: {}", user.username)
    }

    // build our application with a route
    let app = api::foodbase()
        .route("/protected", get(protected_handler))
        .route_layer(RequireAuthorizationLayer::<i64, User>::login())
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
        .with_state(state)
        .layer(auth_layer)
        .layer(session_layer)
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http());

    println!("Listening on http://localhost:3000");

    // run it
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
