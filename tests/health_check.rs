use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::ServiceExt;

mod common;
use common::create_test_app;

#[tokio::test]
async fn health_check_returns_200() {
    let app = create_test_app();

    let req = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
