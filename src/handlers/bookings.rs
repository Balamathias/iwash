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
        Booking, BookingResponse, BookingStatus, BookingWithDetails,
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
    // Parse and validate service ID
    let service_id = Uuid::parse_str(&payload.service_id)
        .map_err(|_| AppError::BadRequest("Invalid service ID format".to_string()))?;

    // Validate service exists and get its pricing
    let service: Option<(i32, i32)> = sqlx::query_as(
        "SELECT base_price_cents, price_per_kg_cents FROM services WHERE id = $1 AND is_active = true"
    )
    .bind(service_id)
    .fetch_optional(&db)
    .await?;

    let (base_price, price_per_kg) = service
        .ok_or(AppError::BadRequest("Service not found or inactive".to_string()))?;

    // Parse scheduled pickup time
    let scheduled_pickup_time = OffsetDateTime::parse(&payload.scheduled_pickup_time, &time::format_description::well_known::Iso8601::DEFAULT)
        .map_err(|_| AppError::BadRequest("Invalid scheduled_pickup_time format. Use ISO 8601 format".to_string()))?;

    // Validate pickup time is in the future
    if scheduled_pickup_time < OffsetDateTime::now_utc() {
        return Err(AppError::BadRequest("Scheduled pickup time must be in the future".to_string()));
    }

    // Parse scheduled delivery time if provided
    let scheduled_delivery_time = if let Some(ref dt) = payload.scheduled_delivery_time {
        let delivery_time = OffsetDateTime::parse(dt, &time::format_description::well_known::Iso8601::DEFAULT)
            .map_err(|_| AppError::BadRequest("Invalid scheduled_delivery_time format. Use ISO 8601 format".to_string()))?;
        
        // Validate delivery time is after pickup time
        if delivery_time <= scheduled_pickup_time {
            return Err(AppError::BadRequest("Delivery time must be after pickup time".to_string()));
        }
        Some(delivery_time)
    } else {
        None
    };

    // Validate addresses
    if payload.pickup_address.trim().is_empty() || payload.delivery_address.trim().is_empty() {
        return Err(AppError::BadRequest("Pickup and delivery addresses cannot be empty".to_string()));
    }

    // Validate weight if provided
    if let Some(weight) = payload.total_weight_kg {
        if weight <= 0.0 {
            return Err(AppError::BadRequest("Total weight must be greater than 0".to_string()));
        }
    }

    // Calculate total price based on service pricing
    // Price = base_price + (price_per_kg * weight_kg)
    let total_price_cents = if let Some(weight_kg) = payload.total_weight_kg {
        base_price + ((price_per_kg as f64 * weight_kg) as i32)
    } else {
        // If no weight provided, use base price only
        base_price
    };

    tracing::info!(
        "Calculated booking price: base={} + (per_kg={} * weight={:?}) = {}",
        base_price,
        price_per_kg,
        payload.total_weight_kg,
        total_price_cents
    );

    // Start a transaction
    let mut tx = db.begin().await?;

    // Create the booking
    let booking_id = Uuid::new_v4();
    let weight_decimal = payload.total_weight_kg
        .map(|w| rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default());
    
    sqlx::query(
        "INSERT INTO bookings (id, user_id, service_id, status, pickup_address, delivery_address, 
                              scheduled_pickup_time, scheduled_delivery_time, total_weight_kg, 
                              total_price_cents, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(booking_id)
    .bind(auth_user.user_id)
    .bind(service_id)
    .bind(BookingStatus::Pending)
    .bind(payload.pickup_address.trim())
    .bind(payload.delivery_address.trim())
    .bind(scheduled_pickup_time)
    .bind(scheduled_delivery_time)
    .bind(weight_decimal)
    .bind(total_price_cents) // Use calculated price
    .bind(payload.notes.as_ref().map(|s| s.trim()))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Fetch the created booking with vendor information
    let booking = sqlx::query_as::<_, BookingWithDetails>(
        "SELECT b.*, 
                s.name as service_name, 
                s.vendor_id,
                v.business_name as vendor_name,
                v.business_phone as vendor_phone,
                v.city as vendor_city,
                v.rating as vendor_rating
         FROM bookings b
         LEFT JOIN services s ON b.service_id = s.id
         LEFT JOIN vendors v ON s.vendor_id = v.id
         WHERE b.id = $1"
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
            "SELECT b.*, s.name as service_name, s.vendor_id, v.business_name as vendor_name
             FROM bookings b
             LEFT JOIN services s ON b.service_id = s.id
             LEFT JOIN vendors v ON s.vendor_id = v.id
             WHERE b.user_id = $1 AND b.status = $2
             ORDER BY b.created_at DESC
             LIMIT $3 OFFSET $4"
        )
    } else {
        (
            "SELECT COUNT(*) FROM bookings WHERE user_id = $1",
            "SELECT b.*, s.name as service_name, s.vendor_id, v.business_name as vendor_name
             FROM bookings b
             LEFT JOIN services s ON b.service_id = s.id
             LEFT JOIN vendors v ON s.vendor_id = v.id
             WHERE b.user_id = $1
             ORDER BY b.created_at DESC
             LIMIT $2 OFFSET $3"
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

    let bookings: Vec<BookingWithDetails> = if let Some(status) = query.status {
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

    let booking = sqlx::query_as::<_, BookingWithDetails>(
        "SELECT b.*, s.name as service_name, s.vendor_id, v.business_name as vendor_name
         FROM bookings b
         LEFT JOIN services s ON b.service_id = s.id
         LEFT JOIN vendors v ON s.vendor_id = v.id
         WHERE b.id = $1 AND b.user_id = $2"
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

    // Fetch updated booking with service/vendor details so response contains vendor info
    let booking = sqlx::query_as::<_, BookingWithDetails>(
        "SELECT b.*, s.name as service_name, s.vendor_id, \
                v.business_name as vendor_name, v.business_phone as vendor_phone, \
                v.city as vendor_city, v.rating as vendor_rating\
         FROM bookings b\
         LEFT JOIN services s ON b.service_id = s.id\
         LEFT JOIN vendors v ON s.vendor_id = v.id\
         WHERE b.id = $1"
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

/// List bookings for vendor's services (Vendor role required)
pub async fn list_vendor_bookings(
    State(db): State<Db>,
    auth_user: AuthUser,
    Query(query): Query<ListBookingsQuery>,
) -> AppResult<Json<PaginatedBookingsResponse>> {
    use crate::middleware::RequireVendor;
    
    // Check vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // Get vendor ID for this user
    let vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM vendors WHERE user_id = $1 AND is_active = true"
    )
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?;

    let vendor_id = vendor_id.ok_or(AppError::BadRequest(
        "Vendor profile not found. Please create a vendor profile first.".to_string()
    ))?;

    // Build query based on filters
    let (count_query, select_query) = if let Some(_status) = query.status {
        (
            "SELECT COUNT(*) FROM bookings b
             INNER JOIN services s ON b.service_id = s.id
             WHERE s.vendor_id = $1 AND b.status = $2",
            "SELECT b.*, s.name as service_name, s.vendor_id, v.business_name as vendor_name
             FROM bookings b
             INNER JOIN services s ON b.service_id = s.id
             LEFT JOIN vendors v ON s.vendor_id = v.id
             WHERE s.vendor_id = $1 AND b.status = $2
             ORDER BY b.created_at DESC
             LIMIT $3 OFFSET $4"
        )
    } else {
        (
            "SELECT COUNT(*) FROM bookings b
             INNER JOIN services s ON b.service_id = s.id
             WHERE s.vendor_id = $1",
            "SELECT b.*, s.name as service_name, s.vendor_id, v.business_name as vendor_name
             FROM bookings b
             INNER JOIN services s ON b.service_id = s.id
             LEFT JOIN vendors v ON s.vendor_id = v.id
             WHERE s.vendor_id = $1
             ORDER BY b.created_at DESC
             LIMIT $2 OFFSET $3"
        )
    };

    let total: i64 = if let Some(status) = query.status {
        sqlx::query_scalar(count_query)
            .bind(vendor_id)
            .bind(status)
            .fetch_one(&db)
            .await?
    } else {
        sqlx::query_scalar(count_query)
            .bind(vendor_id)
            .fetch_one(&db)
            .await?
    };

    let bookings: Vec<BookingWithDetails> = if let Some(status) = query.status {
        sqlx::query_as(select_query)
            .bind(vendor_id)
            .bind(status)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
    } else {
        sqlx::query_as(select_query)
            .bind(vendor_id)
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

/// Update booking status (Vendor role required)
pub async fn update_booking_status_vendor(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateBookingRequest>,
) -> AppResult<Json<BookingResponse>> {
    use crate::middleware::RequireVendor;
    
    // Check vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

    let booking_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid booking ID format".to_string()))?;

    // Get vendor ID
    let vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM vendors WHERE user_id = $1 AND is_active = true"
    )
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?;

    let vendor_id = vendor_id.ok_or(AppError::BadRequest(
        "Vendor profile not found".to_string()
    ))?;

    // Verify booking belongs to vendor's service
    let booking_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM bookings b
            INNER JOIN services s ON b.service_id = s.id
            WHERE b.id = $1 AND s.vendor_id = $2
        )"
    )
    .bind(booking_id)
    .bind(vendor_id)
    .fetch_one(&db)
    .await?;

    if !booking_exists {
        return Err(AppError::NotFound);
    }

    // Vendors can only update specific fields (status, actual times, weight, price)
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

    // Update booking (vendors can update status, actual times, weight, and price)
    sqlx::query(
        "UPDATE bookings
         SET status = COALESCE($1, status),
             actual_pickup_time = COALESCE($2, actual_pickup_time),
             actual_delivery_time = COALESCE($3, actual_delivery_time),
             total_weight_kg = COALESCE($4, total_weight_kg),
             total_price_cents = COALESCE($5, total_price_cents),
             updated_at = NOW()
         WHERE id = $6"
    )
    .bind(payload.status)
    .bind(actual_pickup_time)
    .bind(actual_delivery_time)
    .bind(total_weight_kg)
    .bind(payload.total_price_cents)
    .bind(booking_id)
    .execute(&db)
    .await?;

    // Fetch updated booking with details
    let booking = sqlx::query_as::<_, BookingWithDetails>(
        "SELECT b.*, s.name as service_name, s.vendor_id, v.business_name as vendor_name
         FROM bookings b
         LEFT JOIN services s ON b.service_id = s.id
         LEFT JOIN vendors v ON s.vendor_id = v.id
         WHERE b.id = $1"
    )
    .bind(booking_id)
    .fetch_one(&db)
    .await?;

    Ok(Json(booking.into()))
}
