use axum::{routing::get, Router};

use crate::{db::Db, handlers};

pub fn routes() -> Router<Db> {
    Router::new()
        .route("/me", get(handlers::users::get_me))
        .route("/", get(handlers::users::list_users).post(handlers::users::create_user))
        .route("/{id}", get(handlers::users::get_user).patch(handlers::users::update_user).delete(handlers::users::delete_user))
}
