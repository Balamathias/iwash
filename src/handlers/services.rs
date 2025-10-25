use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::Db,
    errors::{AppError, AppResult},
    middleware::{AuthUser, RequireVendor},
    models::{Service, ServiceResponse},
};

/// List all active services (PUBLIC - no auth required)
pub async fn list_services(State(db): State<Db>) -> AppResult<Json<Vec<ServiceResponse>>> {
    let services = sqlx::query_as::<_, Service>(
        "SELECT id, name, description, base_price_cents, price_per_kg_cents, 
                estimated_duration_hours, is_active, vendor_id, is_featured
         FROM services
         WHERE is_active = true
         ORDER BY is_featured DESC NULLS LAST, name ASC"
    )
    .fetch_all(&db)
    .await?;

    let responses: Vec<ServiceResponse> = services.into_iter().map(|s| s.into()).collect();
    
    Ok(Json(responses))
}

/// Get a specific service by ID (PUBLIC - no auth required)
pub async fn get_service(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> AppResult<Json<ServiceResponse>> {
    let service_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid service ID format".to_string()))?;

    let service = sqlx::query_as::<_, Service>(
        "SELECT id, name, description, base_price_cents, price_per_kg_cents, 
                estimated_duration_hours, is_active, vendor_id, is_featured
         FROM services
         WHERE id = $1"
    )
    .bind(service_id)
    .fetch_optional(&db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(service.into()))
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i32,
    pub price_per_kg_cents: i32,
    pub estimated_duration_hours: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub base_price_cents: Option<i32>,
    pub price_per_kg_cents: Option<i32>,
    pub estimated_duration_hours: Option<i32>,
    pub is_active: Option<bool>,
    pub is_featured: Option<bool>,
}

/// Create a new service for the vendor (Vendor role required)
pub async fn create_vendor_service(
    State(db): State<Db>,
    auth_user: AuthUser,
    Json(payload): Json<CreateServiceRequest>,
) -> AppResult<(StatusCode, Json<ServiceResponse>)> {
    // Check vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

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

    // Validate input
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Service name cannot be empty".to_string()));
    }

    if payload.base_price_cents < 0 {
        return Err(AppError::BadRequest("Base price cannot be negative".to_string()));
    }

    if payload.price_per_kg_cents < 0 {
        return Err(AppError::BadRequest("Price per kg cannot be negative".to_string()));
    }

    if payload.estimated_duration_hours <= 0 {
        return Err(AppError::BadRequest("Estimated duration must be greater than 0".to_string()));
    }

    // Create service
    let service_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO services (id, name, description, base_price_cents, price_per_kg_cents, 
                              estimated_duration_hours, is_active, vendor_id, is_featured)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(service_id)
    .bind(payload.name.trim())
    .bind(payload.description.as_ref().map(|s| s.trim()))
    .bind(payload.base_price_cents)
    .bind(payload.price_per_kg_cents)
    .bind(payload.estimated_duration_hours)
    .bind(true) // is_active defaults to true
    .bind(vendor_id)
    .bind(false) // is_featured defaults to false (admin-controlled)
    .execute(&db)
    .await?;

    // Fetch the created service
    let service = sqlx::query_as::<_, Service>(
        "SELECT id, name, description, base_price_cents, price_per_kg_cents, 
                estimated_duration_hours, is_active, vendor_id, is_featured
         FROM services
         WHERE id = $1"
    )
    .bind(service_id)
    .fetch_one(&db)
    .await?;

    tracing::info!("Vendor {} created service: {}", vendor_id, service_id);

    Ok((StatusCode::CREATED, Json(service.into())))
}

/// List all services for the authenticated vendor
pub async fn list_vendor_services(
    State(db): State<Db>,
    auth_user: AuthUser,
) -> AppResult<Json<Vec<ServiceResponse>>> {
    // Check vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

    // Get vendor ID for this user
    let vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM vendors WHERE user_id = $1 AND is_active = true"
    )
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?;

    let vendor_id = vendor_id.ok_or(AppError::BadRequest(
        "Vendor profile not found".to_string()
    ))?;

    let services = sqlx::query_as::<_, Service>(
        "SELECT id, name, description, base_price_cents, price_per_kg_cents, 
                estimated_duration_hours, is_active, vendor_id, is_featured
         FROM services
         WHERE vendor_id = $1
         ORDER BY is_active DESC, created_at DESC"
    )
    .bind(vendor_id)
    .fetch_all(&db)
    .await?;

    let responses: Vec<ServiceResponse> = services.into_iter().map(|s| s.into()).collect();
    
    Ok(Json(responses))
}

