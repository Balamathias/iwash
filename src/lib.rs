//! Library crate for iWash exposing reusable modules for tests and binaries.

pub mod config;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

// Handy re-exports
pub use db::Db;
pub use routes::create_api_router;
