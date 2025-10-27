mod common;

use axum::http::StatusCode;
use common::{spawn_app, TestUser};
use serde_json::{json, Value};

#[tokio::test]
async fn vendor_can_create_service() {
    let app = spawn_app().await;

    // Create vendor user and profile
    let vendor = TestUser::generate_vendor();
    vendor.register(&app).await;
    let token = vendor.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Test Laundry",
        "business_address": "123 Test St",
        "city": "Lagos",
        "state": "Lagos",
        "country": "Nigeria",
        "latitude": 6.5244,
        "longitude": 3.3792
    });

    let create_vendor_resp = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_vendor_resp.status(), StatusCode::CREATED);

    // Create a service
    let service_payload = json!({
        "name": "Express Wash",
        "description": "Fast laundry service",
        "base_price_cents": 5000,
        "price_per_kg_cents": 2000,
        "estimated_duration_hours": 24
    });

    let create_service_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_service_resp.status(), StatusCode::CREATED);

    let service: Value = create_service_resp.json().await.unwrap();
    assert_eq!(service["name"].as_str().unwrap(), "Express Wash");
    assert_eq!(service["base_price_cents"].as_i64().unwrap(), 5000);
    assert_eq!(service["price_per_kg_cents"].as_i64().unwrap(), 2000);
    assert_eq!(service["is_active"].as_bool().unwrap(), true);
    assert!(service["vendor_id"].is_string());
}

