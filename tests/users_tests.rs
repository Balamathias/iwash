use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

mod common;
use common::{create_test_app, parse_json_response, unique_email, register_and_get_token};

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
    assert!(body["users"].is_array());
    assert!(body["users"].as_array().unwrap().len() > 0);
    assert_eq!(body["page"], 1);
    assert_eq!(body["limit"], 10); // default limit
    assert!(body["total"].as_i64().unwrap() > 0);
    assert!(body["total_pages"].as_u64().unwrap() > 0);
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

#[tokio::test]
async fn test_list_users_with_pagination() {
    let token = register_and_get_token(&unique_email("paginationtest"), "SecureP@ss123").await;

    // Test with custom page and limit
    let req = Request::builder()
        .uri("/api/v1/users?page=1&limit=5")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["page"], 1);
    assert_eq!(body["limit"], 5);
    assert!(body["users"].as_array().unwrap().len() <= 5);
}

#[tokio::test]
async fn test_list_users_with_search() {
    // Create a user with a unique searchable email
    let search_email = unique_email("searchableuser");
    let token = register_and_get_token(&search_email, "SecureP@ss123").await;

    // Search for the user
    let req = Request::builder()
        .uri(&format!("/api/v1/users?search=searchableuser"))
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    let users = body["users"].as_array().unwrap();
    
    // Should find at least the user we created
    assert!(users.len() > 0);
    
    // Verify search worked - at least one user should match
    let found = users.iter().any(|u| {
        u["email"].as_str().unwrap().contains("searchableuser")
    });
    assert!(found, "Search should find the user with 'searchableuser' in email");
}

#[tokio::test]
async fn test_list_users_search_by_name() {
    // Create a user with a specific full name
    let token = register_and_get_token(&unique_email("searchname"), "SecureP@ss123").await;

    // The register_and_get_token creates users with full_name "Test User"
    // Search for "Test"
    let req = Request::builder()
        .uri("/api/v1/users?search=Test")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    let users = body["users"].as_array().unwrap();
    
    // Should find users with "Test" in their name
    assert!(users.len() > 0);
}

#[tokio::test]
async fn test_list_users_limit_cap() {
    let token = register_and_get_token(&unique_email("limitcaptest"), "SecureP@ss123").await;

    // Try to request more than max limit (100)
    let req = Request::builder()
        .uri("/api/v1/users?limit=200")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    // Should be capped at 100
    assert_eq!(body["limit"], 100);
}
