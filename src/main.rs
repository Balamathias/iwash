use axum::{routing::get, Router};
use std::net::SocketAddr;
use crate::routes::create_routes;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod db;
mod routes;
mod models;
mod auth;
mod middleware;
mod errors;

#[tokio::main]
async fn main() {
    // init tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let pool = db::connect().await;

    let app = Router::new()
        .route("/", get(|| async { "Welcome to iWash API 🧺" }))
        .merge(create_routes())
        .with_state(pool.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("🚀 iWash backend running on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
