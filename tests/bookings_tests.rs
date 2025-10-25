mod common;

use axum::http::StatusCode;
use common::{spawn_app, TestUser};
use serde_json::{json, Value};
use uuid::Uuid;

#[tokio::test]
async fn booking_response_includes_vendor_info() {
    let app = spawn_app().await;

    // 1. Create vendor user and profile
    let vendor_user = TestUser::generate_vendor();
    vendor_user.register(&app).await;
    let vendor_token = vendor_user.login(&app).await;

    let vendor_payload = json!({
        "business_name": "Booking Vendor",
        "business_description": "Vendor for booking test",
        "business_email": "vendor-booking@test.com",
        "business_phone": "+1111111111",
        "business_address": "1 Vendor St",
        "city": "Lagos",
        "state": "Lagos",
        "postal_code": "100001",
        "country": "Nigeria",
        "latitude": 6.5244,
        "longitude": 3.3792,
        "service_radius_km": 10
    });

    let create_vendor_resp = app
        .client
        .post(&format!("{}/api/v1/vendors", app.address))
        .header("Authorization", format!("Bearer {}", vendor_token))
        .json(&vendor_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(create_vendor_resp.status(), StatusCode::CREATED);
    let vendor: Value = create_vendor_resp.json().await.unwrap();
    let vendor_id = vendor["id"].as_str().unwrap().to_string();
    let vendor_name = vendor["business_name"].as_str().unwrap().to_string();

    // 2. Insert a service linked to this vendor directly into the test DB
    let service_id = Uuid::new_v4();
    let insert_service_sql = r#"
        INSERT INTO services (id, name, description, base_price_cents, price_per_kg_cents, estimated_duration_hours, is_active, vendor_id, is_featured)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

    sqlx::query(insert_service_sql)
        .bind(service_id)
        .bind("Wash & Fold")
        .bind(Some("Basic wash and fold"))
        .bind(5000_i32)
        .bind(2000_i32)
        .bind(24_i32)
        .bind(true)
        .bind(Some(Uuid::parse_str(&vendor_id).unwrap()))
        .bind(Some(false))
        .execute(&app.db_pool)
        .await
        .expect("Failed to insert service");

    // 3. Register a customer and create a booking for the service
    let customer = TestUser::generate();
    customer.register(&app).await;
    let customer_token = customer.login(&app).await;

    // scheduled pickup time: +2 hours
    let pickup_time = chrono::Utc::now() + chrono::Duration::hours(2);
    let pickup_iso = pickup_time.to_rfc3339();

    let booking_payload = json!({
        "service_id": service_id.to_string(),
        "pickup_address": "10 Customer St",
        "delivery_address": "10 Customer St",
        "scheduled_pickup_time": pickup_iso,
        "total_weight_kg": 3.5,
        "notes": "Please handle with care"
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

    // Assert vendor fields are present in booking response
    assert!(booking["vendor_id"].is_string());
    assert_eq!(booking["vendor_id"].as_str().unwrap(), vendor_id.as_str());
    assert!(booking["vendor_name"].is_string());
    assert_eq!(booking["vendor_name"].as_str().unwrap(), vendor_name.as_str());
}
