use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::{
    db::Db,
    errors::{AppError, AppResult},
    middleware::{AuthUser, RequireVendor},
    models::{
        CreateVendorRequest, ListVendorsQuery, PaginatedVendorsResponse,
        UpdateVendorRequest, Vendor, VendorResponse,
    },
};
use rust_decimal::Decimal;
use serde_json::json;

/// Create vendor profile (Vendor role required)
pub async fn create_vendor(
    State(db): State<crate::db::Db>,
    auth_user: AuthUser,
    Json(payload): Json<CreateVendorRequest>,
) -> AppResult<(axum::http::StatusCode, Json<VendorResponse>)> {
    // Check Vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

    // Validate coordinates if provided
    if let (Some(lat), Some(lng)) = (payload.latitude, payload.longitude) {
        if lat < -90.0 || lat > 90.0 || lng < -180.0 || lng > 180.0 {
            return Err(AppError::BadRequest("Invalid coordinates".to_string()));
        }
    }

    // Check if vendor profile already exists for this user
    let existing: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM vendors WHERE user_id = $1")
        .bind(auth_user.user_id)
        .fetch_optional(&db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Vendor profile already exists for this user".to_string(),
        ));
    }

    // Convert lat/long to Decimal if provided
    let latitude = payload
        .latitude
        .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());
    let longitude = payload
        .longitude
        .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());

    // Create vendor profile
    let vendor_id = Uuid::new_v4();

    let vendor: Vendor = sqlx::query_as(
        "INSERT INTO vendors (
            id, user_id, business_name, business_description, business_email, business_phone,
            business_address, city, state, postal_code, country, latitude, longitude, service_radius_km
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         RETURNING *"
    )
    .bind(vendor_id)
    .bind(auth_user.user_id)
    .bind(payload.business_name.trim())
    .bind(payload.business_description.as_ref().map(|s| s.trim()))
    .bind(payload.business_email.as_ref().map(|s| s.trim()))
    .bind(payload.business_phone.as_ref().map(|s| s.trim()))
    .bind(payload.business_address.trim())
    .bind(payload.city.as_ref().map(|s| s.trim()))
    .bind(payload.state.as_ref().map(|s| s.trim()))
    .bind(payload.postal_code.as_ref().map(|s| s.trim()))
    .bind(payload.country.as_ref().map(|s| s.trim()))
    .bind(latitude)
    .bind(longitude)
    .bind(payload.service_radius_km)
    .fetch_one(&db)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(vendor.into())))
}

