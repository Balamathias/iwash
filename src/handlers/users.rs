use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use uuid::Uuid;

use crate::{
    db::Db,
    errors::{AppError, AppResult},
    middleware::AuthUser,
    models::{CreateUserRequest, ListUsersQuery, PaginatedUsersResponse, UpdateUserRequest, UserResponse},
};

pub async fn list_users(
    State(db): State<Db>,
    _user: AuthUser,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<PaginatedUsersResponse>> {
    // Defaults and bounds for pagination (values are non-optional with serde defaults)
    let limit: u32 = query.limit.max(1).min(100);
    let page: u32 = query.page.max(1);
    let offset = (page - 1) as i64 * (limit as i64);

    // If search is provided, run filtered queries; otherwise run plain queries.
    if let Some(search_term) = query.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", search_term.to_lowercase());

        // Total matching rows
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE LOWER(email) LIKE $1 OR LOWER(full_name) LIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&db)
        .await?;

        // Paginated result set (note bind order: pattern, limit, offset)
        let rows: Vec<(Uuid, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT id, email, full_name, phone FROM users WHERE LOWER(email) LIKE $1 OR LOWER(full_name) LIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(limit as i64)
        .bind(offset)
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

        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

        return Ok(Json(PaginatedUsersResponse {
            users,
            page,
            limit,
            total,
            total_pages,
        }));
    }

    // No search term: full list with pagination
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&db)
        .await?;

    let rows: Vec<(Uuid, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, email, full_name, phone FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit as i64)
    .bind(offset)
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

    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    Ok(Json(PaginatedUsersResponse {
        users,
        page,
        limit,
        total,
        total_pages,
    }))
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
