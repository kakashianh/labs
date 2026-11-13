//! Database helpers.
pub mod buildings;
pub mod regions;
use sqlx::PgPool;
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./src/db/migrations").run(pool).await
}
