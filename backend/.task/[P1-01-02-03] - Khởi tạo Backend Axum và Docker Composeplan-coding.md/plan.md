# KAN-12 — Khởi tạo Backend Rust Axum và Docker Compose

## Mục tiêu

Khởi tạo dự án backend bằng **Rust + Axum**, thiết lập cấu trúc thư mục chuẩn, tích hợp Docker Compose để chạy toàn bộ stack cục bộ (Postgres, Redis, backend).

---

## Công nghệ sử dụng

| Thành phần | Lựa chọn |
|---|---|
| Ngôn ngữ | Rust (stable) |
| Web framework | Axum 0.7 |
| Async runtime | Tokio |
| ORM / Query builder | SQLx (async, compile-time checked) |
| Serialization | Serde + serde_json |
| Validation | validator |
| Tracing / Logging | tracing + tracing-subscriber |
| Config | config-rs hoặc dotenvy |
| Database | PostgreSQL 16 |
| Cache | Redis 7 |
| Container | Docker + Docker Compose v2 |

---

## Cấu trúc thư mục

```
backend/
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── Dockerfile
├── docker-compose.yml          # (tại root project)
└── src/
    ├── main.rs                 # Entrypoint: khởi Tokio runtime, bind Axum router
    ├── lib.rs                  # Re-export các module chính
    ├── config.rs               # Đọc biến môi trường / file config
    ├── error.rs                # AppError enum + IntoResponse
    ├── state.rs                # AppState (db pool, redis, config)
    ├── router.rs               # Ghép tất cả route
    ├── middleware/
    │   ├── auth.rs             # JWT extractor middleware
    │   └── logging.rs          # Request/response tracing layer
    ├── routes/
    │   ├── health.rs           # GET /health
    │   └── mod.rs
    └── db/
        ├── mod.rs
        └── migrations/         # SQLx migration files (.sql)
```

---

## Các bước thực hiện

### Bước 1 — Khởi tạo Cargo project

```bash
cargo new backend --name backend
cd backend
```

Thêm dependencies vào `Cargo.toml`:

```toml
[dependencies]
axum          = { version = "0.7", features = ["macros"] }
tokio         = { version = "1", features = ["full"] }
tower         = "0.4"
tower-http    = { version = "0.5", features = ["cors", "trace"] }
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
sqlx          = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "macros", "uuid", "chrono"] }
redis         = { version = "0.25", features = ["tokio-comp"] }
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy       = "0.15"
validator     = { version = "0.18", features = ["derive"] }
uuid          = { version = "1", features = ["v4", "serde"] }
chrono        = { version = "0.4", features = ["serde"] }
anyhow        = "1"
thiserror     = "1"
```

---

### Bước 2 — Viết `main.rs` và `state.rs`

**`src/state.rs`** — Shared application state:

```rust
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    // pub redis: redis::Client,
}
```

**`src/main.rs`**:

```rust
use axum::Router;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod middleware;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL phải được set");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Chạy migration tự động
    sqlx::migrate!("./src/db/migrations").run(&db).await?;

    let state = AppState { db };

    let app = Router::new()
        .merge(routes::health::router())
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Server lắng nghe tại {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

### Bước 3 — Health check endpoint

**`src/routes/health.rs`**:

```rust
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "backend" }))
}
```

---

### Bước 4 — Error handling

**`src/error.rs`**:

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Database(_) | AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

---

### Bước 5 — Dockerfile

```dockerfile
# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/backend*

# Build thực sự
COPY src ./src
RUN cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/backend ./backend

EXPOSE 8000
ENV RUST_LOG=backend=info,tower_http=info

CMD ["./backend"]
```

---

### Bước 6 — Docker Compose

**`docker-compose.yml`** (đặt tại root project):

```yaml
version: "3.9"

