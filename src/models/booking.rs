use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "booking_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BookingStatus {
    Pending,
    Confirmed,
    #[sqlx(rename = "picked_up")]
    #[serde(rename = "picked_up")]
    PickedUp,
    #[sqlx(rename = "in_progress")]
    #[serde(rename = "in_progress")]
    InProgress,
    Ready,
    Delivered,
    Cancelled,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Booking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service_id: Uuid,
    pub status: BookingStatus,
    pub pickup_address: String,
    pub delivery_address: String,
    pub scheduled_pickup_time: OffsetDateTime,
    pub scheduled_delivery_time: Option<OffsetDateTime>,
    pub actual_pickup_time: Option<OffsetDateTime>,
    pub actual_delivery_time: Option<OffsetDateTime>,
    pub total_weight_kg: Option<rust_decimal::Decimal>,
    pub total_price_cents: i32,
    pub notes: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Extended booking with service and vendor information
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BookingWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service_id: Uuid,
    pub service_name: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub vendor_name: Option<String>,
    pub vendor_phone: Option<String>,
    pub vendor_city: Option<String>,
    pub vendor_rating: Option<rust_decimal::Decimal>,
    pub status: BookingStatus,
    pub pickup_address: String,
    pub delivery_address: String,
    pub scheduled_pickup_time: OffsetDateTime,
    pub scheduled_delivery_time: Option<OffsetDateTime>,
    pub actual_pickup_time: Option<OffsetDateTime>,
    pub actual_delivery_time: Option<OffsetDateTime>,
    pub total_weight_kg: Option<rust_decimal::Decimal>,
    pub total_price_cents: i32,
    pub notes: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BookingItem {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub item_type: String,
    pub quantity: i32,
    pub weight_kg: Option<rust_decimal::Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub service_id: String,
    pub pickup_address: String,
    pub delivery_address: String,
    pub scheduled_pickup_time: String, // ISO 8601 datetime string
    pub scheduled_delivery_time: Option<String>, // ISO 8601 datetime string
    pub total_weight_kg: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingItemRequest {
    pub item_type: String,
    pub quantity: i32,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookingRequest {
    pub status: Option<BookingStatus>,
    pub pickup_address: Option<String>,
    pub delivery_address: Option<String>,
    pub scheduled_pickup_time: Option<String>,
    pub scheduled_delivery_time: Option<String>,
    pub actual_pickup_time: Option<String>,
    pub actual_delivery_time: Option<String>,
    pub total_weight_kg: Option<f64>,
    pub total_price_cents: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub id: String,
    pub user_id: String,
    pub service_id: String,
    pub service_name: Option<String>,
    pub vendor_id: Option<String>,
    pub vendor_name: Option<String>,
    pub vendor_phone: Option<String>,
    pub vendor_city: Option<String>,
    pub vendor_rating: Option<f64>,
    pub status: BookingStatus,
    pub pickup_address: String,
    pub delivery_address: String,
    pub scheduled_pickup_time: String,
    pub scheduled_delivery_time: Option<String>,
    pub actual_pickup_time: Option<String>,
    pub actual_delivery_time: Option<String>,
    pub total_weight_kg: Option<f64>,
    pub total_price_cents: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct BookingItemResponse {
    pub id: String,
    pub booking_id: String,
    pub item_type: String,
    pub quantity: i32,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListBookingsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub status: Option<BookingStatus>,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct PaginatedBookingsResponse {
    pub bookings: Vec<BookingResponse>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
    pub total_pages: u32,
}

impl From<Booking> for BookingResponse {
    fn from(booking: Booking) -> Self {
        BookingResponse {
            id: booking.id.to_string(),
            user_id: booking.user_id.to_string(),
            service_id: booking.service_id.to_string(),
            service_name: None,
            vendor_id: None,
            vendor_name: None,
            vendor_phone: None,
            vendor_city: None,
            vendor_rating: None,
            status: booking.status,
            pickup_address: booking.pickup_address,
            delivery_address: booking.delivery_address,
            scheduled_pickup_time: booking.scheduled_pickup_time.to_string(),
            scheduled_delivery_time: booking.scheduled_delivery_time.map(|t| t.to_string()),
            actual_pickup_time: booking.actual_pickup_time.map(|t| t.to_string()),
            actual_delivery_time: booking.actual_delivery_time.map(|t| t.to_string()),
            total_weight_kg: booking.total_weight_kg.map(|d| d.to_string().parse().unwrap_or(0.0)),
            total_price_cents: booking.total_price_cents,
            notes: booking.notes,
            created_at: booking.created_at.to_string(),
            updated_at: booking.updated_at.to_string(),
        }
    }
}

impl From<BookingWithDetails> for BookingResponse {
    fn from(booking: BookingWithDetails) -> Self {
        BookingResponse {
            id: booking.id.to_string(),
            user_id: booking.user_id.to_string(),
            service_id: booking.service_id.to_string(),
            service_name: booking.service_name,
            vendor_id: booking.vendor_id.map(|id| id.to_string()),
            vendor_name: booking.vendor_name,
            vendor_phone: booking.vendor_phone,
            vendor_city: booking.vendor_city,
            vendor_rating: booking.vendor_rating.map(|d| d.to_string().parse().unwrap_or(0.0)),
            status: booking.status,
            pickup_address: booking.pickup_address,
            delivery_address: booking.delivery_address,
            scheduled_pickup_time: booking.scheduled_pickup_time.to_string(),
            scheduled_delivery_time: booking.scheduled_delivery_time.map(|t| t.to_string()),
            actual_pickup_time: booking.actual_pickup_time.map(|t| t.to_string()),
            actual_delivery_time: booking.actual_delivery_time.map(|t| t.to_string()),
            total_weight_kg: booking.total_weight_kg.map(|d| d.to_string().parse().unwrap_or(0.0)),
            total_price_cents: booking.total_price_cents,
            notes: booking.notes,
            created_at: booking.created_at.to_string(),
            updated_at: booking.updated_at.to_string(),
        }
    }
}

impl From<BookingItem> for BookingItemResponse {
    fn from(item: BookingItem) -> Self {
        BookingItemResponse {
            id: item.id.to_string(),
            booking_id: item.booking_id.to_string(),
            item_type: item.item_type,
            quantity: item.quantity,
            weight_kg: item.weight_kg.map(|d| d.to_string().parse().unwrap_or(0.0)),
            notes: item.notes,
        }
    }
}
