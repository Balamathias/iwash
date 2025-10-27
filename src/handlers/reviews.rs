use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::auth::AuthUser,
    models::vendor::{CreateReviewRequest, Review, ReviewResponse, VendorResponseRequest},
};

/// Create a review for a booking (Customer only)
/// POST /api/v1/bookings/:booking_id/review
pub async fn create_review(
    State(pool): State<PgPool>,
    Path(booking_id): Path<String>,
    auth_user: AuthUser,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Parse booking_id
    let booking_id = Uuid::parse_str(&booking_id)
        .map_err(|_| AppError::BadRequest("Invalid booking ID".to_string()))?;

    // Validate rating range
    if payload.rating < 1 || payload.rating > 5 {
        return Err(AppError::BadRequest(
            "Rating must be between 1 and 5".to_string(),
        ));
    }

    // Fetch booking details and verify ownership
    let booking = sqlx::query!(
        r#"
        SELECT 
            id, 
            user_id, 
            service_id,
            status::text as status
        FROM bookings
        WHERE id = $1
        "#,
        booking_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching booking: {:?}", e);
        AppError::Internal
    })?
    .ok_or(AppError::NotFound)?;

    // Verify booking belongs to the customer
    if booking.user_id != auth_user.user_id {
        return Err(AppError::Forbidden(Some(
            "You can only review your own bookings".to_string(),
        )));
    }

    // Verify booking status is 'delivered' (only completed bookings can be reviewed)
    let status = booking.status.as_deref().unwrap_or("");
    if status != "delivered" {
        return Err(AppError::BadRequest(
            "Only delivered bookings can be reviewed".to_string(),
        ));
    }

    // Get vendor_id from the service
    let service = sqlx::query!(
        r#"SELECT vendor_id FROM services WHERE id = $1"#,
        booking.service_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching service: {:?}", e);
        AppError::Internal
    })?;

    let vendor_id = service.vendor_id.ok_or_else(|| {
        AppError::Internal // Service should always have a vendor
    })?;

    // Check if review already exists for this booking
    let existing_review = sqlx::query!(
        r#"SELECT id FROM reviews WHERE booking_id = $1"#,
        booking_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking existing review: {:?}", e);
        AppError::Internal
    })?;

    if existing_review.is_some() {
        return Err(AppError::BadRequest(
            "This booking has already been reviewed".to_string(),
        ));
    }

    // Create review
    let review_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    let review = sqlx::query_as!(
        Review,
        r#"
        INSERT INTO reviews (
            id, vendor_id, user_id, booking_id, rating, comment, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
            id,
            vendor_id,
            user_id,
            booking_id,
            rating,
            comment,
            vendor_response,
            vendor_response_at,
            created_at,
            updated_at
        "#,
        review_id,
        vendor_id,
        auth_user.user_id,
        booking_id,
        payload.rating,
        payload.comment,
        now,
        now
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error creating review: {:?}", e);
        AppError::Internal
    })?;

    // Update vendor rating
    update_vendor_rating(&pool, vendor_id).await?;

    tracing::info!(
        "Customer {} created review {} for vendor {}",
        auth_user.user_id,
        review_id,
        vendor_id
    );

    Ok((StatusCode::CREATED, Json(ReviewResponse::from(review))))
}

/// Vendor responds to a review
/// POST /api/v1/vendors/me/reviews/:review_id/response
pub async fn vendor_respond_to_review(
    State(pool): State<PgPool>,
    Path(review_id): Path<String>,
    auth_user: AuthUser,
    Json(payload): Json<VendorResponseRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Parse review_id
    let review_id = Uuid::parse_str(&review_id)
        .map_err(|_| AppError::BadRequest("Invalid review ID".to_string()))?;

    // Validate response is not empty
    if payload.response.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Response cannot be empty".to_string(),
        ));
    }

    // Get vendor profile for this user
    let vendor = sqlx::query!(
        r#"SELECT id FROM vendors WHERE user_id = $1"#,
        auth_user.user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching vendor: {:?}", e);
        AppError::Internal
    })?
    .ok_or_else(|| AppError::Forbidden(Some("Vendor profile not found".to_string())))?;

    // Fetch review and verify it belongs to this vendor
    let review = sqlx::query!(
        r#"SELECT vendor_id FROM reviews WHERE id = $1"#,
        review_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching review: {:?}", e);
        AppError::Internal
    })?
    .ok_or(AppError::NotFound)?;

    if review.vendor_id != vendor.id {
        return Err(AppError::Forbidden(Some(
            "You can only respond to reviews for your business".to_string(),
        )));
    }

    // Update review with vendor response
    let now = OffsetDateTime::now_utc();
    let updated_review = sqlx::query_as!(
        Review,
        r#"
        UPDATE reviews
        SET 
            vendor_response = $1,
            vendor_response_at = $2,
            updated_at = $3
        WHERE id = $4
        RETURNING 
            id,
            vendor_id,
            user_id,
            booking_id,
            rating,
            comment,
            vendor_response,
            vendor_response_at,
            created_at,
            updated_at
        "#,
        payload.response,
        now,
        now,
        review_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating review response: {:?}", e);
        AppError::Internal
    })?;

    tracing::info!(
        "Vendor {} responded to review {}",
        auth_user.user_id,
        review_id
    );

    Ok((StatusCode::OK, Json(ReviewResponse::from(updated_review))))
}

/// Helper function to update vendor's average rating and total reviews
async fn update_vendor_rating(pool: &PgPool, vendor_id: Uuid) -> Result<(), AppError> {
    // Calculate average rating and total reviews
    let stats = sqlx::query!(
        r#"
        SELECT 
            COALESCE(AVG(rating), 0)::DECIMAL(3,2) as "avg_rating!",
            COUNT(*)::INT as "total_reviews!"
        FROM reviews
        WHERE vendor_id = $1
        "#,
        vendor_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error calculating vendor rating: {:?}", e);
        AppError::Internal
    })?;

    // Update vendor record
    sqlx::query!(
        r#"
        UPDATE vendors
        SET 
            rating = $1,
            total_reviews = $2,
            updated_at = NOW()
        WHERE id = $3
        "#,
        stats.avg_rating,
        stats.total_reviews,
        vendor_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating vendor rating: {:?}", e);
        AppError::Internal
    })?;

    tracing::info!(
        "Updated vendor {} rating to {} ({} reviews)",
        vendor_id,
        stats.avg_rating,
        stats.total_reviews
    );

    Ok(())
}
