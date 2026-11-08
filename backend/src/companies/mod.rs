pub mod handlers;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub headquarters: Option<String>,
    pub country: Option<String>,
    pub company_size: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompanyInput {
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub headquarters: Option<String>,
    pub country: Option<String>,
    pub company_size: Option<String>,
    pub notes: Option<String>,
}
