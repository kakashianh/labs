//! Database helpers.

pub mod buildings;

use sqlx::PgPool;

/// Run embedded SQLx migrations.
///
/// Migrations live in `src/db/migrations/` as `*.sql` files and are
/// embedded into the binary via `sqlx::migrate!`, so the Docker runtime
/// image does not need to ship the `.sql` files separately.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./src/db/migrations").run(pool).await
}
