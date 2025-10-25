use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

mod common;
use common::{create_test_app, parse_json_response, unique_email};

#[tokio::test]
async fn test_register_success() {
    let app = create_test_app();

    let payload = json!({
        "email": unique_email("newuser"),
        "password": "SecureP@ss123",
        "full_name": "New User",
        "phone": "+15551234567"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::CREATED);
    
    let body = parse_json_response(resp.into_body()).await;
    assert!(body.get("token").is_some());
    assert!(body["token"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_register_invalid_email() {
    let app = create_test_app();

    let payload = json!({
        "email": "",
        "password": "SecureP@ss123"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["error"], "bad_request");
}

#[tokio::test]
async fn test_register_short_password() {
    let app = create_test_app();

    let payload = json!({
        "email": "test@example.com",
        "password": "short"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"].as_str().unwrap().contains("password >= 8 chars"));
}

#[tokio::test]
async fn test_login_success() {
    // First register a user with unique email
    let email = unique_email("logintest");
    
    let register_payload = json!({
        "email": email,
        "password": "SecureP@ss123"
    });

    let register_req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();

    let _ = create_test_app().oneshot(register_req).await.unwrap();

    // Now login
    let login_payload = json!({
        "email": email,
        "password": "SecureP@ss123"
    });

    let login_req = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(login_req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert!(body.get("token").is_some());
    assert!(body["token"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = create_test_app();

    let payload = json!({
        "email": "nonexistent@example.com",
        "password": "WrongPassword123"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["error"], "unauthorized");
}

#[tokio::test]
async fn test_login_wrong_password() {
    // First register a user with unique email
    let email = unique_email("wrongpass");
    
    let register_payload = json!({
        "email": email,
        "password": "CorrectPassword123"
    });

    let register_req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(register_payload.to_string()))
        .unwrap();

    let _ = create_test_app().oneshot(register_req).await.unwrap();

    // Try to login with wrong password
    let login_payload = json!({
        "email": email,
        "password": "WrongPassword123"
    });

    let login_req = Request::builder()
        .uri("/api/v1/auth/login")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(login_req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
