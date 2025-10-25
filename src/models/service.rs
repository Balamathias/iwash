use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Service {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i32,
    pub price_per_kg_cents: i32,
    pub estimated_duration_hours: i32,
    pub is_active: bool,
    pub vendor_id: Option<Uuid>,
    pub is_featured: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ServiceResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i32,
    pub price_per_kg_cents: i32,
    pub estimated_duration_hours: i32,
    pub is_active: bool,
    pub vendor_id: Option<String>,
    pub is_featured: bool,
}

impl From<Service> for ServiceResponse {
    fn from(service: Service) -> Self {
        ServiceResponse {
            id: service.id.to_string(),
            name: service.name,
            description: service.description,
            base_price_cents: service.base_price_cents,
            price_per_kg_cents: service.price_per_kg_cents,
            estimated_duration_hours: service.estimated_duration_hours,
            is_active: service.is_active,
            vendor_id: service.vendor_id.map(|id| id.to_string()),
            is_featured: service.is_featured.unwrap_or(false),
        }
    }
}
