use axum::{extract::State, Json};
use serde::Serialize;

use crate::db::Db;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
    pub version: &'static str,
}

pub async fn health_check(State(db): State<Db>) -> Json<HealthResponse> {
    // Test database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_one(&db).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    Json(HealthResponse {
        status: "healthy",
        database: db_status,
        version: env!("CARGO_PKG_VERSION"),
    })
}
