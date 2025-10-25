use axum::{routing::{get, patch, post}, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        // Public endpoints (no auth required)
        .route("/", get(handlers::vendors::list_vendors))
        .route("/{id}", get(handlers::vendors::get_vendor))
        // Protected endpoints (auth required)
        .route("/", post(handlers::vendors::create_vendor))
        .route("/me", get(handlers::vendors::get_my_vendor))
        .route("/me/stats", get(handlers::vendors::get_vendor_stats))
        .route("/{id}", patch(handlers::vendors::update_vendor))
}
