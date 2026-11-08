use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{auth::AuthUser, common::response::ApiResponse, AppState};

#[derive(Serialize, ToSchema)]
pub struct MeResponse {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

pub fn settings_router() -> Router<AppState> {
    Router::new()
        .route("/preferences", get(get_prefs).put(update_prefs))
        .route("/subscription", get(get_subscription))
        .route("/export", get(export_data))
}

async fn get_prefs(State(s): State<AppState>, user: AuthUser) -> Json<ApiResponse<serde_json::Value>> {
    let row = sqlx::query_as::<_, (String,i32,bool)>("SELECT timezone, follow_up_days, email_notifications FROM user_preferences WHERE user_id=$1")
        .bind(user.id).fetch_optional(&s.db).await.ok().flatten();
    let (tz, days, notif) = row.unwrap_or(("UTC".into(), 7, false));
    Json(ApiResponse::new(serde_json::json!({"timezone":tz,"follow_up_days":days,"email_notifications":notif})))
}

#[derive(serde::Deserialize)]
struct PrefInput { timezone: Option<String>, follow_up_days: Option<i32>, email_notifications: Option<bool> }
async fn update_prefs(State(s): State<AppState>, user: AuthUser, Json(input): Json<PrefInput>) -> Json<ApiResponse<serde_json::Value>> {
    let _ = sqlx::query("UPDATE user_preferences SET timezone=COALESCE($2,timezone), follow_up_days=COALESCE($3,follow_up_days), email_notifications=COALESCE($4,email_notifications), updated_at=now() WHERE user_id=$1")
        .bind(user.id).bind(input.timezone).bind(input.follow_up_days).bind(input.email_notifications).execute(&s.db).await;
    get_prefs(State(s), user).await
}

async fn get_subscription(State(s): State<AppState>, user: AuthUser) -> Json<ApiResponse<serde_json::Value>> {
    let row = sqlx::query_as::<_, (String,String)>("SELECT plan, status FROM subscriptions WHERE user_id=$1").bind(user.id).fetch_optional(&s.db).await.ok().flatten();
    let (plan,status)=row.unwrap_or(("free".into(),"active".into()));
    Json(ApiResponse::new(serde_json::json!({"plan":plan,"status":status,"limits":{"free":{"active_applications":50},"pro":{"active_applications":5000,"email_sync":true,"advanced_analytics":true,"csv_export":true}}})))
}

async fn export_data(State(s): State<AppState>, user: AuthUser) -> Json<ApiResponse<serde_json::Value>> {
    let apps = sqlx::query_as::<_, (uuid::Uuid,String,String,String)>("SELECT id,company_name,job_title,status FROM applications WHERE user_id=$1 ORDER BY created_at DESC").bind(user.id).fetch_all(&s.db).await.unwrap_or_default();
    Json(ApiResponse::new(serde_json::json!({"user_id":user.id,"applications": apps.into_iter().map(|(id,c,t,st)| serde_json::json!({"id":id,"company":c,"title":t,"status":st})).collect::<Vec<_>>()})))
}

#[utoipa::path(get, path = "/api/v1/me", responses((status=200, body=MeResponse)))]
pub async fn me(user: AuthUser) -> Json<ApiResponse<MeResponse>> {
    Json(ApiResponse::new(MeResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
    }))
}
