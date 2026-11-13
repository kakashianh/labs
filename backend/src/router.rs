//! Top-level router.

use axum::Router;

use crate::{middleware::logging::tracing_layer, routes, state::AppState};

/// Build the full Axum router with shared state and layers.
///
/// - Merges all route groups (currently `GET /health`).
/// - Applies `tower_http::trace::TraceLayer` for request/response logging.
/// - Applies permissive CORS so the React frontend can call the API.
pub fn router(state: AppState) -> Router {
    let health_routes = routes::health::router().with_state(state.clone());
    let building_routes = routes::buildings::router().with_state(state.clone());
    let openapi_routes = routes::openapi::router().with_state(state.clone());
    Router::new()
        .merge(health_routes)
        .merge(building_routes)
        .merge(openapi_routes)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tracing_layer())
}
