//! Binary entrypoint: start Tokio runtime, connect Postgres, run migrations,
//! bind the Axum router.

use std::time::Duration;

use backend::{config::Config, router::router, state::AppState};
use dotenvy::dotenv;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load `.env` if present (ignored when absent, e.g. inside Docker).
    dotenv().ok();

    // Initialize tracing/logs (tracing-subscriber + EnvFilter).
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    // Never log DATABASE_URL: it may contain database credentials.
    tracing::info!("configuration loaded");

    // Lazy pool: the server can bind immediately even if Postgres is
    // briefly unavailable; migrations below retry until it is ready.
    let db_options = postgres_connect_options(&config)?;
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy_with(db_options);

    tracing::info!("running migrations");
    run_migrations_with_retry(&db).await?;

    // Optional Redis client (skeleton — not yet used by handlers).
    let redis = match &config.redis_url {
        Some(url) => match redis::Client::open(url.as_str()) {
            Ok(client) => {
                // Do not log the Redis URL; it may contain credentials.
                tracing::info!("redis client configured");
                Some(client)
            }
            Err(err) => {
                tracing::warn!(error = %err, "invalid REDIS_URL, continuing without Redis");
                None
            }
        },
        None => None,
    };

    let state = AppState::with_redis(db, redis);
    let app = router(state);

    let addr = config.bind_addr();
    tracing::info!("Server lắng nghe tại {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Build PostgreSQL options from discrete environment variables when running
/// under Docker Compose. This avoids interpolating a password into a URL,
/// where reserved characters (for example @, :, / or #) must be percent-encoded.
/// For direct local development, DATABASE_URL remains supported.
fn postgres_connect_options(config: &Config) -> anyhow::Result<PgConnectOptions> {
    if let Ok(password) = std::env::var("POSTGRES_PASSWORD") {
        let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
        let port = std::env::var("POSTGRES_PORT")
            .ok()
            .map(|value| value.parse::<u16>())
            .transpose()?
            .unwrap_or(5432);
        let username = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "appuser".into());
        let database = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "appdb".into());

        Ok(PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&username)
            .password(&password)
            .database(&database))
    } else {
        Ok(config.database_url.parse()?)
    }
}

/// Run migrations with retry so `docker compose up` stays reliable when
/// Postgres takes a moment to become ready after its healthcheck passes.
async fn run_migrations_with_retry(db: &sqlx::PgPool) -> anyhow::Result<()> {
    const MAX_ATTEMPTS: u32 = 10;
    const RETRY_DELAY: Duration = Duration::from_secs(2);

    for attempt in 1..=MAX_ATTEMPTS {
        match backend::db::run_migrations(db).await {
            Ok(()) => {
                tracing::info!("migrations applied successfully");
                return Ok(());
            }
            Err(err) => {
                if attempt == MAX_ATTEMPTS {
                    tracing::error!(error = %err, "migrations failed after {} attempts", MAX_ATTEMPTS);
                    return Err(err.into());
                }
                tracing::warn!(error = %err, attempt, "migrations attempt failed, retrying in {}s", RETRY_DELAY.as_secs());
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }

    unreachable!("loop always returns or retries")
}
