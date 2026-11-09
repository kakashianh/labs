//! Shared application state.

use sqlx::PgPool;

/// Shared state available to every request handler.
///
/// `PgPool` is cheap to clone (it wraps an `Arc`) and `redis::Client`
/// is also cheap to clone, so `AppState` derives `Clone` for Axum.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub db: PgPool,
    /// Optional Redis client (configured via `REDIS_URL`).
    pub redis: Option<redis::Client>,
}

impl AppState {
    /// Create state from an existing Postgres pool.
    pub fn new(db: PgPool) -> Self {
        Self { db, redis: None }
    }

    /// Create state with an optional Redis client.
    pub fn with_redis(db: PgPool, redis: Option<redis::Client>) -> Self {
        Self { db, redis }
    }
}
