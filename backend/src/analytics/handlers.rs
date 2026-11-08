use axum::{extract::{Query, State}, routing::get, Json, Router};
use serde::Deserialize;
use crate::{auth::AuthUser, common::{error::AppResult, response::ApiResponse}, AppState};

#[derive(Deserialize)]
pub struct Filters { pub from: Option<String>, pub to: Option<String>, pub source: Option<String>, pub country: Option<String> }

pub fn router() -> Router<AppState> {
    Router::new().route("/summary", get(summary)).route("/dashboard", get(summary)).route("/export.csv", get(export_csv))
}

async fn summary(State(s): State<AppState>, user: AuthUser, Query(_f): Query<Filters>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // Submitted applications = status != Saved (or has date_applied). We define submitted as date_applied IS NOT NULL OR status NOT IN ('Saved').
    let totals = sqlx::query_as::<_, (i64,i64,i64,i64,i64,i64,i64,i64)>(
        r#"SELECT
            COUNT(*)::bigint as total,
            COUNT(*) FILTER (WHERE status <> 'Saved')::bigint as submitted,
            COUNT(*) FILTER (WHERE status='Interview')::bigint as interviews,
            COUNT(*) FILTER (WHERE status='Assessment')::bigint as assessments,
            COUNT(*) FILTER (WHERE status='Offer')::bigint as offers,
            COUNT(*) FILTER (WHERE status='Rejected')::bigint as rejected,
            COUNT(*) FILTER (WHERE status NOT IN ('Saved','Rejected','Withdrawn','Offer','No response'))::bigint as pending,
            COUNT(*) FILTER (WHERE status='Saved')::bigint as saved
           FROM applications WHERE user_id=$1 AND archived=false"#
    ).bind(user.id).fetch_one(&s.db).await?;

    let (total, submitted, interviews, assessments, offers, rejected, pending, saved) = totals;
    let response_rate = if submitted > 0 { (submitted - sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM applications WHERE user_id=$1 AND archived=false AND status='Applied' AND date_applied < CURRENT_DATE - INTERVAL '30 days'").bind(user.id).fetch_one(&s.db).await.unwrap_or(0)) as f64 / submitted as f64 * 100.0 } else { 0.0 };
    let interview_rate = if submitted > 0 { interviews as f64 / submitted as f64 * 100.0 } else { 0.0 };

    let status_dist = sqlx::query_as::<_, (String,i64)>("SELECT status, COUNT(*)::bigint FROM applications WHERE user_id=$1 AND archived=false GROUP BY status ORDER BY COUNT(*) DESC")
        .bind(user.id).fetch_all(&s.db).await?;
    let sources = sqlx::query_as::<_, (Option<String>,i64)>("SELECT application_source, COUNT(*)::bigint FROM applications WHERE user_id=$1 AND archived=false GROUP BY application_source ORDER BY COUNT(*) DESC NULLS LAST LIMIT 10")
        .bind(user.id).fetch_all(&s.db).await?;
    let by_month = sqlx::query_as::<_, (String,i64)>(
        "SELECT to_char(COALESCE(date_applied, created_at::date), 'YYYY-MM') as m, COUNT(*)::bigint FROM applications WHERE user_id=$1 AND archived=false AND status <> 'Saved' GROUP BY m ORDER BY m DESC LIMIT 12"
    ).bind(user.id).fetch_all(&s.db).await?;
    let companies = sqlx::query_as::<_, (String,i64)>("SELECT company_name, COUNT(*)::bigint FROM applications WHERE user_id=$1 AND archived=false GROUP BY company_name ORDER BY COUNT(*) DESC LIMIT 10")
        .bind(user.id).fetch_all(&s.db).await?;
    let recent = sqlx::query_as::<_, (uuid::Uuid,String,String,String,chrono::DateTime<chrono::Utc>)>(
        "SELECT id, company_name, job_title, status, updated_at FROM applications WHERE user_id=$1 ORDER BY updated_at DESC LIMIT 8"
    ).bind(user.id).fetch_all(&s.db).await?;

    Ok(Json(ApiResponse::new(serde_json::json!({
        "totals": {"total": total, "saved": saved, "submitted": submitted, "interviews": interviews, "assessments": assessments, "offers": offers, "rejections": rejected, "pending": pending},
        "rates": {"response_rate": (response_rate*10.0).round()/10.0, "interview_conversion_rate": (interview_rate*10.0).round()/10.0, "rejection_rate": if submitted>0 { (rejected as f64/submitted as f64*1000.0).round()/10.0 } else {0.0}, "offer_rate": if submitted>0 { (offers as f64/submitted as f64*1000.0).round()/10.0 } else {0.0}},
        "status_distribution": status_dist.into_iter().map(|(s,c)| serde_json::json!({"status":s,"count":c})).collect::<Vec<_>>(),
        "sources": sources.into_iter().map(|(s,c)| serde_json::json!({"source":s.unwrap_or_else(||"Unknown".into()),"count":c})).collect::<Vec<_>>(),
        "applications_by_month": by_month.into_iter().map(|(m,c)| serde_json::json!({"month":m,"count":c})).collect::<Vec<_>>(),
        "top_companies": companies.into_iter().map(|(n,c)| serde_json::json!({"company":n,"count":c})).collect::<Vec<_>>(),
        "recent_activity": recent.into_iter().map(|(id,co,t,st,at)| serde_json::json!({"id":id,"company_name":co,"job_title":t,"status":st,"updated_at":at})).collect::<Vec<_>>(),
        "definitions": {
            "submitted": "Applications with status other than Saved, or with an applied date.",
            "response_rate": "Approximate: submitted minus applications still in Applied status older than 30 days, divided by submitted. Refine with event data when available.",
            "interview_conversion_rate": "Interview applications divided by submitted applications."
        }
    }))))
}

async fn export_csv(State(s): State<AppState>, user: AuthUser) -> AppResult<impl axum::response::IntoResponse> {
    let rows = sqlx::query_as::<_, (String,String,String,Option<chrono::NaiveDate>,Option<String>)>(
        "SELECT company_name,job_title,status,date_applied,application_source FROM applications WHERE user_id=$1 ORDER BY created_at DESC"
    ).bind(user.id).fetch_all(&s.db).await?;
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["company","title","status","date_applied","source"]).ok();
    for r in rows { wtr.write_record([r.0,r.1,r.2,r.3.map(|d|d.to_string()).unwrap_or_default(),r.4.unwrap_or_default()]).ok(); }
    let data = wtr.into_inner().unwrap_or_default();
    Ok(([(axum::http::header::CONTENT_TYPE, "text/csv"), (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=applytrack-export.csv")], data))
}
