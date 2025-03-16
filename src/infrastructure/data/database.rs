use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn create_pg_pool(connection_string: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(connection_string)
        .await
}
