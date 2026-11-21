//! Tiny helper to run one-off SQL files against the configured database
//! (used for ad-hoc schema changes when provisioning local dev databases).
//!
//! Usage (from `server/`): `cargo run --example db_exec -- path/to/file.sql [...]`

use sqlx::PgPool;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url).await?;

    for path in std::env::args().skip(1) {
        let sql = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {path}: {err}"));
        for (i, statement) in sql
            .split(';')
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
            .enumerate()
        {
            let result = sqlx::query(statement).execute(&pool).await?;
            println!("{path} #{}: rows_affected={}", i + 1, result.rows_affected());
        }
    }
    Ok(())
}
