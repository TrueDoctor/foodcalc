use std::env;

use db::FoodBase;
use sqlx::PgPool;
use tokio;

mod api;
mod db;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool =
        PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL env var was not set"))
            .await
            .unwrap();
    // build our application with a route
    let app = api::foodbase(FoodBase::new(pool));

    // run it
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
