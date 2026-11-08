pub mod service;
pub mod handlers;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub keycloak_sub: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}
