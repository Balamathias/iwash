use axum::Router;

use crate::db::Db;

pub mod auth;
pub mod bookings;
pub mod health;
pub mod services;
pub mod users;
pub mod vendors;

/// Creates the versioned API router with all routes
pub fn create_api_router() -> Router<Db> {
    Router::new()
        .nest("/health", health::routes())
        .nest("/auth", auth::routes())
        .nest("/users", users::routes())
        .nest("/services", services::routes())
        .nest("/bookings", bookings::routes())
        .nest("/vendors", vendors::routes())
}
