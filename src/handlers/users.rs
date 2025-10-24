use axum::{extract::{Path, State}, http::StatusCode, Json};
use uuid::Uuid;

use crate::{
    db::Db,
    errors::{AppError, AppResult},
    middleware::AuthUser,
    models::{CreateUserRequest, UpdateUserRequest, UserResponse},
};

pub async fn list_users(
    State(db): State<Db>,
    _user: AuthUser,
) -> AppResult<Json<Vec<UserResponse>>> {
    let rows: Vec<(Uuid, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, email, full_name, phone FROM users ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&db)
    .await?;

    let users = rows
        .into_iter()
        .map(|(id, email, full_name, phone)| UserResponse {
            id: id.to_string(),
            email,
            full_name,
            phone,
        })
        .collect();

    Ok(Json(users))
}

pub async fn get_user(
    State(db): State<Db>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let (id, email, full_name, phone): (Uuid, String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT id, email, full_name, phone FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&db)
            .await?;

    Ok(Json(UserResponse {
        id: id.to_string(),
        email,
        full_name,
        phone,
    }))
}

pub async fn create_user(
    State(db): State<Db>,
    _user: AuthUser,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let email = payload.email.trim().to_lowercase();
    if email.is_empty() || payload.password.len() < 8 {
        return Err(AppError::BadRequest(
            "email required and password >= 8 chars".into(),
        ));
    }

    let full_name = payload
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let phone = payload
        .phone
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let id = Uuid::new_v4();
    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)?;

    let (id, email, full_name, phone): (Uuid, String, Option<String>, Option<String>) =
        sqlx::query_as(
            "INSERT INTO users (id, email, password_hash, full_name, phone) VALUES ($1, $2, $3, $4, $5) RETURNING id, email, full_name, phone",
        )
        .bind(id)
        .bind(&email)
        .bind(&password_hash)
        .bind(&full_name)
        .bind(&phone)
        .fetch_one(&db)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: id.to_string(),
            email,
            full_name,
            phone,
        }),
    ))
}

pub async fn update_user(
    State(db): State<Db>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let email = payload
        .email
        .as_deref()
        .map(|e| e.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    let full_name = payload
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let phone = payload
        .phone
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let password_hash: Option<String> = match payload.password {
        Some(ref p) if !p.is_empty() => Some(bcrypt::hash(p, bcrypt::DEFAULT_COST)?),
        _ => None,
    };

    let (id, email, full_name, phone): (Uuid, String, Option<String>, Option<String>) =
        sqlx::query_as(
            "UPDATE users SET
            email = COALESCE($2, email),
            password_hash = COALESCE($3, password_hash),
            full_name = COALESCE($4, full_name),
            phone = COALESCE($5, phone)
         WHERE id = $1
         RETURNING id, email, full_name, phone",
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(full_name)
        .bind(phone)
        .fetch_one(&db)
        .await?;

    Ok(Json(UserResponse {
        id: id.to_string(),
        email,
        full_name,
        phone,
    }))
}

pub async fn delete_user(
    State(db): State<Db>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_me(State(db): State<Db>, user: AuthUser) -> AppResult<Json<UserResponse>> {
    let (id, email, full_name, phone): (Uuid, String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT id, email, full_name, phone FROM users WHERE id = $1")
            .bind(user.user_id)
            .fetch_one(&db)
            .await?;

    Ok(Json(UserResponse {
        id: id.to_string(),
        email,
        full_name,
        phone,
    }))
}
