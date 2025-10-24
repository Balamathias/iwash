use axum::Router;

use crate::db::Db;

pub mod auth;
pub mod health;
pub mod users;

/// Creates the versioned API router with all routes
pub fn create_api_router() -> Router<Db> {
    Router::new()
        .nest("/health", health::routes())
        .nest("/auth", auth::routes())
        .nest("/users", users::routes())
}
