//! Request logging helpers.
//!
//! Logging is provided by `tower_http::trace::TraceLayer` (see
//! `router.rs` / `main.rs`), which integrates with `tracing` so
//! request/response logs appear in the console and Docker logs.
//! This module only exposes a small factory to keep layer wiring in
//! one place.

use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::TraceLayer;

/// Build the tracing layer for HTTP request/response logging.
///
/// Uses `tracing-subscriber`'s formatting layer configured in `main.rs`.
/// The layer logs method, URI, latency and status for every request.
pub fn tracing_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}
