use axum::{extract::State, routing::{get, post}, Json, Router};
use serde::Serialize;

use crate::{auth::{login, register}, db::Db, errors::AppResult, middleware::AuthUser, users};

pub fn create_routes() -> Router<Db> {
    Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/me", get(me))
    .nest("/users", users::router())
}

async fn health_check() -> &'static str {
    "✅ iWash API is healthy!"
}

#[derive(Serialize)]
struct MeResponse {
    id: String,
    email: String,
    full_name: Option<String>,
    phone: Option<String>,
}

async fn me(State(db): State<Db>, user: AuthUser) -> AppResult<Json<MeResponse>> {
    let (id, email, full_name, phone): (uuid::Uuid, String, Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT id, email, full_name, phone FROM users WHERE id = $1",
        )
        .bind(user.user_id)
        .fetch_one(&db)
        .await?;

    Ok(Json(MeResponse {
        id: id.to_string(),
        email,
        full_name,
        phone,
    }))
}
