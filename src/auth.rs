use axum::{extract::State, http::StatusCode, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;
use time::{Duration, OffsetDateTime};
use tracing::info;
use uuid::Uuid;

use crate::{db::Db, errors::{AppError, AppResult}, models::{LoginRequest, RegisterRequest, TokenResponse, User}};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
	sub: Uuid,
	exp: usize,
	iat: usize,
}

fn jwt_secret() -> Result<String, AppError> {
	env::var("JWT_SECRET").map_err(|_| AppError::Unauthorized)
}

pub async fn register(
	State(db): State<Db>,
	Json(payload): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<TokenResponse>)> {
	let email = payload.email.trim().to_lowercase();
	if email.is_empty() || payload.password.len() < 8 {
		return Err(AppError::BadRequest("email required and password >= 8 chars".into()));
	}

	// Check if exists
	let existing: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
		.bind(&email)
		.fetch_optional(&db)
		.await?;
	if existing.is_some() {
		return Err(AppError::BadRequest("email already registered".into()));
	}

	// Hash password
	let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)?;

	// Insert (generate UUID on app side to avoid DB extensions)
	let id = Uuid::new_v4();
	let user: User = sqlx::query_as::<_, User>(
		"INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) RETURNING id, email, password_hash",
	)
	.bind(id)
	.bind(&email)
	.bind(&password_hash)
	.fetch_one(&db)
	.await?;

	info!(user_id = %user.id, "User registered");

	let token = generate_token(user.id)?;
	Ok((StatusCode::CREATED, Json(TokenResponse { token })))
}

pub async fn login(
	State(db): State<Db>,
	Json(payload): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
	let email = payload.email.trim().to_lowercase();
	let user: Option<User> = sqlx::query_as(
		"SELECT id, email, password_hash FROM users WHERE email = $1",
	)
	.bind(&email)
	.fetch_optional(&db)
	.await?;

	let Some(user) = user else {
		return Err(AppError::Unauthorized);
	};

	let valid = bcrypt::verify(&payload.password, &user.password_hash)?;
	if !valid {
		return Err(AppError::Unauthorized);
	}

	let token = generate_token(user.id)?;

	info!(user_id = %user.id, "User logged in");

	Ok(Json(TokenResponse { token }))
}

fn generate_token(user_id: Uuid) -> AppResult<String> {
	let secret = jwt_secret()?;
	let now = OffsetDateTime::now_utc();
	let exp = now + Duration::hours(24);
	let claims = Claims {
		sub: user_id,
		iat: now.unix_timestamp() as usize,
		exp: exp.unix_timestamp() as usize,
	};
	let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
	Ok(token)
}

