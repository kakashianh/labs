//! Top-level router.

use crate::{middleware::logging::tracing_layer, routes, state::AppState};
use axum::Router;

/// Build the full Axum router with shared state and layers.
pub fn router(state: AppState) -> Router {
    let health_routes = routes::health::router().with_state(state.clone());
    let building_routes = routes::buildings::router().with_state(state.clone());
    let building_detail_routes = routes::building_detail::router().with_state(state.clone());
    let region_routes = routes::regions::router().with_state(state.clone());
    let openapi_routes = routes::openapi::router().with_state(state.clone());
    Router::new()
        .merge(health_routes)
        .merge(building_routes)
        .merge(building_detail_routes)
        .merge(region_routes)
        .merge(openapi_routes)
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tracing_layer())
}
