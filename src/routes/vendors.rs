use axum::{routing::{delete, get, patch, post}, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        // Public endpoints (no auth required)
        .route("/", get(handlers::vendors::list_vendors))
        .route("/{id}", get(handlers::vendors::get_vendor))
        .route("/{id}/services", get(handlers::services::list_services_by_vendor))
        .route("/{id}/reviews", get(handlers::vendors::list_vendor_reviews))
        // Protected endpoints (auth required)
        .route("/", post(handlers::vendors::create_vendor))
        .route("/me", get(handlers::vendors::get_my_vendor))
        .route("/me/stats", get(handlers::vendors::get_vendor_stats))
        .route("/{id}", patch(handlers::vendors::update_vendor))
        // Vendor service management (vendor role required)
        .route("/me/services", get(handlers::services::list_vendor_services))
        .route("/me/services", post(handlers::services::create_vendor_service))
        .route("/me/services/{id}", patch(handlers::services::update_vendor_service))
        .route("/me/services/{id}", delete(handlers::services::delete_vendor_service))
        // Vendor review responses (vendor role required)
        .route("/me/reviews/{id}/response", post(handlers::reviews::vendor_respond_to_review))
}
