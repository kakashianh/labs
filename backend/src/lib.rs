//! Backend library crate.
//!
//! Re-export the main modules so `src/main.rs` and integration tests
//! share the same code (`config`, `error`, `state`, `router`, …).

pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod state;
