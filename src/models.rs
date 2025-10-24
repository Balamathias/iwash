use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
	pub id: Uuid,
	pub email: String,
	pub password_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
	pub email: String,
	pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
	pub email: String,
	pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
	pub token: String,
}

