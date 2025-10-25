mod common;

use axum::http::StatusCode;
use common::{spawn_app, TestUser};
use serde_json::{json, Value};

#[tokio::test]
async fn test_vendor_registration_flow() {
    let app = spawn_app().await;

    // 1. Register as vendor
    let timestamp = chrono::Utc::now().timestamp();
    let email = format!("vendor{}@test.com", timestamp);

    let register_payload = json!({
        "full_name": "Test Vendor",
        "email": email,
        "phone": "+1234567890",
        "password": "password123",
        "role": "vendor"
    });

    let response = app
        .client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&register_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    // 2. Create vendor profile
    let vendor_payload = json!({
        "business_name": "Test Laundry Business",
        "business_description": "Professional laundry services",
        "business_email": "business@test.com",
        "business_phone": "+1234567891",
        "business_address": "123 Main St",
        "city": "Lagos",
        "state": "Lagos",
        "postal_code": "100001",
        "country": "Nigeria",
        "latitude": 6.5244,
        "longitude": 3.3792,
        "service_radius_km": 10
    });

    let response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let vendor: Value = response.json().await.unwrap();
    assert_eq!(vendor["business_name"], "Test Laundry Business");
    assert_eq!(vendor["city"], "Lagos");
    assert_eq!(vendor["is_verified"], false);
}

#[tokio::test]
async fn test_customer_cannot_create_vendor_profile() {
    let app = spawn_app().await;

    // Register as customer
    let timestamp = chrono::Utc::now().timestamp();
    let email = format!("customer{}@test.com", timestamp);

    let register_payload = json!({
        "full_name": "Test Customer",
        "email": email,
        "phone": "+1234567890",
        "password": "password123",
        "role": "customer"
    });

    let response = app
        .client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&register_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = response.json().await.unwrap();
    let token = body["token"].as_str().unwrap();

    // Try to create vendor profile
    let vendor_payload = json!({
        "business_name": "Test Laundry",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    let response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_vendors_public() {
    let app = spawn_app().await;

    // Public endpoint - no auth required
    let response = app
        .client
        .get(&format!("{}/api/v1/vendors", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert!(body["vendors"].is_array());
    assert!(body["total"].is_number());
}

#[tokio::test]
async fn test_list_vendors_with_filters() {
    let app = spawn_app().await;

    // Filter by city
    let response = app
        .client
        .get(&format!("{}/api/v1/vendors?city=Lagos", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    
    // All returned vendors should be from Lagos
    for vendor in body["vendors"].as_array().unwrap() {
        if let Some(city) = vendor["city"].as_str() {
            assert_eq!(city.to_lowercase(), "lagos");
        }
    }
}

#[tokio::test]
async fn test_list_vendors_with_search() {
    let app = spawn_app().await;

    // Create a vendor with distinctive name
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    let timestamp = chrono::Utc::now().timestamp();
    let vendor_payload = json!({
        "business_name": format!("UniqueVendor{}", timestamp),
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Search for it
    let response = app
        .client
        .get(&format!("{}/api/v1/vendors?search=UniqueVendor{}", app.address, timestamp))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    let vendors = body["vendors"].as_array().unwrap();
    assert!(vendors.len() > 0);
}

#[tokio::test]
async fn test_get_vendor_details_public() {
    let app = spawn_app().await;

    // Create a vendor
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Public Test Vendor",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    let create_response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let vendor: Value = create_response.json().await.unwrap();
    let vendor_id = vendor["id"].as_str().unwrap();

    // Get vendor details without auth (public endpoint)
    let response = app
        .client
        .get(&format!("{}/api/v1/vendors/{}", app.address, vendor_id))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let details: Value = response.json().await.unwrap();
    assert_eq!(details["business_name"], "Public Test Vendor");
}

#[tokio::test]
async fn test_get_my_vendor_profile() {
    let app = spawn_app().await;

    // Create vendor
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    let vendor_payload = json!({
        "business_name": "My Test Vendor",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Get my vendor profile
    let response = app
        .client
        .get(&format!("{}/api/v1/vendors/me", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let profile: Value = response.json().await.unwrap();
    assert_eq!(profile["business_name"], "My Test Vendor");
}

#[tokio::test]
async fn test_update_vendor_profile() {
    let app = spawn_app().await;

    // Create vendor
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Original Name",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    let create_response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let vendor: Value = create_response.json().await.unwrap();
    let vendor_id = vendor["id"].as_str().unwrap();

    // Update vendor profile
    let update_payload = json!({
        "business_name": "Updated Name",
        "business_description": "New description"
    });

    let response = app
        .client
        .patch(&format!("{}/api/v1/vendors/{}", app.address, vendor_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let updated: Value = response.json().await.unwrap();
    assert_eq!(updated["business_name"], "Updated Name");
    assert_eq!(updated["business_description"], "New description");
}

#[tokio::test]
async fn test_vendor_cannot_update_another_vendors_profile() {
    let app = spawn_app().await;

    // Create first vendor
    let vendor1 = TestUser::generate_vendor();
    vendor1.register(&app).await;
    let token1 = vendor1.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Vendor 1",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    let create_response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token1))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let vendor: Value = create_response.json().await.unwrap();
    let vendor_id = vendor["id"].as_str().unwrap();

    // Create second vendor
    let vendor2 = TestUser::generate_vendor();
    vendor2.register(&app).await;
    let token2 = vendor2.login(&app).await;

    // Try to update first vendor's profile with second vendor's token
    let update_payload = json!({
        "business_name": "Hacked Name"
    });

    let response = app
        .client
        .patch(&format!("{}/api/v1/vendors/{}", app.address, vendor_id))
        .header("Authorization", format!("Bearer {}", token2))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_vendor_stats_dashboard() {
    let app = spawn_app().await;

    // Create vendor
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Stats Test Vendor",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Get vendor stats
    let response = app
        .client
        .get(&format!("{}/api/v1/vendors/me/stats", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let stats: Value = response.json().await.unwrap();
    
    // Should have stats fields
    assert!(stats["vendor_id"].is_string());
    assert!(stats["business_name"].is_string());
    assert!(stats["total_bookings"].is_number());
    assert!(stats["pending_bookings"].is_number());
    assert!(stats["completed_bookings"].is_number());
    assert!(stats["total_revenue_cents"].is_number());
}

#[tokio::test]
async fn test_invalid_coordinates_rejected() {
    let app = spawn_app().await;

    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    // Invalid latitude
    let vendor_payload = json!({
        "business_name": "Test Vendor",
        "business_address": "123 Main St",
        "city": "Lagos",
        "latitude": 95.0,  // Invalid - must be -90 to 90
        "longitude": 3.3792
    });

    let response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_vendor_profile_rejected() {
    let app = spawn_app().await;

    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let token = vendor_user.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Test Vendor",
        "business_address": "123 Main St",
        "city": "Lagos"
    });

    // Create first vendor profile
    let response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to create second vendor profile with same user
    let response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
