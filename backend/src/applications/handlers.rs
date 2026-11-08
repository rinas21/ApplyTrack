use axum::{extract::{Path, Query, State}, routing::{get, patch}, Json, Router};
use serde::Deserialize;
use uuid::Uuid;
use crate::{auth::AuthUser, common::{error::{AppError, AppResult}, pagination::PageParams, response::ApiResponse}, AppState};
use super::{Application, ApplicationInput, StatusChangeInput, StatusHistory, valid_status};

#[derive(Deserialize, Default)]
pub struct AppQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub company: Option<String>,
    pub country: Option<String>,
    pub source: Option<String>,
    pub q: Option<String>,
    pub archived: Option<bool>,
    pub sort: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .nest(
            "/:id",
            Router::new()
                .route("/", get(get_one).put(update).delete(remove))
                .route("/status", patch(change_status))
                .route("/history", get(history)),
        )
}

const APP_COLS: &str = "id,job_id,company_id,company_name,job_title,job_reference,job_url,location,country,city,industry,employment_type,application_source,date_saved,date_applied,status,notes,resume_version,cover_letter_ref,follow_up_date,archived,created_at,updated_at";

fn select_sql() -> String {
    format!("SELECT {} FROM applications", APP_COLS)
}

async fn list(State(s): State<AppState>, user: AuthUser, Query(q): Query<AppQuery>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let (limit, offset) = PageParams { page: q.page, page_size: q.page_size }.limit_offset();
    let archived = q.archived.unwrap_or(false);
    let search = q.q.as_deref().unwrap_or("").to_lowercase();
    let sort = match q.sort.as_deref() { Some("date_applied") => "date_applied DESC NULLS LAST, created_at DESC", Some("company") => "lower(company_name), created_at DESC", _ => "created_at DESC" };
    // Simple dynamic filtering via WHERE clauses built safely with parameters where possible; use COALESCE patterns.
    let status = q.status.filter(|v| !v.is_empty());
    let company = q.company.filter(|v| !v.is_empty()).map(|v| format!("%{}%", v.to_lowercase()));
    let country = q.country.filter(|v| !v.is_empty());
    let source = q.source.filter(|v| !v.is_empty());
    let qpat = if search.is_empty() { None } else { Some(format!("%{search}%")) };

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM applications WHERE user_id=$1 AND archived=$2 AND ($3::text IS NULL OR status=$3) AND ($4::text IS NULL OR lower(company_name) LIKE $4) AND ($5::text IS NULL OR country=$5) AND ($6::text IS NULL OR application_source=$6) AND ($7::text IS NULL OR lower(job_title) LIKE $7 OR lower(company_name) LIKE $7)"
    ).bind(user.id).bind(archived).bind(&status).bind(&company).bind(&country).bind(&source).bind(&qpat).fetch_one(&s.db).await?;

    let sql = format!("{} WHERE user_id=$1 AND archived=$2 AND ($3::text IS NULL OR status=$3) AND ($4::text IS NULL OR lower(company_name) LIKE $4) AND ($5::text IS NULL OR country=$5) AND ($6::text IS NULL OR application_source=$6) AND ($7::text IS NULL OR lower(job_title) LIKE $7 OR lower(company_name) LIKE $7) ORDER BY {} LIMIT $8 OFFSET $9", select_sql(), sort);
    let items = sqlx::query_as::<_, Application>(&sql)
        .bind(user.id).bind(archived).bind(&status).bind(&company).bind(&country).bind(&source).bind(&qpat).bind(limit).bind(offset)
        .fetch_all(&s.db).await?;

    Ok(Json(ApiResponse::new(serde_json::json!({"items": items, "total": total.0, "page": q.page.unwrap_or(1).max(1), "page_size": limit}))))
}

