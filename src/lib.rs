//! Library crate for iWash exposing reusable modules for tests and binaries.

pub mod db;
pub mod routes;
pub mod models;
pub mod auth;
pub mod middleware;
pub mod errors;
pub mod users;

// Handy re-exports
pub use db::Db;
pub use routes::create_routes;
