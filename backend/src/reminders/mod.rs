pub mod handlers;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Reminder {
    pub id: Uuid,
    pub application_id: Option<Uuid>,
    pub title: String,
    pub due_at: DateTime<Utc>,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReminderInput {
    pub application_id: Option<Uuid>,
    pub title: String,
    pub due_at: DateTime<Utc>,
}
