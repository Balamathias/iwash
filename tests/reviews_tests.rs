mod common;

use axum::http::StatusCode;
use common::{spawn_app, TestUser};
use serde_json::{json, Value};

#[tokio::test]
async fn test_customer_creates_review_after_delivered_booking() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    let vendor_response = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "business_name": "Test Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor");
    
    let vendor_data: Value = vendor_response.json().await.unwrap();
    let vendor_id = vendor_data["id"].as_str().unwrap();

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "name": "Express Wash",
            "description": "Fast wash service",
            "base_price_cents": 1000,
            "price_per_kg_cents": 500,
            "estimated_duration_hours": 12
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create customer and booking
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 5.0,
            "notes": "Handle with care"
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Update booking status to delivered
    app.client
        .patch(&format!("{}/api/v1/bookings/vendor/{}", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "status": "delivered"
        }))
        .send()
        .await
        .expect("Failed to update booking status");

    // Create review
    let review_response = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 5,
            "comment": "Excellent service! Very satisfied."
        }))
        .send()
        .await
        .expect("Failed to create review");

    assert_eq!(review_response.status(), StatusCode::CREATED);
    
    let review: Value = review_response.json().await.unwrap();
    assert_eq!(review["rating"], 5);
    assert_eq!(review["comment"], "Excellent service! Very satisfied.");
    assert_eq!(review["vendor_id"], vendor_id);
    assert_eq!(review["booking_id"], booking_id);

    // Verify vendor rating was updated
    let vendor_details = app
        .client
        .get(&format!("{}/api/v1/vendors/{}", app.address, vendor_id))
        .send()
        .await
        .expect("Failed to get vendor");
    
    let vendor_info: Value = vendor_details.json().await.unwrap();
    assert_eq!(vendor_info["rating"], "5.00");
    assert_eq!(vendor_info["total_reviews"], 1);
}

#[tokio::test]
async fn test_cannot_review_non_delivered_booking() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "business_name": "Test Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor");

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "name": "Regular Wash",
            "base_price_cents": 1000,
            "price_per_kg_cents": 300
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create customer and booking
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 3.0
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Try to create review for pending booking
    let review_response = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 5,
            "comment": "Great!"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(review_response.status(), StatusCode::BAD_REQUEST);
    
    let error: Value = review_response.json().await.unwrap();
    assert!(error["message"].as_str().unwrap().contains("delivered"));
}

#[tokio::test]
async fn test_cannot_review_same_booking_twice() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "business_name": "Test Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor");

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "name": "Deluxe Wash",
            "base_price_cents": 1500,
            "price_per_kg_cents": 600
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create customer and booking
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 4.0
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Update booking to delivered
    app.client
        .patch(&format!("{}/api/v1/bookings/vendor/{}", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({"status": "delivered"}))
        .send()
        .await
        .expect("Failed to update booking status");

    // Create first review
    let first_review = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 4,
            "comment": "Good service"
        }))
        .send()
        .await
        .expect("Failed to create first review");

    assert_eq!(first_review.status(), StatusCode::CREATED);

    // Try to create second review for same booking
    let second_review = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 5,
            "comment": "Changed my mind, it was excellent!"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(second_review.status(), StatusCode::BAD_REQUEST);
    
    let error: Value = second_review.json().await.unwrap();
    assert!(error["message"].as_str().unwrap().contains("already been reviewed"));
}

#[tokio::test]
async fn test_invalid_rating_rejected() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "business_name": "Test Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor");

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "name": "Standard Wash",
            "base_price_cents": 800,
            "price_per_kg_cents": 400
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create customer and booking
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 2.5
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Update booking to delivered
    app.client
        .patch(&format!("{}/api/v1/bookings/vendor/{}", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({"status": "delivered"}))
        .send()
        .await
        .expect("Failed to update booking status");

    // Try to create review with rating > 5
    let review_response = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 6,
            "comment": "Too high rating"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(review_response.status(), StatusCode::BAD_REQUEST);
    
    let error: Value = review_response.json().await.unwrap();
    assert!(error["message"].as_str().unwrap().contains("between 1 and 5"));
}

