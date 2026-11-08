pub mod handlers;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Job {
    pub id: Uuid,
    pub company_id: Option<Uuid>,
    pub company_name: String,
    pub title: String,
    pub job_url: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub employment_type: Option<String>,
    pub work_mode: Option<String>,
    pub source: Option<String>,
    pub is_interesting: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct JobInput {
    pub company_id: Option<Uuid>,
    pub company_name: String,
    pub title: String,
    pub job_url: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub salary_min: Option<f64>,
    pub salary_max: Option<f64>,
    pub salary_currency: Option<String>,
    pub employment_type: Option<String>,
    pub work_mode: Option<String>,
    pub source: Option<String>,
    pub external_ref: Option<String>,
    pub is_interesting: Option<bool>,
}
