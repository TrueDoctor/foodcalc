//! One-off helper for the SPA smoke test (run from `server/`):
//!
//! ```bash
//! cargo run --example spa_smoke_user            # create the throwaway user
//! cargo run --example spa_smoke_user -- cleanup # remove it again
//! ```
//!
//! Creates a personal throwaway account in the configured (test) database
//! so the `/api/auth/*` endpoints can be validated end-to-end with curl.

use sqlx::PgPool;

const USERNAME: &str = "spa_smoke_test";
const PASSWORD: &str = "spa_smoke_1234";
const EMAIL: &str = "spa-smoke-test@example.com";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url).await?;
    let lib = foodlib_new::FoodLib::new(pool.clone());
    let cleanup = std::env::args().any(|arg| arg == "cleanup");

    if cleanup {
        match lib
            .users()
            .get_user_by_string_reference(USERNAME.to_string())
            .await
        {
            Some(user) => {
                lib.users().delete_user_cascade(user.id).await?;
                println!("cleanup: removed user {USERNAME} (id {})", user.id);
            }
            None => println!("cleanup: user {USERNAME} not found, nothing to do"),
        }
        return Ok(());
    }

    if lib
        .users()
        .get_user_by_string_reference(USERNAME.to_string())
        .await
        .is_some()
    {
        println!("user {USERNAME} already exists — nothing to create");
        return Ok(());
    }

    let backend = foodlib_new::auth::AuthBackend::new(std::sync::Arc::new(pool));
    let user = foodlib_new::auth::register(
        &backend,
        EMAIL.to_string(),
        foodlib_new::auth::Credentials {
            username: USERNAME.to_string(),
            password: PASSWORD.to_string(),
        },
        false,
    )
    .await?;
    println!(
        "created user {} (id {}) with personal group",
        user.username, user.id
    );
    Ok(())
}
