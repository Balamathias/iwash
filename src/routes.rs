use axum::{routing::{get, post}, Json, Router};
use serde::Serialize;

use crate::{auth::{login, register}, db::Db, middleware::AuthUser};

pub fn create_routes() -> Router<Db> {
    Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/me", get(me))
}

async fn health_check() -> &'static str {
    "✅ iWash API is healthy!"
}

#[derive(Serialize)]
struct MeResponse {
    user_id: String
}

async fn me(user: AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        user_id: user.user_id.to_string(),
    })
}
