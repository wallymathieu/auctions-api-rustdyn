use sqlx::{migrate::MigrateError, PgPool};

pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    return sqlx::migrate!("./migrations")
        .run(pool)
        .await;
}