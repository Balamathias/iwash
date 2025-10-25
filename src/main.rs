use axum::{middleware, routing::get, Router};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

use iwash::{db, middleware::request_id_middleware, routes};

#[tokio::main]
async fn main() {
    // init tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let pool = db::connect().await;

    // Configure CORS for React Native frontend
    let cors = CorsLayer::new()
        .allow_origin(Any) // In production, specify exact origins
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the main application router with /api/v1 prefix
    let app = Router::new()
        .route("/", get(|| async { "Welcome to iWash API 🧺" }))
        .nest("/api/v1", routes::create_api_router())
        .with_state(pool.clone())
        .layer(middleware::from_fn(request_id_middleware))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("🚀 iWash backend running on http://{}", addr);
    info!("📍 API endpoints available at http://{}/api/v1", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
