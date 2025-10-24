use axum::Router;
use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::ServiceExt;

use iwash::routes::create_api_router;

#[tokio::test]
async fn health_check_returns_200() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://postgres:matiecodes@localhost/iwash_db")
        .expect("failed to create lazy pool");

    let app: Router = Router::new()
        .nest("/api/v1", create_api_router())
        .with_state(pool);

    let req = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
