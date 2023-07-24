use std::env;

use db::FoodBase;
use sqlx::postgres::PgPool;

use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod api;
mod db;

use axum::{extract::State, response::IntoResponse, routing::get, Extension};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SessionLayer},
    secrecy::SecretVec,
    AuthLayer, AuthUser, PostgresStore, RequireAuthorizationLayer,
};
use rand::Rng;

#[derive(Debug, Default, Clone, sqlx::FromRow)]
struct User {
    id: i64,
    password_hash: String,
    name: String,
}

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

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool =
        PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL env var was not set"))
            .await
            .unwrap();

    let secret = rand::thread_rng().gen::<[u8; 64]>();

    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret).with_secure(false);

    let user_store = PostgresStore::<User>::new(pool.clone());
    let auth_layer = AuthLayer::new(user_store, &secret);

    let state = MyAppState {
        db_connection: FoodBase::new(pool),
    };

    async fn login_handler(mut auth: AuthContext, State(state): State<MyAppState>) {
        let pool = &state.db_connection;
        let mut conn = pool.pg_pool.acquire().await.unwrap();
        let user: User = sqlx::query_as("select * from users where id = 1")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        auth.login(&user).await.unwrap();
    }

    async fn logout_handler(mut auth: AuthContext) {
        dbg!("Logging out user: {}", &auth.current_user);
        auth.logout().await;
    }

    async fn protected_handler(Extension(user): Extension<User>) -> impl IntoResponse {
        format!("Logged in as: {}", user.name)
    }

    // build our application with a route
    let app = api::foodbase()
        .route("/protected", get(protected_handler))
        .route_layer(RequireAuthorizationLayer::<i64, User>::login())
        .route("/login", get(login_handler))
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
