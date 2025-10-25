use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("hashing error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found")]
    NotFound,
    #[error("internal server error")]
    Internal,
    #[error("forbidden")]
    Forbidden,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, message) = match &self {
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db_error", self.to_string()),
            AppError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "hash_error", self.to_string()),
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "jwt_error", self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request", self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", self.to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal", self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
        };

        let body = Json(ErrorBody { error, message });
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
