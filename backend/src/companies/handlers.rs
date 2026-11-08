use axum::{extract::{Path, State}, routing::get, Json, Router};
use uuid::Uuid;

use crate::{auth::AuthUser, common::{error::{AppError, AppResult}, response::ApiResponse}, AppState};
use super::{Company, CompanyInput};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(get_one).put(update).delete(remove))
        .route("/:id/stats", get(stats))
}

async fn list(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<Vec<Company>>>> {
    let items = sqlx::query_as::<_, Company>(
        "SELECT id, name, website, industry, headquarters, country, company_size, notes FROM companies WHERE user_id=$1 ORDER BY lower(name)",
    ).bind(user.id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(items)))
}

async fn create(State(s): State<AppState>, user: AuthUser, Json(input): Json<CompanyInput>) -> AppResult<Json<ApiResponse<Company>>> {
    if input.name.trim().is_empty() { return Err(AppError::Validation("name required".into())); }
    let item = sqlx::query_as::<_, Company>(
        "INSERT INTO companies (user_id,name,website,industry,headquarters,country,company_size,notes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING id,name,website,industry,headquarters,country,company_size,notes"
    ).bind(user.id).bind(input.name.trim()).bind(input.website).bind(input.industry).bind(input.headquarters).bind(input.country).bind(input.company_size).bind(input.notes)
    .fetch_one(&s.db).await.map_err(|e| match e { sqlx::Error::Database(d) if d.is_unique_violation()=>AppError::Conflict("company exists".into()), _=>AppError::Sqlx(e) })?;
    Ok(Json(ApiResponse::new(item)))
}

async fn get_one(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<Company>>> {
    let item = sqlx::query_as::<_, Company>(
        "SELECT id, name, website, industry, headquarters, country, company_size, notes FROM companies WHERE id=$1 AND user_id=$2"
    ).bind(id).bind(user.id).fetch_optional(&s.db).await?.ok_or_else(|| AppError::NotFound("company not found".into()))?;
    Ok(Json(ApiResponse::new(item)))
}

async fn update(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(input): Json<CompanyInput>) -> AppResult<Json<ApiResponse<Company>>> {
    let item = sqlx::query_as::<_, Company>(
        "UPDATE companies SET name=$3, website=$4, industry=$5, headquarters=$6, country=$7, company_size=$8, notes=$9, updated_at=now() WHERE id=$1 AND user_id=$2 RETURNING id,name,website,industry,headquarters,country,company_size,notes"
    ).bind(id).bind(user.id).bind(input.name.trim()).bind(input.website).bind(input.industry).bind(input.headquarters).bind(input.country).bind(input.company_size).bind(input.notes)
    .fetch_optional(&s.db).await?.ok_or_else(|| AppError::NotFound("company not found".into()))?;
    Ok(Json(ApiResponse::new(item)))
}

async fn remove(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let res = sqlx::query("DELETE FROM companies WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    if res.rows_affected()==0 { return Err(AppError::NotFound("company not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"deleted": true}))))
}

async fn stats(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let row = sqlx::query_as::<_, (i64,i64,i64,i64)>(
        "SELECT COUNT(*)::bigint, COUNT(*) FILTER (WHERE status='Interview')::bigint, COUNT(*) FILTER (WHERE status='Rejected')::bigint, COUNT(*) FILTER (WHERE status='Offer')::bigint FROM applications WHERE company_id=$1 AND user_id=$2 AND archived=false"
    ).bind(id).bind(user.id).fetch_one(&s.db).await?;
    Ok(Json(ApiResponse::new(serde_json::json!({"applications": row.0, "interviews": row.1, "rejections": row.2, "offers": row.3}))))
}
