use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

use iwash::routes::create_api_router;

/// Helper to create a test app with lazy pool
fn create_test_app() -> Router {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://postgres:matiecodes@localhost/iwash_db")
        .expect("failed to create lazy pool");

    Router::new()
        .nest("/api/v1", create_api_router())
        .with_state(pool)
}

/// Helper to parse JSON response body
async fn parse_json_response(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Generate a unique email for testing
fn unique_email(base: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}+{}@example.com", base, timestamp)
}

/// Helper to register and get a token
async fn register_and_get_token(email: &str, password: &str) -> String {
    let payload = json!({
        "email": email,
        "password": password,
        "full_name": "Test User"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    let body = parse_json_response(resp.into_body()).await;
    body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_get_me_success() {
    let email = unique_email("metest");
    let token = register_and_get_token(&email, "SecureP@ss123").await;

    let req = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["email"], email);
    assert_eq!(body["full_name"], "Test User");
    assert!(body.get("id").is_some());
}

#[tokio::test]
async fn test_get_me_unauthorized() {
    let req = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_me_invalid_token() {
    let req = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .header("authorization", "Bearer invalid_token_here")
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_users_success() {
    let token = register_and_get_token(&unique_email("listuser"), "SecureP@ss123").await;

    let req = Request::builder()
        .uri("/api/v1/users")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert!(body.is_array());
    assert!(body.as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_list_users_unauthorized() {
    let req = Request::builder()
        .uri("/api/v1/users")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_user_success() {
    let token = register_and_get_token(&unique_email("adminuser"), "SecureP@ss123").await;

    let payload = json!({
        "email": unique_email("createduser"),
        "password": "AnotherPass123",
        "full_name": "Created User",
        "phone": "+15559876543"
    });

    let req = Request::builder()
        .uri("/api/v1/users")
        .method("POST")
        .header("authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::CREATED);
    
    let body = parse_json_response(resp.into_body()).await;
    // Email will be unique, just verify it exists
    assert!(body["email"].as_str().unwrap().contains("createduser"));
    assert_eq!(body["full_name"], "Created User");
    assert_eq!(body["phone"], "+15559876543");
    assert!(body.get("id").is_some());
}

#[tokio::test]
async fn test_create_user_unauthorized() {
    let payload = json!({
        "email": "test@example.com",
        "password": "Password123"
    });

    let req = Request::builder()
        .uri("/api/v1/users")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_user_by_id_success() {
    let email = unique_email("getusertest");
    let token = register_and_get_token(&email, "SecureP@ss123").await;

    // First get "me" to get the user ID
    let me_req = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let me_resp = create_test_app().oneshot(me_req).await.unwrap();
    let me_body = parse_json_response(me_resp.into_body()).await;
    let user_id = me_body["id"].as_str().unwrap();

    // Now get user by ID
    let req = Request::builder()
        .uri(&format!("/api/v1/users/{}", user_id))
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["email"], email);
    assert_eq!(body["id"], user_id);
}

#[tokio::test]
async fn test_update_user_success() {
    let email = unique_email("updatetest");
    let token = register_and_get_token(&email, "SecureP@ss123").await;

    // Get user ID
    let me_req = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let me_resp = create_test_app().oneshot(me_req).await.unwrap();
    let me_body = parse_json_response(me_resp.into_body()).await;
    let user_id = me_body["id"].as_str().unwrap();

    // Update user
    let update_payload = json!({
        "full_name": "Updated Name",
        "phone": "+15551111111"
    });

    let update_req = Request::builder()
        .uri(&format!("/api/v1/users/{}", user_id))
        .method("PATCH")
        .header("authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(update_payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(update_req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["full_name"], "Updated Name");
    assert_eq!(body["phone"], "+15551111111");
    assert_eq!(body["email"], email); // Email unchanged
}

#[tokio::test]
async fn test_delete_user_success() {
    let token = register_and_get_token(&unique_email("deletetest"), "SecureP@ss123").await;

    // Get user ID
    let me_req = Request::builder()
        .uri("/api/v1/users/me")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let me_resp = create_test_app().oneshot(me_req).await.unwrap();
    let me_body = parse_json_response(me_resp.into_body()).await;
    let user_id = me_body["id"].as_str().unwrap();

    // Delete user
    let delete_req = Request::builder()
        .uri(&format!("/api/v1/users/{}", user_id))
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(delete_req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_nonexistent_user() {
    let token = register_and_get_token(&unique_email("deletenonexistent"), "SecureP@ss123").await;

    // Try to delete a non-existent user (using a fake UUID)
    let fake_id = "00000000-0000-0000-0000-000000000000";
    
    let delete_req = Request::builder()
        .uri(&format!("/api/v1/users/{}", fake_id))
        .method("DELETE")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(delete_req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
