use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    db::Db,
    errors::{AppError, AppResult},
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