/// Update a service owned by the vendor
pub async fn update_vendor_service(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateServiceRequest>,
) -> AppResult<Json<ServiceResponse>> {
    // Check vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

    let service_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid service ID format".to_string()))?;

    // Get vendor ID for this user
    let vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM vendors WHERE user_id = $1 AND is_active = true"
    )
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?;

    let vendor_id = vendor_id.ok_or(AppError::BadRequest(
        "Vendor profile not found".to_string()
    ))?;

    // Verify service belongs to vendor
    let service_vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT vendor_id FROM services WHERE id = $1"
    )
    .bind(service_id)
    .fetch_optional(&db)
    .await?;

    match service_vendor_id {
        None => return Err(AppError::NotFound),
        Some(svi) if svi != vendor_id => {
            return Err(AppError::Forbidden(Some(
                "You don't have permission to update this service".to_string()
            )));
        }
        _ => {}
    }

    // Validate input
    if let Some(ref name) = payload.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Service name cannot be empty".to_string()));
        }
    }

    if let Some(base_price) = payload.base_price_cents {
        if base_price < 0 {
            return Err(AppError::BadRequest("Base price cannot be negative".to_string()));
        }
    }

    if let Some(price_per_kg) = payload.price_per_kg_cents {
        if price_per_kg < 0 {
            return Err(AppError::BadRequest("Price per kg cannot be negative".to_string()));
        }
    }

    if let Some(duration) = payload.estimated_duration_hours {
        if duration <= 0 {
            return Err(AppError::BadRequest("Estimated duration must be greater than 0".to_string()));
        }
    }

    // Update service
    sqlx::query(
        "UPDATE services
         SET name = COALESCE($1, name),
             description = COALESCE($2, description),
             base_price_cents = COALESCE($3, base_price_cents),
             price_per_kg_cents = COALESCE($4, price_per_kg_cents),
             estimated_duration_hours = COALESCE($5, estimated_duration_hours),
             is_active = COALESCE($6, is_active)
         WHERE id = $7"
    )
    .bind(payload.name.as_ref().map(|s| s.trim()))
    .bind(payload.description.as_deref().map(|s| s.trim()))
    .bind(payload.base_price_cents)
    .bind(payload.price_per_kg_cents)
    .bind(payload.estimated_duration_hours)
    .bind(payload.is_active)
    .bind(service_id)
    .execute(&db)
    .await?;

    // Fetch updated service
    let service = sqlx::query_as::<_, Service>(
        "SELECT id, name, description, base_price_cents, price_per_kg_cents, 
                estimated_duration_hours, is_active, vendor_id, is_featured
         FROM services
         WHERE id = $1"
    )
    .bind(service_id)
    .fetch_one(&db)
    .await?;

    tracing::info!("Vendor {} updated service: {}", vendor_id, service_id);

    Ok(Json(service.into()))
}

/// Delete (deactivate) a service owned by the vendor
pub async fn delete_vendor_service(
    State(db): State<Db>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    // Check vendor role
    let _vendor_role = RequireVendor::check(&db, &auth_user).await?;

    let service_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid service ID format".to_string()))?;

    // Get vendor ID for this user
    let vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM vendors WHERE user_id = $1 AND is_active = true"
    )
    .bind(auth_user.user_id)
    .fetch_optional(&db)
    .await?;

    let vendor_id = vendor_id.ok_or(AppError::BadRequest(
        "Vendor profile not found".to_string()
    ))?;

    // Verify service belongs to vendor
    let service_vendor_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT vendor_id FROM services WHERE id = $1"
    )
    .bind(service_id)
    .fetch_optional(&db)
    .await?;

    match service_vendor_id {
        None => return Err(AppError::NotFound),
        Some(svi) if svi != vendor_id => {
            return Err(AppError::Forbidden(Some(
                "You don't have permission to delete this service".to_string()
            )));
        }
        _ => {}
    }

    // Soft delete by setting is_active to false
    sqlx::query("UPDATE services SET is_active = false WHERE id = $1")
        .bind(service_id)
        .execute(&db)
        .await?;

    tracing::info!("Vendor {} deactivated service: {}", vendor_id, service_id);

    Ok(StatusCode::NO_CONTENT)
}

/// List services for a specific vendor (PUBLIC - no auth required)
pub async fn list_services_by_vendor(
    State(db): State<Db>,
    Path(vendor_id_str): Path<String>,
) -> AppResult<Json<Vec<ServiceResponse>>> {
    let vendor_id = Uuid::parse_str(&vendor_id_str)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    // Verify vendor exists
    let vendor_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vendors WHERE id = $1 AND is_active = true)"
    )
    .bind(vendor_id)
    .fetch_one(&db)
    .await?;

    if !vendor_exists {
        return Err(AppError::NotFound);
    }

    // Fetch active services for this vendor
    let services = sqlx::query_as::<_, Service>(
        "SELECT id, name, description, base_price_cents, price_per_kg_cents, 
                estimated_duration_hours, is_active, vendor_id, is_featured
         FROM services
         WHERE vendor_id = $1 AND is_active = true
         ORDER BY is_featured DESC NULLS LAST, name ASC"
    )
    .bind(vendor_id)
    .fetch_all(&db)
    .await?;

    let responses: Vec<ServiceResponse> = services.into_iter().map(|s| s.into()).collect();
    
    Ok(Json(responses))
}