#[tokio::test]
async fn test_vendor_responds_to_review() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "business_name": "Test Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor");

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "name": "Premium Wash",
            "base_price_cents": 2000,
            "price_per_kg_cents": 800
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create customer and booking
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 6.0
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Update booking to delivered
    app.client
        .patch(&format!("{}/api/v1/bookings/vendor/{}", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({"status": "delivered"}))
        .send()
        .await
        .expect("Failed to update booking status");

    // Create review
    let review_response = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 4,
            "comment": "Good but could be faster"
        }))
        .send()
        .await
        .expect("Failed to create review");
    
    let review: Value = review_response.json().await.unwrap();
    let review_id = review["id"].as_str().unwrap();

    // Vendor responds to review
    let response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/reviews/{}/response", app.address, review_id))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "response": "Thank you for your feedback! We're working on improving our turnaround time."
        }))
        .send()
        .await
        .expect("Failed to respond to review");

    assert_eq!(response.status(), StatusCode::OK);
    
    let updated_review: Value = response.json().await.unwrap();
    assert_eq!(updated_review["vendor_response"], "Thank you for your feedback! We're working on improving our turnaround time.");
    assert!(updated_review["vendor_response_at"].as_str().is_some());
}

#[tokio::test]
async fn test_vendor_cannot_respond_to_other_vendors_review() {
    let app = spawn_app().await;

    // Create first vendor
    let vendor1 = TestUser::generate_vendor();
    vendor1.register(&app).await;
    let vendor1_token = vendor1.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor1_token))
        .json(&json!({
            "business_name": "Vendor 1 Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor 1");

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor1_token))
        .json(&json!({
            "name": "Service 1",
            "base_price_cents": 1000,
            "price_per_kg_cents": 500
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create customer and booking
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 3.0
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Update booking to delivered
    app.client
        .patch(&format!("{}/api/v1/bookings/vendor/{}", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", vendor1_token))
        .json(&json!({"status": "delivered"}))
        .send()
        .await
        .expect("Failed to update booking status");

    // Create review
    let review_response = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer_token))
        .json(&json!({
            "rating": 5,
            "comment": "Excellent!"
        }))
        .send()
        .await
        .expect("Failed to create review");
    
    let review: Value = review_response.json().await.unwrap();
    let review_id = review["id"].as_str().unwrap();

    // Create second vendor
    let vendor2 = TestUser::generate_vendor();
    vendor2.register(&app).await;
    let vendor2_token = vendor2.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor2_token))
        .json(&json!({
            "business_name": "Vendor 2 Laundry",
            "business_address": "456 Other St",
            "city": "Abuja"
        }))
        .send()
        .await
        .expect("Failed to create vendor 2");

    // Vendor 2 tries to respond to Vendor 1's review
    let response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/reviews/{}/response", app.address, review_id))
        .header("Authorization", format!("Bearer {}", vendor2_token))
        .json(&json!({
            "response": "Trying to respond to another vendor's review"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_customer_cannot_review_other_customers_booking() {
    let app = spawn_app().await;

    // Create vendor and service
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    app.client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "business_name": "Test Laundry",
            "business_address": "123 Main St",
            "city": "Lagos"
        }))
        .send()
        .await
        .expect("Failed to create vendor");

    let service_response = app
        .client
        .post(&format!("{}/api/v1/vendors/me/services", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({
            "name": "Quick Wash",
            "base_price_cents": 900,
            "price_per_kg_cents": 350
        }))
        .send()
        .await
        .expect("Failed to create service");
    
    let service_data: Value = service_response.json().await.unwrap();
    let service_id = service_data["id"].as_str().unwrap();

    // Create first customer and booking
    let customer1 = TestUser::generate();
    customer1.register(&app).await;
    let customer1_token = customer1.login(&app).await;

    let booking_response = app
        .client
        .post(&format!("{}/api/v1/bookings", app.address))
        .header("Authorization", format!("Bearer {}", customer1_token))
        .json(&json!({
            "service_id": service_id,
            "pickup_address": "123 Customer St",
            "delivery_address": "123 Customer St",
            "scheduled_pickup_time": "2025-10-28T10:00:00Z",
            "total_weight_kg": 4.0
        }))
        .send()
        .await
        .expect("Failed to create booking");
    
    let booking_data: Value = booking_response.json().await.unwrap();
    let booking_id = booking_data["id"].as_str().unwrap();

    // Update booking to delivered
    app.client
        .patch(&format!("{}/api/v1/bookings/vendor/{}", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&json!({"status": "delivered"}))
        .send()
        .await
        .expect("Failed to update booking status");

    // Create second customer
    let customer2 = TestUser::generate();
    customer2.register(&app).await;
    let customer2_token = customer2.login(&app).await;

    // Customer 2 tries to review Customer 1's booking
    let review_response = app
        .client
        .post(&format!("{}/api/v1/bookings/{}/review", app.address, booking_id))
        .header("Authorization", format!("Bearer {}", customer2_token))
        .json(&json!({
            "rating": 1,
            "comment": "Trying to review someone else's booking"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(review_response.status(), StatusCode::FORBIDDEN);
}
