use axum::{routing::post, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
}
