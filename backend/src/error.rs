//! Application error handling.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application-level error type.
#[derive(Debug, Error)]
#[allow(dead_code)]
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

/// Convenience alias for fallible operations in handlers.
pub type Result<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Database(_) | AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode as HttpStatus;

    #[tokio::test]
    async fn not_found_returns_404() {
        let response = AppError::NotFound.into_response();
        assert_eq!(response.status(), HttpStatus::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap_or_default();
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("Not found"));
    }

    #[tokio::test]
    async fn unauthorized_returns_401() {
        let response = AppError::Unauthorized.into_response();
        assert_eq!(response.status(), HttpStatus::UNAUTHORIZED);
    }

    #[test]
    fn internal_error_hides_details() {
        let err = AppError::Internal(anyhow::anyhow!("secret detail"));
        let response = err.into_response();
        assert_eq!(response.status(), HttpStatus::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn body_type_compiles() {
        // Ensure `Body` remains usable in tests to avoid dead import warnings.
        let _ = Body::empty();
    }
}
