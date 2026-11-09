//! Health check route.

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// Routes for health checks.
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_handler))
}

/// `GET /health` — liveness probe.
///
/// Returns `{"status":"ok","service":"backend"}` with HTTP 200.
/// The `"status":"ok"` field satisfies the Definition of Done check
/// (`GET /health` returns `{"status":"ok"}`), the extra `service`
/// field aids debugging and does not break the contract.
pub async fn health_handler() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "backend" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_handler_returns_ok_status() {
        let response = health_handler().await;
        let value = response.0;
        assert_eq!(value["status"], "ok");
        assert_eq!(value["service"], "backend");
    }

    #[tokio::test]
    async fn health_router_serves_get_health() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let pool = sqlx::PgPool::connect_lazy("postgres://localhost:5432/dummy")
            .expect("lazy pool should not fail for a well-formed URL");
        let router = router().with_state(AppState::new(pool));
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap_or_else(|_| Request::new(Body::empty())),
            )
            .await
            .unwrap_or_else(|_| axum::http::Response::new(Body::empty()));
        assert_eq!(response.status(), 200);
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap_or_default();
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("\"status\""));
        assert!(text.contains("\"ok\""));
    }
}
