use axum::{extract::{Path, State}, routing::{get, patch}, Json, Router};
use uuid::Uuid;
use crate::{auth::AuthUser, common::{error::{AppError, AppResult}, response::ApiResponse}, AppState};
use super::{Reminder, ReminderInput};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list).post(create)).route("/:id/complete", patch(complete)).route("/:id/dismiss", patch(dismiss)).route("/suggestions", get(suggestions))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<Vec<Reminder>>>> {
    let items = sqlx::query_as::<_, Reminder>("SELECT id,application_id,title,due_at,status,source FROM reminders WHERE user_id=$1 ORDER BY due_at ASC").bind(user.id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(items)))
}
async fn create(State(s): State<AppState>, user: AuthUser, Json(input): Json<ReminderInput>) -> AppResult<Json<ApiResponse<Reminder>>> {
    if let Some(app_id)=input.application_id { let ok: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM applications WHERE id=$1 AND user_id=$2").bind(app_id).bind(user.id).fetch_optional(&s.db).await?; if ok.is_none(){ return Err(AppError::BadRequest("invalid application".into())); } }
    let item = sqlx::query_as::<_, Reminder>("INSERT INTO reminders (user_id,application_id,title,due_at) VALUES ($1,$2,$3,$4) RETURNING id,application_id,title,due_at,status,source")
        .bind(user.id).bind(input.application_id).bind(input.title).bind(input.due_at).fetch_one(&s.db).await?;
    Ok(Json(ApiResponse::new(item)))
}
async fn complete(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let r = sqlx::query("UPDATE reminders SET status='completed', completed_at=now() WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    if r.rows_affected()==0 { return Err(AppError::NotFound("reminder not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"ok":true}))))
}
async fn dismiss(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let r = sqlx::query("UPDATE reminders SET status='dismissed', completed_at=now() WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    if r.rows_affected()==0 { return Err(AppError::NotFound("reminder not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"ok":true}))))
}
async fn suggestions(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let rows = sqlx::query_as::<_, (uuid::Uuid,String,String)>("SELECT id, company_name, job_title FROM applications WHERE user_id=$1 AND archived=false AND status='Applied' AND COALESCE(date_applied, created_at::date) < CURRENT_DATE - INTERVAL '7 days' LIMIT 20")
        .bind(user.id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(rows.into_iter().map(|(id,c,t)| serde_json::json!({"application_id":id,"title":format!("Follow up: {} — {}", c,t)})).collect())))
}
