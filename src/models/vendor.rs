use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Vendor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_name: String,
    pub business_description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub business_email: Option<String>,
    pub business_phone: Option<String>,
    pub business_address: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<rust_decimal::Decimal>,
    pub longitude: Option<rust_decimal::Decimal>,
    pub operating_hours: Option<serde_json::Value>,
    pub service_radius_km: Option<i32>,
    pub rating: Option<rust_decimal::Decimal>,
    pub total_reviews: Option<i32>,
    pub total_bookings: Option<i32>,
    pub is_verified: bool,
    pub is_active: bool,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_name: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateVendorRequest {
    pub business_name: String,
    pub business_description: Option<String>,
    pub business_email: Option<String>,
    pub business_phone: Option<String>,
    pub business_address: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub service_radius_km: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVendorRequest {
    pub business_name: Option<String>,
    pub business_description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub business_email: Option<String>,
    pub business_phone: Option<String>,
    pub business_address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub operating_hours: Option<serde_json::Value>,
    pub service_radius_km: Option<i32>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VendorResponse {
    pub id: String,
    pub user_id: String,
    pub business_name: String,
    pub business_description: Option<String>,
    pub logo_url: Option<String>,
    pub banner_url: Option<String>,
    pub business_email: Option<String>,
    pub business_phone: Option<String>,
    pub business_address: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub operating_hours: Option<serde_json::Value>,
    pub service_radius_km: Option<i32>,
    pub rating: Option<f64>,
    pub total_reviews: Option<i32>,
    pub total_bookings: Option<i32>,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListVendorsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub city: Option<String>,
    pub search: Option<String>,
    pub is_verified: Option<bool>,
    pub min_rating: Option<f64>,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct PaginatedVendorsResponse {
    pub vendors: Vec<VendorResponse>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
    pub total_pages: u32,
}

impl From<Vendor> for VendorResponse {
    fn from(vendor: Vendor) -> Self {
        VendorResponse {
            id: vendor.id.to_string(),
            user_id: vendor.user_id.to_string(),
            business_name: vendor.business_name,
            business_description: vendor.business_description,
            logo_url: vendor.logo_url,
            banner_url: vendor.banner_url,
            business_email: vendor.business_email,
            business_phone: vendor.business_phone,
            business_address: vendor.business_address,
            city: vendor.city,
            state: vendor.state,
            postal_code: vendor.postal_code,
            country: vendor.country,
            latitude: vendor.latitude.map(|d| d.to_string().parse().unwrap_or(0.0)),
            longitude: vendor.longitude.map(|d| d.to_string().parse().unwrap_or(0.0)),
            operating_hours: vendor.operating_hours,
            service_radius_km: vendor.service_radius_km,
            rating: vendor.rating.map(|d| d.to_string().parse().unwrap_or(0.0)),
            total_reviews: vendor.total_reviews,
            total_bookings: vendor.total_bookings,
            is_verified: vendor.is_verified,
            is_active: vendor.is_active,
            created_at: vendor.created_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Review {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub user_id: Uuid,
    pub booking_id: Option<Uuid>,
    pub rating: i32,
    pub comment: Option<String>,
    pub vendor_response: Option<String>,
    pub vendor_response_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub vendor_id: String,
    pub booking_id: Option<String>,
    pub rating: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: String,
    pub vendor_id: String,
    pub user_id: String,
    pub booking_id: Option<String>,
    pub rating: i32,
    pub comment: Option<String>,
    pub vendor_response: Option<String>,
    pub vendor_response_at: Option<String>,
    pub created_at: String,
}

impl From<Review> for ReviewResponse {
    fn from(review: Review) -> Self {
        ReviewResponse {
            id: review.id.to_string(),
            vendor_id: review.vendor_id.to_string(),
            user_id: review.user_id.to_string(),
            booking_id: review.booking_id.map(|id| id.to_string()),
            rating: review.rating,
            comment: review.comment,
            vendor_response: review.vendor_response,
            vendor_response_at: review.vendor_response_at.map(|t| t.to_string()),
            created_at: review.created_at.to_string(),
        }
    }
}
