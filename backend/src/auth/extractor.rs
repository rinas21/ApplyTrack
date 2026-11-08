use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::{common::error::AppError, AppState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub keycloak_sub: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // Explicit dev bypass for local verification without Keycloak. Never enable in production.
        if state.config.dev_auth_bypass {
            let sub = parts
                .headers
                .get("x-dev-sub")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("dev-user");
            let email = parts.headers.get("x-dev-email").and_then(|v| v.to_str().ok());
            return crate::users::service::upsert_user(
                &state.db,
                sub,
                email,
                Some("Dev User"),
            )
            .await
            .map_err(AppError::Internal);
        }

        let auth = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".into()))?;
        let token = auth
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid authorization scheme".into()))?;
        let claims = state
            .jwt
            .verify(token)
            .await
            .map_err(|e| AppError::Unauthorized(format!("invalid token: {e}")))?;
        let display = claims.name.clone().or(claims.preferred_username.clone());
        crate::users::service::upsert_user(&state.db, &claims.sub, claims.email.as_deref(), display.as_deref())
            .await
            .map_err(AppError::Internal)
    }
}