async fn create(State(s): State<AppState>, user: AuthUser, Json(input): Json<ApplicationInput>) -> AppResult<Json<ApiResponse<Application>>> {
    if input.company_name.trim().is_empty() || input.job_title.trim().is_empty() { return Err(AppError::Validation("company_name and job_title required".into())); }
    let status = input.status.unwrap_or_else(|| "Saved".into());
    if !valid_status(&status) { return Err(AppError::Validation("invalid status".into())); }
    let mut tx = s.db.begin().await?;
    let item = sqlx::query_as::<_, Application>(
        &format!(r#"INSERT INTO applications (user_id,job_id,company_id,company_name,job_title,job_reference,job_url,location,country,city,industry,salary_min,salary_max,salary_currency,employment_type,application_source,date_saved,date_applied,status,notes,resume_version,cover_letter_ref,follow_up_date,archived)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24) RETURNING {}"#, APP_COLS))
        .bind(user.id).bind(input.job_id).bind(input.company_id).bind(input.company_name.trim()).bind(input.job_title.trim()).bind(input.job_reference).bind(input.job_url).bind(input.location).bind(input.country).bind(input.city).bind(input.industry)
        .bind(input.salary_min).bind(input.salary_max).bind(input.salary_currency).bind(input.employment_type).bind(input.application_source).bind(input.date_saved).bind(input.date_applied).bind(&status).bind(input.notes).bind(input.resume_version).bind(input.cover_letter_ref).bind(input.follow_up_date).bind(input.archived.unwrap_or(false))
        .fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO application_status_history (application_id,user_id,from_status,to_status,source) VALUES ($1,$2,NULL,$3,'manual')").bind(item.id).bind(user.id).bind(&status).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(ApiResponse::new(item)))
}

async fn get_one(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<Application>>> {
    let item = sqlx::query_as::<_, Application>(&format!("{} WHERE id=$1 AND user_id=$2", select_sql()))
        .bind(id).bind(user.id).fetch_optional(&s.db).await?.ok_or_else(|| AppError::NotFound("application not found".into()))?;
    Ok(Json(ApiResponse::new(item)))
}

async fn update(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(input): Json<ApplicationInput>) -> AppResult<Json<ApiResponse<Application>>> {
    let status = input.status.unwrap_or_else(|| "Saved".into());
    if !valid_status(&status) { return Err(AppError::Validation("invalid status".into())); }
    let item = sqlx::query_as::<_, Application>(
        &format!("UPDATE applications SET company_id=$3,company_name=$4,job_title=$5,job_reference=$6,job_url=$7,location=$8,country=$9,city=$10,industry=$11,employment_type=$12,application_source=$13,date_saved=$14,date_applied=$15,notes=$16,resume_version=$17,cover_letter_ref=$18,follow_up_date=$19,archived=$20,updated_at=now() WHERE id=$1 AND user_id=$2 RETURNING {}", APP_COLS))
        .bind(id).bind(user.id).bind(input.company_id).bind(input.company_name.trim()).bind(input.job_title.trim()).bind(input.job_reference).bind(input.job_url).bind(input.location).bind(input.country).bind(input.city).bind(input.industry)
        .bind(input.employment_type).bind(input.application_source).bind(input.date_saved).bind(input.date_applied).bind(input.notes).bind(input.resume_version).bind(input.cover_letter_ref).bind(input.follow_up_date).bind(input.archived.unwrap_or(false))
        .fetch_optional(&s.db).await?.ok_or_else(|| AppError::NotFound("application not found".into()))?;
    Ok(Json(ApiResponse::new(item)))
}

async fn change_status(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(input): Json<StatusChangeInput>) -> AppResult<Json<ApiResponse<Application>>> {
    if !valid_status(&input.status) { return Err(AppError::Validation("invalid status".into())); }
    let mut tx = s.db.begin().await?;
    let current: Option<(String,)> = sqlx::query_as("SELECT status FROM applications WHERE id=$1 AND user_id=$2 FOR UPDATE").bind(id).bind(user.id).fetch_optional(&mut *tx).await?;
    let from = current.ok_or_else(|| AppError::NotFound("application not found".into()))?.0;
    let item = sqlx::query_as::<_, Application>(&format!("UPDATE applications SET status=$3, updated_at=now(), date_applied=CASE WHEN $3='Applied' AND date_applied IS NULL THEN CURRENT_DATE ELSE date_applied END WHERE id=$1 AND user_id=$2 RETURNING {}", APP_COLS))
        .bind(id).bind(user.id).bind(&input.status).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO application_status_history (application_id,user_id,from_status,to_status,source,note) VALUES ($1,$2,$3,$4,'manual',$5)")
        .bind(id).bind(user.id).bind(from).bind(&input.status).bind(input.note).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(ApiResponse::new(item)))
}

async fn history(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<Vec<StatusHistory>>>> {
    // ownership check
    let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM applications WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).fetch_optional(&s.db).await?;
    if exists.is_none() { return Err(AppError::NotFound("application not found".into())); }
    let items = sqlx::query_as::<_, StatusHistory>("SELECT id,from_status,to_status,changed_at,source,note FROM application_status_history WHERE application_id=$1 ORDER BY changed_at DESC")
        .bind(id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(items)))
}

async fn remove(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let res = sqlx::query("DELETE FROM applications WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    if res.rows_affected()==0 { return Err(AppError::NotFound("application not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"deleted": true}))))
}
