use axum::{routing::get, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        .route("/", get(handlers::services::list_services))
        .route("/{id}", get(handlers::services::get_service))
}
