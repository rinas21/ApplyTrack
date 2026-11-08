pub mod handlers;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema, Clone)]
pub struct Application {
    pub id: Uuid,
    pub job_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub company_name: String,
    pub job_title: String,
    pub job_reference: Option<String>,
    pub job_url: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub industry: Option<String>,
    pub employment_type: Option<String>,
    pub application_source: Option<String>,
    pub date_saved: Option<NaiveDate>,
    pub date_applied: Option<NaiveDate>,
    pub status: String,
    pub notes: Option<String>,
    pub resume_version: Option<String>,
    pub cover_letter_ref: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplicationInput {
    pub job_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub company_name: String,
    pub job_title: String,
    pub job_reference: Option<String>,
    pub job_url: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub industry: Option<String>,
    pub salary_min: Option<f64>,
    pub salary_max: Option<f64>,
    pub salary_currency: Option<String>,
    pub employment_type: Option<String>,
    pub application_source: Option<String>,
    pub date_saved: Option<NaiveDate>,
    pub date_applied: Option<NaiveDate>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub resume_version: Option<String>,
    pub cover_letter_ref: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
    pub archived: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct StatusHistory {
    pub id: Uuid,
    pub from_status: Option<String>,
    pub to_status: String,
    pub changed_at: DateTime<Utc>,
    pub source: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StatusChangeInput {
    pub status: String,
    pub note: Option<String>,
}

pub const STATUSES: &[&str] = &["Saved","Applied","Assessment","Interview","Offer","Rejected","Withdrawn","No response"];
pub fn valid_status(s: &str) -> bool { STATUSES.contains(&s) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_known_statuses() {
        assert!(valid_status("Applied"));
        assert!(valid_status("No response"));
        assert!(!valid_status("Hired"));
        assert!(!valid_status(""));
    }
}
