use axum::{extract::{Path, Query, State}, routing::get, Json, Router};
use serde::Deserialize;
use uuid::Uuid;
use crate::{auth::AuthUser, common::{error::{AppError, AppResult}, response::ApiResponse}, AppState};
use super::{Job, JobInput};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct JobQuery { pub q: Option<String>, pub interesting: Option<bool> }

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list).post(create)).route("/:id", get(get_one).put(update).delete(remove))
}

async fn list(State(s): State<AppState>, user: AuthUser, Query(q): Query<JobQuery>) -> AppResult<Json<ApiResponse<Vec<Job>>>> {
    let search = q.q.unwrap_or_default();
    let items = if search.is_empty() {
        sqlx::query_as::<_, Job>("SELECT id,company_id,company_name,title,job_url,description,location,country,employment_type,work_mode,source,is_interesting FROM jobs WHERE user_id=$1 AND ($2::bool IS NULL OR is_interesting=$2) ORDER BY created_at DESC")
            .bind(user.id).bind(q.interesting).fetch_all(&s.db).await?
    } else {
        let pat = format!("%{}%", search.to_lowercase());
        sqlx::query_as::<_, Job>("SELECT id,company_id,company_name,title,job_url,description,location,country,employment_type,work_mode,source,is_interesting FROM jobs WHERE user_id=$1 AND (lower(title) LIKE $2 OR lower(company_name) LIKE $2) ORDER BY created_at DESC")
            .bind(user.id).bind(pat).fetch_all(&s.db).await?
    };
    Ok(Json(ApiResponse::new(items)))
}

pub async fn create_internal(db: &PgPool, user_id: uuid::Uuid, input: &JobInput) -> anyhow::Result<Job> {
    if input.title.trim().is_empty() || input.company_name.trim().is_empty() { anyhow::bail!("title and company_name required"); }
    let item = sqlx::query_as::<_, Job>(
        r#"INSERT INTO jobs (user_id,company_id,company_name,title,job_url,description,location,country,city,salary_min,salary_max,salary_currency,employment_type,work_mode,source,external_ref,is_interesting)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
        ON CONFLICT (user_id, job_url) WHERE job_url IS NOT NULL AND job_url <> '' DO UPDATE SET title=EXCLUDED.title, description=COALESCE(EXCLUDED.description,jobs.description), updated_at=now()
        RETURNING id,company_id,company_name,title,job_url,description,location,country,employment_type,work_mode,source,is_interesting"#)
        .bind(user_id).bind(input.company_id).bind(input.company_name.trim()).bind(input.title.trim()).bind(&input.job_url).bind(&input.description).bind(&input.location).bind(&input.country).bind(&input.city)
        .bind(input.salary_min).bind(input.salary_max).bind(&input.salary_currency).bind(&input.employment_type).bind(&input.work_mode).bind(&input.source).bind(&input.external_ref).bind(input.is_interesting.unwrap_or(false))
        .fetch_one(db).await?;
    Ok(item)
}

async fn create(State(s): State<AppState>, user: AuthUser, Json(input): Json<JobInput>) -> AppResult<Json<ApiResponse<Job>>> {
    let item = create_internal(&s.db, user.id, &input).await.map_err(AppError::Internal)?;
    Ok(Json(ApiResponse::new(item)))
}

async fn get_one(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<Job>>> {
    let item = sqlx::query_as::<_, Job>("SELECT id,company_id,company_name,title,job_url,description,location,country,employment_type,work_mode,source,is_interesting FROM jobs WHERE id=$1 AND user_id=$2")
        .bind(id).bind(user.id).fetch_optional(&s.db).await?.ok_or_else(|| AppError::NotFound("job not found".into()))?;
    Ok(Json(ApiResponse::new(item)))
}

async fn update(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(input): Json<JobInput>) -> AppResult<Json<ApiResponse<Job>>> {
    let item = sqlx::query_as::<_, Job>(
        "UPDATE jobs SET company_id=$3,company_name=$4,title=$5,job_url=$6,description=$7,location=$8,country=$9,employment_type=$10,work_mode=$11,source=$12,is_interesting=$13,updated_at=now() WHERE id=$1 AND user_id=$2 RETURNING id,company_id,company_name,title,job_url,description,location,country,employment_type,work_mode,source,is_interesting")
        .bind(id).bind(user.id).bind(input.company_id).bind(input.company_name.trim()).bind(input.title.trim()).bind(input.job_url).bind(input.description).bind(input.location).bind(input.country).bind(input.employment_type).bind(input.work_mode).bind(input.source).bind(input.is_interesting.unwrap_or(false))
        .fetch_optional(&s.db).await?.ok_or_else(|| AppError::NotFound("job not found".into()))?;
    Ok(Json(ApiResponse::new(item)))
}

async fn remove(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let res = sqlx::query("DELETE FROM jobs WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    if res.rows_affected()==0 { return Err(AppError::NotFound("job not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"deleted": true}))))
}
