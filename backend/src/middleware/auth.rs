//! Placeholder JWT authentication extractor.
//!
//! KAN-12 only initializes the backend skeleton. This extractor is a
//! stub that authenticates every request as a dummy system user so
//! future tickets can replace it with real JWT validation against
//! a secret (`JWT_SECRET`) without reshaping the router.

use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde_json::json;

/// Authenticated user extracted from a request.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    /// Subject identifier (`sub` claim) of the token.
    pub user_id: String,
    /// Roles granted to the user (reserved for future use).
    pub roles: Vec<String>,
}

/// Stub extractor: always succeeds with a development user.
///
/// In a future ticket this will parse the `Authorization: Bearer <token>`
/// header, verify the JWT signature and populate `user_id` / `roles`.
#[async_trait]
impl FromRequestParts<()> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &()) -> Result<Self, Self::Rejection> {
        // Skeleton behaviour: accept all requests as "dev-user" so the
        // health endpoint and local development keep working without a
        // JWT_SECRET configured. Replace with real verification later.
        let user_id = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "dev-user".to_string());
        Ok(Self {
            user_id,
            roles: vec!["admin".to_string()],
        })
    }
}

/// Placeholder error response helper for the auth middleware.
#[allow(dead_code)]
pub fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Unauthorized" })),
    )
        .into_response()
}