services:
  postgres:
    image: postgres:16-alpine
    restart: unless-stopped
    environment:
      POSTGRES_USER: ${POSTGRES_USER:-appuser}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-apppassword}
      POSTGRES_DB: ${POSTGRES_DB:-appdb}
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    restart: unless-stopped
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    restart: unless-stopped
    env_file: ./backend/.env
    environment:
      DATABASE_URL: postgres://${POSTGRES_USER:-appuser}:${POSTGRES_PASSWORD:-apppassword}@postgres:5432/${POSTGRES_DB:-appdb}
      REDIS_URL: redis://redis:6379
    ports:
      - "8000:8000"
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

volumes:
  postgres_data:
```

---

### Bước 7 — File `.env.example`

```dotenv
DATABASE_URL=postgres://appuser:apppassword@localhost:5432/appdb
REDIS_URL=redis://localhost:6379
RUST_LOG=backend=debug,tower_http=debug

POSTGRES_USER=appuser
POSTGRES_PASSWORD=apppassword
POSTGRES_DB=appdb
```

---

### Bước 8 — SQLx Migration đầu tiên

```bash
# Cài sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Tạo migration
sqlx migrate add init_schema

# Chạy migration
sqlx migrate run
```

---

## Tiêu chí hoàn thành (Definition of Done)

- [x] `cargo build --release` không có lỗi / warning
- [x] `GET /health` trả về `{"status":"ok"}` với HTTP 200
- [ ] `docker compose up` khởi động thành công Postgres, Redis, backend
- [ ] Backend kết nối được Postgres qua SQLx pool
- [ ] Migration chạy tự động khi container start
- [ ] Tracing logs hiển thị request/response đầy đủ
- [x] File `.env.example` đã được commit, `.env` đã có trong `.gitignore`

### Kết quả kiểm tra (Verification — 2026-11-09, sandbox)

Đã kiểm tra trong sandbox (Rust 1.99.1, Cargo 1.99.1):

- [x] `cargo build --release` thành công, không có warning từ crate `backend`
  (chỉ có `future-incompat` của dependency `sqlx-postgres v0.7.4`, không phải code dự án).
  Nhị phân `backend/target/release/backend` ~6.6 MB.
  `cargo fmt --check` sạch, `cargo clippy --all-targets --all-features -- -D warnings` sạch,
  `cargo test` 8/8 tests pass (config 2, error 4, health 2).
- [x] `GET /health` trả HTTP 200, body `{"status":"ok","service":"backend"}`
  (superset của yêu cầu `{"status":"ok"}`) — kiểm chứng qua
  `cargo test health_router_serves_get_health` (oneshot qua `Router` + `TraceLayer`)
  và `health_handler_returns_ok_status`.
- [x] File `.env.example` đã tạo tại `backend/.env.example`, `.env` đã có trong `.gitignore`
  (root `.gitignore` + `backend/.dockerignore` đã tạo, `docker compose config --quiet` OK).
  Lưu ý: các file mới hiện là untracked, cần `git add`/`commit` mới tính là committed.
- [ ] `docker compose up`, kết nối Postgres qua SQLx, migration tự động và tracing request/response
  chưa kiểm chứng được trong sandbox: Docker daemon không truy cập được
  (`permission denied /var/run/docker.sock`, user không thuộc group `docker`, sudo bị chặn `no new privileges`),
  và chưa có Postgres live để chạy migration. `docker-compose.yml` chỉ được kiểm chứng bằng
  `docker compose config` (YAML hợp lệ). Code đã triển khai `PgPoolOptions::connect_lazy` + retry migrations
  và `TraceLayer::new_for_http()` + `tracing-subscriber`, nhưng cần chạy `docker compose up --build`
  trên máy local để xác thực các mục này trước khi tick thêm.

---

## Lưu ý kỹ thuật

- Dùng `sqlx::migrate!` macro để embed migration vào binary — không cần copy file `.sql` riêng vào Docker image.
- `AppState` phải implement `Clone` vì Axum clone state cho mỗi request.
- Dùng `tower-http::TraceLayer` thay vì middleware logging tự viết để tích hợp sẵn với `tracing`.
- Build multi-stage trong Dockerfile giúp image runtime nhỏ (~20 MB) so với image dev (~1.5 GB).
