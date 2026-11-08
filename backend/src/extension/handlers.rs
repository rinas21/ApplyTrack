use axum::{extract::State, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{auth::AuthUser, common::{error::AppResult, response::ApiResponse}, AppState};

#[derive(Deserialize, ToSchema)]
pub struct CaptureInput {
    pub company_name: String,
    pub job_title: String,
    pub job_url: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub external_ref: Option<String>,
    pub mark_applied: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct CaptureResult { pub job_id: uuid::Uuid, pub application_id: Option<uuid::Uuid>, pub duplicate: bool }

pub fn router() -> Router<AppState> {
    Router::new().route("/capture", post(capture)).route("/recent", get(recent))
}

async fn capture(State(s): State<AppState>, user: AuthUser, Json(input): Json<CaptureInput>) -> AppResult<Json<ApiResponse<CaptureResult>>> {
    let source = input.source.unwrap_or_else(|| "extension".into());
    let job = crate::jobs::handlers::create_internal(&s.db, user.id, &crate::jobs::JobInput {
        company_id: None, company_name: input.company_name.clone(), title: input.job_title.clone(), job_url: input.job_url.clone(), description: input.description, location: input.location.clone(), country: None, city: None,
        salary_min: None, salary_max: None, salary_currency: None, employment_type: None, work_mode: None, source: Some(source.clone()), external_ref: input.external_ref, is_interesting: Some(false),
    }).await?;
    let mut app_id = None;
    let mut duplicate = false;
    if input.mark_applied.unwrap_or(false) {
        // avoid duplicate applied record for same URL
        if let Some(url) = &input.job_url {
            let existing: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM applications WHERE user_id=$1 AND job_url=$2 AND status <> 'Saved' LIMIT 1").bind(user.id).bind(url).fetch_optional(&s.db).await?;
            if let Some((id,)) = existing { app_id = Some(id); duplicate = true; }
        }
        if app_id.is_none() {
            let a = sqlx::query_scalar::<_, uuid::Uuid>("INSERT INTO applications (user_id,job_id,company_name,job_title,job_url,location,application_source,status,date_saved,date_applied) VALUES ($1,$2,$3,$4,$5,$6,$7,'Applied',CURRENT_DATE,CURRENT_DATE) RETURNING id")
                .bind(user.id).bind(job.id).bind(&input.company_name).bind(&input.job_title).bind(&input.job_url).bind(&input.location).bind(&source).fetch_one(&s.db).await?;
            sqlx::query("INSERT INTO application_status_history (application_id,user_id,to_status,source) VALUES ($1,$2,'Applied','extension')").bind(a).bind(user.id).execute(&s.db).await?;
            app_id = Some(a);
        }
    }
    Ok(Json(ApiResponse::new(CaptureResult { job_id: job.id, application_id: app_id, duplicate })))
}

async fn recent(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let rows = sqlx::query_as::<_, (uuid::Uuid,String,String,Option<String>,chrono::DateTime<chrono::Utc>)>(
        "SELECT id, company_name, title, job_url, created_at FROM jobs WHERE user_id=$1 ORDER BY created_at DESC LIMIT 10"
    ).bind(user.id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(rows.into_iter().map(|(id,c,t,u,at)| serde_json::json!({"id":id,"company_name":c,"title":t,"job_url":u,"created_at":at})).collect())))
}
