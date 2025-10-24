use axum::{routing::get, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        .route("/", get(handlers::health::health_check))
}
