use axum::{routing::{delete, get, patch, post}, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        .route("/", post(handlers::bookings::create_booking))
        .route("/", get(handlers::bookings::list_bookings))
        .route("/{id}", get(handlers::bookings::get_booking))
        .route("/{id}", patch(handlers::bookings::update_booking))
        .route("/{id}/cancel", delete(handlers::bookings::cancel_booking))
        .route("/vendor", get(handlers::bookings::list_vendor_bookings))
        .route("/vendor/{id}", patch(handlers::bookings::update_booking_status_vendor))
}
