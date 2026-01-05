use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_pool() -> PgPool {
    let database_url = dotenv()
        .ok()
        .and_then(|_| std::env::var("DATABASE_URL").ok())
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database")
}
