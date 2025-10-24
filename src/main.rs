use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

use iwash::{db, routes};

#[tokio::main]
async fn main() {
    // init tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let pool = db::connect().await;

    // Build the main application router with /api/v1 prefix
    let app = Router::new()
        .route("/", get(|| async { "Welcome to iWash API 🧺" }))
        .nest("/api/v1", routes::create_api_router())
        .with_state(pool.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("🚀 iWash backend running on http://{}", addr);
    info!("📍 API endpoints available at http://{}/api/v1", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
