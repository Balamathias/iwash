use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    db::Db,
    errors::{AppError, AppResult},
    middleware::AuthUser,
    models::{
        Booking, BookingResponse, BookingStatus,
        CreateBookingRequest, ListBookingsQuery, PaginatedBookingsResponse,
        UpdateBookingRequest,
    },
};

/// Create a new booking
pub async fn create_booking(
    State(db): State<Db>,
    auth_user: AuthUser,
    Json(payload): Json<CreateBookingRequest>,
) -> AppResult<(StatusCode, Json<BookingResponse>)> {
    // Validate service exists
    let service_id = Uuid::parse_str(&payload.service_id)
        .map_err(|_| AppError::BadRequest("Invalid service ID format".to_string()))?;

    let service_exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM services WHERE id = $1 AND is_active = true)")
        .bind(service_id)
        .fetch_one(&db)
        .await?;

    if !service_exists {
        return Err(AppError::BadRequest("Service not found or inactive".to_string()));
    }

    // Parse scheduled pickup time
    let scheduled_pickup_time = OffsetDateTime::parse(&payload.scheduled_pickup_time, &time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(|_| AppError::BadRequest("Invalid scheduled_pickup_time format. Use ISO 8601 format".to_string()))?;

    // Validate pickup time is in the future
    if scheduled_pickup_time < OffsetDateTime::now_utc() {
        return Err(AppError::BadRequest("Scheduled pickup time must be in the future".to_string()));
    }

    // Validate addresses
    if payload.pickup_address.trim().is_empty() || payload.delivery_address.trim().is_empty() {
        return Err(AppError::BadRequest("Pickup and delivery addresses cannot be empty".to_string()));
    }

    // Validate items
    if payload.items.is_empty() {
        return Err(AppError::BadRequest("At least one item must be provided".to_string()));
    }

    // Start a transaction
    let mut tx = db.begin().await?;

    // Create the booking
    let booking_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO bookings (id, user_id, service_id, status, pickup_address, delivery_address, 
                              scheduled_pickup_time, total_price_cents, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(booking_id)
    .bind(auth_user.user_id)
    .bind(service_id)
    .bind(BookingStatus::Pending)
    .bind(payload.pickup_address.trim())
    .bind(payload.delivery_address.trim())
    .bind(scheduled_pickup_time)
    .bind(0) // Initial price, will be calculated later
    .bind(payload.notes.as_ref().map(|s| s.trim()))
    .execute(&mut *tx)
    .await?;

    // Insert booking items
    for item in &payload.items {
        if item.quantity < 1 {
            return Err(AppError::BadRequest("Item quantity must be at least 1".to_string()));
        }

        let weight = item.weight_kg.map(|w| rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default());

        sqlx::query(
            "INSERT INTO booking_items (id, booking_id, item_type, quantity, weight_kg, notes)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(Uuid::new_v4())
        .bind(booking_id)
        .bind(item.item_type.trim())
        .bind(item.quantity)
        .bind(weight)
        .bind(item.notes.as_ref().map(|s| s.trim()))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Fetch the created booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(booking.into())))
}

/// List bookings for the authenticated user
pub async fn list_bookings(
    State(db): State<Db>,
    auth_user: AuthUser,
    Query(query): Query<ListBookingsQuery>,
) -> AppResult<Json<PaginatedBookingsResponse>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // Build query based on filters
    let (count_query, select_query) = if let Some(_status) = query.status {
        (
            "SELECT COUNT(*) FROM bookings WHERE user_id = $1 AND status = $2",
            "SELECT * FROM bookings WHERE user_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
        )
    } else {
        (
            "SELECT COUNT(*) FROM bookings WHERE user_id = $1",
            "SELECT * FROM bookings WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
    };

    let total: i64 = if let Some(status) = query.status {
        sqlx::query_scalar(count_query)
            .bind(auth_user.user_id)
            .bind(status)
            .fetch_one(&db)
            .await?
    } else {
        sqlx::query_scalar(count_query)
            .bind(auth_user.user_id)
            .fetch_one(&db)
            .await?
    };

    let bookings: Vec<Booking> = if let Some(status) = query.status {
        sqlx::query_as(select_query)
            .bind(auth_user.user_id)
            .bind(status)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
    } else {
        sqlx::query_as(select_query)
            .bind(auth_user.user_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
    let booking_responses: Vec<BookingResponse> = bookings.into_iter().map(|b| b.into()).collect();

    Ok(Json(PaginatedBookingsResponse {
        bookings: booking_responses,
        page,
        limit,
        total,
        total_pages,
    }))
}

/// Get a specific booking by ID
pub async fn get_booking(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<BookingResponse>> {
    let booking_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid booking ID format".to_string()))?;

    let booking = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1 AND user_id = $2"
    )
    .bind(booking_id)
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(booking.into()))
}

/// Update a booking
pub async fn update_booking(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateBookingRequest>,
) -> AppResult<Json<BookingResponse>> {
    let booking_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid booking ID format".to_string()))?;

    // Verify booking exists and belongs to user
    let existing = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1 AND user_id = $2"
    )
    .bind(booking_id)
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?
    .ok_or(AppError::NotFound)?;

    // Prevent updates to cancelled or delivered bookings
    if matches!(existing.status, BookingStatus::Cancelled | BookingStatus::Delivered) {
        return Err(AppError::BadRequest("Cannot update cancelled or delivered bookings".to_string()));
    }

    // Parse datetime fields if provided
    let scheduled_pickup_time = if let Some(ref time_str) = payload.scheduled_pickup_time {
        Some(OffsetDateTime::parse(time_str, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| AppError::BadRequest("Invalid scheduled_pickup_time format".to_string()))?)
    } else {
        None
    };

    let scheduled_delivery_time = if let Some(ref time_str) = payload.scheduled_delivery_time {
        Some(OffsetDateTime::parse(time_str, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| AppError::BadRequest("Invalid scheduled_delivery_time format".to_string()))?)
    } else {
        None
    };

    let actual_pickup_time = if let Some(ref time_str) = payload.actual_pickup_time {
        Some(OffsetDateTime::parse(time_str, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| AppError::BadRequest("Invalid actual_pickup_time format".to_string()))?)
    } else {
        None
    };

    let actual_delivery_time = if let Some(ref time_str) = payload.actual_delivery_time {
        Some(OffsetDateTime::parse(time_str, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| AppError::BadRequest("Invalid actual_delivery_time format".to_string()))?)
    } else {
        None
    };

    let total_weight_kg = payload.total_weight_kg.map(|w| rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default());

    // Update booking
    sqlx::query(
        "UPDATE bookings
         SET status = COALESCE($1, status),
             pickup_address = COALESCE($2, pickup_address),
             delivery_address = COALESCE($3, delivery_address),
             scheduled_pickup_time = COALESCE($4, scheduled_pickup_time),
             scheduled_delivery_time = COALESCE($5, scheduled_delivery_time),
             actual_pickup_time = COALESCE($6, actual_pickup_time),
             actual_delivery_time = COALESCE($7, actual_delivery_time),
             total_weight_kg = COALESCE($8, total_weight_kg),
             total_price_cents = COALESCE($9, total_price_cents),
             notes = COALESCE($10, notes),
             updated_at = NOW()
         WHERE id = $11"
    )
    .bind(payload.status)
    .bind(payload.pickup_address.as_ref().map(|s| s.trim()))
    .bind(payload.delivery_address.as_ref().map(|s| s.trim()))
    .bind(scheduled_pickup_time)
    .bind(scheduled_delivery_time)
    .bind(actual_pickup_time)
    .bind(actual_delivery_time)
    .bind(total_weight_kg)
    .bind(payload.total_price_cents)
    .bind(payload.notes.as_ref().map(|s| s.trim()))
    .bind(booking_id)
    .execute(&db)
    .await?;

    // Fetch updated booking
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_one(&db)
    .await?;

    Ok(Json(booking.into()))
}

/// Cancel a booking
pub async fn cancel_booking(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let booking_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid booking ID format".to_string()))?;

    // Verify booking exists and belongs to user
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = $1 AND user_id = $2"
    )
    .bind(booking_id)
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?
    .ok_or(AppError::NotFound)?;

    // Prevent cancelling already cancelled or delivered bookings
    if matches!(booking.status, BookingStatus::Cancelled | BookingStatus::Delivered) {
        return Err(AppError::BadRequest("Booking is already cancelled or delivered".to_string()));
    }

    // Update status to cancelled
    sqlx::query(
        "UPDATE bookings SET status = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(BookingStatus::Cancelled)
    .bind(booking_id)
    .execute(&db)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
