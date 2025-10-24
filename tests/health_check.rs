use axum::Router;
use axum::http::{Request, StatusCode};
use axum::body::Body;
use axum::ServiceExt;

use iwash::routes::create_routes;

#[tokio::test]
async fn health_check_returns_200() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://postgres:matiecodes@localhost/iwash_db")
        .expect("failed to create lazy pool");

    let app: Router = Router::new().merge(create_routes()).with_state(pool);

    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