/// List all vendors (PUBLIC - no auth required)
/// Supports filtering by city, search term, verification status, and minimum rating
pub async fn list_vendors(
    State(db): State<Db>,
    Query(query): Query<ListVendorsQuery>,
) -> AppResult<Json<PaginatedVendorsResponse>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // Build query based on filters - simpler approach with direct queries
    let (vendors, total) = if query.city.is_some() || query.search.is_some() || query.is_verified.is_some() || query.min_rating.is_some() {
        // Filtered query
        let city_lower = query.city.as_ref().map(|c| c.to_lowercase());
        let search_pattern = query.search.as_ref().map(|s| format!("%{}%", s.to_lowercase()));
        let min_rating_decimal = query.min_rating.map(|r| Decimal::from_f64_retain(r).unwrap_or_default());

        // Count total
        let total: i64 = if let Some(ref city) = city_lower {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM vendors 
                 WHERE is_active = true AND LOWER(city) = $1"
            )
            .bind(city)
            .fetch_one(&db)
            .await?
        } else if let Some(ref pattern) = search_pattern {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM vendors 
                 WHERE is_active = true 
                 AND (LOWER(business_name) LIKE $1 OR LOWER(business_description) LIKE $1)"
            )
            .bind(pattern)
            .fetch_one(&db)
            .await?
        } else if let Some(verified) = query.is_verified {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM vendors 
                 WHERE is_active = true AND is_verified = $1"
            )
            .bind(verified)
            .fetch_one(&db)
            .await?
        } else if let Some(min_rating) = min_rating_decimal {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM vendors 
                 WHERE is_active = true AND rating >= $1"
            )
            .bind(min_rating)
            .fetch_one(&db)
            .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM vendors WHERE is_active = true")
                .fetch_one(&db)
                .await?
        };

        // Fetch vendors with same filters
        let vendors: Vec<Vendor> = if let Some(ref city) = city_lower {
            sqlx::query_as(
                "SELECT * FROM vendors 
                 WHERE is_active = true AND LOWER(city) = $1
                 ORDER BY rating DESC NULLS LAST, created_at DESC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(city)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
        } else if let Some(ref pattern) = search_pattern {
            sqlx::query_as(
                "SELECT * FROM vendors 
                 WHERE is_active = true 
                 AND (LOWER(business_name) LIKE $1 OR LOWER(business_description) LIKE $1)
                 ORDER BY rating DESC NULLS LAST, created_at DESC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(pattern)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
        } else if let Some(verified) = query.is_verified {
            sqlx::query_as(
                "SELECT * FROM vendors 
                 WHERE is_active = true AND is_verified = $1
                 ORDER BY rating DESC NULLS LAST, created_at DESC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(verified)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
        } else if let Some(min_rating) = min_rating_decimal {
            sqlx::query_as(
                "SELECT * FROM vendors 
                 WHERE is_active = true AND rating >= $1
                 ORDER BY rating DESC NULLS LAST, created_at DESC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(min_rating)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
        } else {
            sqlx::query_as(
                "SELECT * FROM vendors 
                 WHERE is_active = true
                 ORDER BY rating DESC NULLS LAST, created_at DESC 
                 LIMIT $1 OFFSET $2"
            )
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&db)
            .await?
        };

        (vendors, total)
    } else {
        // No filters - simple query
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendors WHERE is_active = true")
            .fetch_one(&db)
            .await?;

        let vendors: Vec<Vendor> = sqlx::query_as(
            "SELECT * FROM vendors 
             WHERE is_active = true
             ORDER BY rating DESC NULLS LAST, created_at DESC 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&db)
        .await?;

        (vendors, total)
    };

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
    let vendor_responses: Vec<VendorResponse> = vendors.into_iter().map(|v| v.into()).collect();

    Ok(Json(PaginatedVendorsResponse {
        vendors: vendor_responses,
        page,
        limit,
        total,
        total_pages,
    }))
}

/// Get current user's vendor profile
pub async fn get_my_vendor(
    State(db): State<Db>,
    auth_user: AuthUser,
) -> AppResult<Json<VendorResponse>> {
    let vendor: Vendor = sqlx::query_as("SELECT * FROM vendors WHERE user_id = $1")
        .bind(auth_user.user_id)
        .fetch_optional(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(vendor.into()))
}

/// Get vendor by ID (PUBLIC - no auth required)
pub async fn get_vendor(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> AppResult<Json<VendorResponse>> {
    let vendor_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    let vendor: Vendor = sqlx::query_as("SELECT * FROM vendors WHERE id = $1 AND is_active = true")
        .bind(vendor_id)
        .fetch_optional(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(vendor.into()))
}

/// Update vendor profile (requires ownership or admin)
pub async fn update_vendor(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateVendorRequest>,
) -> AppResult<Json<VendorResponse>> {
    let vendor_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    // Verify ownership
    let vendor: Vendor = sqlx::query_as("SELECT * FROM vendors WHERE id = $1")
        .bind(vendor_id)
        .fetch_optional(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    if vendor.user_id != auth_user.user_id {
        return Err(AppError::BadRequest(
            "You can only update your own vendor profile".to_string(),
        ));
    }

    // Convert lat/long to Decimal
    let latitude = payload
        .latitude
        .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());
    let longitude = payload
        .longitude
        .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());

    // Update vendor
    let updated_vendor: Vendor = sqlx::query_as(
        "UPDATE vendors SET
            business_name = COALESCE($2, business_name),
            business_description = COALESCE($3, business_description),
            logo_url = COALESCE($4, logo_url),
            banner_url = COALESCE($5, banner_url),
            business_email = COALESCE($6, business_email),
            business_phone = COALESCE($7, business_phone),
            business_address = COALESCE($8, business_address),
            city = COALESCE($9, city),
            state = COALESCE($10, state),
            postal_code = COALESCE($11, postal_code),
            latitude = COALESCE($12, latitude),
            longitude = COALESCE($13, longitude),
            operating_hours = COALESCE($14, operating_hours),
            service_radius_km = COALESCE($15, service_radius_km),
            bank_account_name = COALESCE($16, bank_account_name),
            bank_account_number = COALESCE($17, bank_account_number),
            bank_name = COALESCE($18, bank_name),
            updated_at = NOW()
         WHERE id = $1
         RETURNING *",
    )
    .bind(vendor_id)
    .bind(payload.business_name.as_ref().map(|s| s.trim()))
    .bind(payload.business_description.as_ref().map(|s| s.trim()))
    .bind(payload.logo_url.as_ref().map(|s| s.trim()))
    .bind(payload.banner_url.as_ref().map(|s| s.trim()))
    .bind(payload.business_email.as_ref().map(|s| s.trim()))
    .bind(payload.business_phone.as_ref().map(|s| s.trim()))
    .bind(payload.business_address.as_ref().map(|s| s.trim()))
    .bind(payload.city.as_ref().map(|s| s.trim()))
    .bind(payload.state.as_ref().map(|s| s.trim()))
    .bind(payload.postal_code.as_ref().map(|s| s.trim()))
    .bind(latitude)
    .bind(longitude)
    .bind(&payload.operating_hours)
    .bind(payload.service_radius_km)
    .bind(payload.bank_account_name.as_ref().map(|s| s.trim()))
    .bind(payload.bank_account_number.as_ref().map(|s| s.trim()))
    .bind(payload.bank_name.as_ref().map(|s| s.trim()))
    .fetch_one(&db)
    .await?;

    Ok(Json(updated_vendor.into()))
}

/// Get vendor dashboard statistics
pub async fn get_vendor_stats(
    State(db): State<Db>,
    auth_user: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    // Get vendor profile
    let vendor: Vendor = sqlx::query_as("SELECT * FROM vendors WHERE user_id = $1")
        .bind(auth_user.user_id)
        .fetch_optional(&db)
        .await?
        .ok_or(AppError::NotFound)?;

    // Get booking statistics
    let total_bookings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bookings b 
         JOIN services s ON b.service_id = s.id 
         WHERE s.vendor_id = $1",
    )
    .bind(vendor.id)
    .fetch_one(&db)
    .await?;

    let pending_bookings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bookings b 
         JOIN services s ON b.service_id = s.id 
         WHERE s.vendor_id = $1 AND b.status = 'pending'",
    )
    .bind(vendor.id)
    .fetch_one(&db)
    .await?;

    let completed_bookings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bookings b 
         JOIN services s ON b.service_id = s.id 
         WHERE s.vendor_id = $1 AND b.status = 'delivered'",
    )
    .bind(vendor.id)
    .fetch_one(&db)
    .await?;

    // Get total revenue (sum of total_price_cents for completed bookings)
    let total_revenue_cents: Option<i64> = sqlx::query_scalar(
        "SELECT SUM(b.total_price_cents) FROM bookings b 
         JOIN services s ON b.service_id = s.id 
         WHERE s.vendor_id = $1 AND b.status = 'delivered'",
    )
    .bind(vendor.id)
    .fetch_one(&db)
    .await?;

    let stats = json!({
        "vendor_id": vendor.id.to_string(),
        "business_name": vendor.business_name,
        "rating": vendor.rating,
        "total_reviews": vendor.total_reviews,
        "total_bookings": total_bookings,
        "pending_bookings": pending_bookings,
        "completed_bookings": completed_bookings,
        "total_revenue_cents": total_revenue_cents.unwrap_or(0),
        "is_verified": vendor.is_verified,
        "is_active": vendor.is_active,
    });

    Ok(Json(stats))
}