#[tokio::test]
async fn vendor_can_list_their_services() {
    let app = spawn_app().await;

    let vendor = TestUser::generate_vendor();
    vendor.register(&app).await;
    let token = vendor.login(&app).await;

    // Create vendor profile
    let vendor_payload = json!({
        "business_name": "Test Laundry",
        "business_address": "123 Test St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Create two services
    let service1 = json!({
        "name": "Service 1",
        "base_price_cents": 3000,
        "price_per_kg_cents": 1000,
        "estimated_duration_hours": 12
    });

    let service2 = json!({
        "name": "Service 2",
        "base_price_cents": 5000,
        "price_per_kg_cents": 2000,
        "estimated_duration_hours": 24
    });

    for payload in [service1, service2] {
        app.client
            .post(&format!("{}/api/v1/vendors/me/services", app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await
            .expect("Failed to execute request");
    }

    // List services
    let list_resp = app
        .client
        .get(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(list_resp.status(), StatusCode::OK);

    let services: Value = list_resp.json().await.unwrap();
    let services_array = services.as_array().unwrap();
    assert_eq!(services_array.len(), 2);
}

#[tokio::test]
async fn vendor_can_update_their_service() {
    let app = spawn_app().await;

    let vendor = TestUser::generate_vendor();
    vendor.register(&app).await;
    let token = vendor.login(&app).await;

    // Create vendor profile
    let vendor_payload = json!({
        "business_name": "Test Laundry",
        "business_address": "123 Test St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Create service
    let service_payload = json!({
        "name": "Original Name",
        "base_price_cents": 3000,
        "price_per_kg_cents": 1000,
        "estimated_duration_hours": 12
    });

    let create_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let service: Value = create_resp.json().await.unwrap();
    let service_id = service["id"].as_str().unwrap();

    // Update service
    let update_payload = json!({
        "name": "Updated Name",
        "base_price_cents": 4000
    });

    let update_resp = app
        .client
        .patch(&format!("{}/api/v1/vendors/me/services/{}", app.address, service_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(update_resp.status(), StatusCode::OK);

    let updated_service: Value = update_resp.json().await.unwrap();
    assert!(updated_service["name"].as_str().unwrap().starts_with("Updated Name"));
    assert_eq!(updated_service["base_price_cents"].as_i64().unwrap(), 4000);
    assert_eq!(updated_service["price_per_kg_cents"].as_i64().unwrap(), 1000); // Unchanged
}

#[tokio::test]
async fn vendor_can_delete_their_service() {
    let app = spawn_app().await;

    let vendor = TestUser::generate_vendor();
    vendor.register(&app).await;
    let token = vendor.login(&app).await;

    // Create vendor profile
    let vendor_payload = json!({
        "business_name": "Test Laundry",
        "business_address": "123 Test St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Create service
    let service_payload = json!({
        "name": "Service to Delete",
        "base_price_cents": 3000,
        "price_per_kg_cents": 1000,
        "estimated_duration_hours": 12
    });

    let create_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let service: Value = create_resp.json().await.unwrap();
    let service_id = service["id"].as_str().unwrap();

    // Delete service
    let delete_resp = app
        .client
        .delete(&format!("{}/api/v1/vendors/me/services/{}", app.address, service_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);

    // Verify service is deactivated (still exists but is_active = false)
    let list_resp = app
        .client
        .get(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to execute request");

    let services: Value = list_resp.json().await.unwrap();
    let services_array = services.as_array().unwrap();
    let deleted_service = services_array
        .iter()
        .find(|s| s["id"].as_str().unwrap() == service_id)
        .unwrap();
    
    assert_eq!(deleted_service["is_active"].as_bool().unwrap(), false);
}

#[tokio::test]
async fn vendor_cannot_update_other_vendor_service() {
    let app = spawn_app().await;

    // Create first vendor
    let vendor1 = TestUser::generate_vendor();
    vendor1.register(&app).await;
    let token1 = vendor1.login(&app).await;

    let vendor1_payload = json!({
        "business_name": "Vendor 1",
        "business_address": "123 Test St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token1))
        .json(&vendor1_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Create service for vendor 1
    let service_payload = json!({
        "name": "Vendor 1 Service",
        "base_price_cents": 3000,
        "price_per_kg_cents": 1000,
        "estimated_duration_hours": 12
    });

    let create_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token1))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let service: Value = create_resp.json().await.unwrap();
    let service_id = service["id"].as_str().unwrap();

    // Create second vendor
    let vendor2 = TestUser::generate_vendor();
    vendor2.register(&app).await;
    let token2 = vendor2.login(&app).await;

    let vendor2_payload = json!({
        "business_name": "Vendor 2",
        "business_address": "456 Test Ave",
        "city": "Abuja"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", token2))
        .json(&vendor2_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Try to update vendor 1's service with vendor 2's token
    let update_payload = json!({
        "name": "Hacked Service"
    });

    let update_resp = app
        .client
        .patch(&format!("{}/api/v1/vendors/me/services/{}", app.address, service_id))
        .header("Authorization", format!("Bearer {}", token2))
        .json(&update_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(update_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn customer_cannot_create_service() {
    let app = spawn_app().await;

    // Register as customer (not vendor)
    let customer = TestUser::generate();
    customer.register(&app).await;
    let token = customer.login(&app).await;

    let service_payload = json!({
        "name": "Unauthorized Service",
        "base_price_cents": 3000,
        "price_per_kg_cents": 1000,
        "estimated_duration_hours": 12
    });

    let create_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Should fail because user is not a vendor
    assert_eq!(create_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn booking_price_calculated_automatically() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor = TestUser::generate_vendor();
    vendor.register(&app).await;
    let vendor_token = vendor.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Price Test Vendor",
        "business_address": "123 Test St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // Create service with specific pricing
    // base_price = 5000 cents ($50)
    // price_per_kg = 2000 cents ($20/kg)
    let service_payload = json!({
        "name": "Pricing Test Service",
        "base_price_cents": 5000,
        "price_per_kg_cents": 2000,
        "estimated_duration_hours": 24
    });

    let create_service_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let service: Value = create_service_resp.json().await.unwrap();
    let service_id = service["id"].as_str().unwrap();

    // Create customer and book the service
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let pickup_time = chrono::Utc::now() + chrono::Duration::hours(2);
    let pickup_iso = pickup_time.to_rfc3339();

    // Book with 3.5 kg of laundry
    // Expected price = 5000 + (2000 * 3.5) = 5000 + 7000 = 12000 cents ($120)
    let booking_payload = json!({
        "service_id": service_id,
        "pickup_address": "10 Customer St",
        "delivery_address": "10 Customer St",
        "scheduled_pickup_time": pickup_iso,
        "total_weight_kg": 3.5
    });

    let create_booking_resp = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&booking_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_booking_resp.status(), StatusCode::CREATED);

    let booking: Value = create_booking_resp.json().await.unwrap();
    
    // Verify automatic price calculation
    assert_eq!(booking["total_price_cents"].as_i64().unwrap(), 12000);
    assert_eq!(booking["total_weight_kg"].as_f64().unwrap(), 3.5);
}

#[tokio::test]
async fn booking_price_uses_base_price_when_no_weight() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor = TestUser::generate_vendor();
    vendor.register(&app).await;
    let vendor_token = vendor.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Base Price Vendor",
        "business_address": "123 Test St",
        "city": "Lagos"
    });

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let service_payload = json!({
        "name": "Base Price Service",
        "base_price_cents": 8000,
        "price_per_kg_cents": 2000,
        "estimated_duration_hours": 24
    });

    let create_service_resp = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&service_payload)
        .send()
        .await
        .expect("Failed to execute request");

    let service: Value = create_service_resp.json().await.unwrap();
    let service_id = service["id"].as_str().unwrap();

    // Create customer and book without specifying weight
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let pickup_time = chrono::Utc::now() + chrono::Duration::hours(2);
    let pickup_iso = pickup_time.to_rfc3339();

    let booking_payload = json!({
        "service_id": service_id,
        "pickup_address": "10 Customer St",
        "delivery_address": "10 Customer St",
        "scheduled_pickup_time": pickup_iso
        // No total_weight_kg provided
    });

    let create_booking_resp = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&booking_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_booking_resp.status(), StatusCode::CREATED);

    let booking: Value = create_booking_resp.json().await.unwrap();
    
    // Should use base price only when weight not provided
    assert_eq!(booking["total_price_cents"].as_i64().unwrap(), 8000);
    assert!(booking["total_weight_kg"].is_null());
}
