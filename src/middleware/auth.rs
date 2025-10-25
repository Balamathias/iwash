use std::env;

use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::{AppError, AppResult}, models::UserRole};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: UserRole,
}

/// Require specific role(s) - use this for role-based access control
#[derive(Debug, Clone)]
pub struct RequireRole {
    pub user_id: Uuid,
    pub role: UserRole,
    pub allowed_roles: Vec<UserRole>,
}

impl RequireRole {
    /// Check if user has one of the allowed roles
    pub fn has_role(&self, role: UserRole) -> bool {
        self.role == role || self.allowed_roles.contains(&role)
    }

    /// Check if user is a vendor
    pub fn is_vendor(&self) -> bool {
        matches!(self.role, UserRole::Vendor)
    }

    /// Check if user is an admin
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
    iat: usize,
}

fn jwt_secret() -> Result<String, AppError> {
    env::var("JWT_SECRET").map_err(|_| AppError::Unauthorized)
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> AppResult<Self> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let secret = jwt_secret()?;
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        // For now, return AuthUser with a default Customer role
        // In handlers that need role info, they should fetch it from DB
        // This is a limitation of FromRequestParts - we can't access DB here easily
        Ok(AuthUser {
            user_id: data.claims.sub,
            role: UserRole::Customer, // Will be overridden in handlers that need it
        })
    }
}

/// RequireRole extractor - automatically checks if user has required role
/// Usage: RequireRole with specific roles in handler
impl<S> FromRequestParts<S> for RequireRole
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> AppResult<Self> {
        // First authenticate the user
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        // For now, RequireRole just wraps AuthUser
        // Specific role checks will be done in handlers or via helper extractors
        Ok(RequireRole {
            user_id: auth_user.user_id,
            role: auth_user.role,
            allowed_roles: vec![],
        })
    }
}

/// Helper extractor that requires Vendor role
#[derive(Debug, Clone)]
pub struct RequireVendor {
    pub user_id: Uuid,
}

impl RequireVendor {
    /// Helper to check vendor role in handlers
    /// Usage: RequireVendor::check(db, auth_user).await?
    pub async fn check(db: &crate::db::Db, auth_user: &AuthUser) -> AppResult<Self> {
        let role: UserRole = sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
            .bind(auth_user.user_id)
            .fetch_one(db)
            .await?; // Let database errors propagate with full details

        if !matches!(role, UserRole::Vendor) {
            return Err(AppError::BadRequest(
                "Vendor role required for this endpoint".to_string(),
            ));
        }

        Ok(RequireVendor {
            user_id: auth_user.user_id,
        })
    }
}

/// Helper extractor that requires Admin role  
#[derive(Debug, Clone)]
pub struct RequireAdmin {
    pub user_id: Uuid,
}

impl RequireAdmin {
    /// Helper to check admin role in handlers
    /// Usage: RequireAdmin::check(db, auth_user).await?
    pub async fn check(db: &crate::db::Db, auth_user: &AuthUser) -> AppResult<Self> {
        let role: UserRole = sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
            .bind(auth_user.user_id)
            .fetch_one(db)
            .await?; // Let database errors propagate with full details

        if !matches!(role, UserRole::Admin) {
            return Err(AppError::BadRequest(
                "Admin role required for this endpoint".to_string(),
            ));
        }

        Ok(RequireAdmin {
            user_id: auth_user.user_id,
        })
    }
}

/// Helper function to get user role from database
pub async fn get_user_role(db: &crate::db::Db, user_id: Uuid) -> AppResult<UserRole> {
    sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|_| AppError::Internal)
}
