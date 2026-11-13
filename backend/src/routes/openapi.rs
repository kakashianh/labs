//! OpenAPI specification and interactive API documentation.
use axum::{response::{Html, IntoResponse}, routing::get, Router};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-docs/openapi.json", get(openapi_json))
        .route("/docs", get(swagger_ui))
}
async fn openapi_json() -> impl IntoResponse {
    ([("content-type", "application/json; charset=utf-8")], include_str!("../../openapi/openapi.json"))
}
async fn swagger_ui() -> Html<&'static str> {
    Html(include_str!("../../openapi/swagger-ui.html"))
}
