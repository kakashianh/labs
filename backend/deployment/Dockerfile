# SQLx 0.7.4 relies on the previous never-type fallback behavior.
# Rust 1.100 changes that behavior for all editions; pin 1.99.1 until SQLx is upgraded/fixed.
FROM rust:1.99.1-slim-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/backend*

# Build application
COPY openapi ./openapi
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
