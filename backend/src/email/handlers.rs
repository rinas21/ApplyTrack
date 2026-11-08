use axum::{extract::{Path, State}, routing::{get, post, patch, delete}, Json, Router};
use serde::Deserialize;
use uuid::Uuid;
use crate::{auth::AuthUser, common::{error::{AppError, AppResult}, response::ApiResponse}, AppState};

#[derive(Deserialize)]
pub struct ConnectQuery { pub redirect: Option<String> }
#[derive(Deserialize)]
pub struct CallbackInput { pub code: String, pub email_address: Option<String> }
#[derive(Deserialize)]
pub struct CorrectInput { pub application_id: Option<Uuid>, pub status: String } // status: confirmed/rejected

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/accounts", get(list_accounts))
        .route("/gmail/auth-url", get(gmail_auth_url))
        .route("/gmail/callback", post(gmail_callback))
        .route("/accounts/:id/sync", post(sync_account))
        .route("/accounts/:id", delete(disconnect))
        .route("/activity", get(activity))
        .route("/matches/:id", patch(correct_match))
}

async fn list_accounts(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let rows = sqlx::query_as::<_, (Uuid,String,Option<String>,String)>("SELECT id,provider,email_address,status FROM connected_email_accounts WHERE user_id=$1 ORDER BY created_at DESC")
        .bind(user.id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(rows.into_iter().map(|(id,p,e,st)| serde_json::json!({"id":id,"provider":p,"email_address":e,"status":st})).collect())))
}

async fn gmail_auth_url(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let state = format!("{}:{}", user.id, uuid::Uuid::new_v4());
    let url = super::gmail::auth_url(&s.config, &state).map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(ApiResponse::new(serde_json::json!({"url": url, "state": state, "note": "Requires GOOGLE_CLIENT_ID/SECRET/REDIRECT_URI env vars."}))))
}

async fn gmail_callback(State(s): State<AppState>, user: AuthUser, Json(input): Json<CallbackInput>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let (access, refresh) = super::gmail::exchange_code(&s.config, &input.code).await.map_err(|e| AppError::BadRequest(e.to_string()))?;
    let refresh = refresh.ok_or_else(|| AppError::BadRequest("no refresh token returned; ensure prompt=consent and access_type=offline".into()))?;
    let enc = crate::common::crypto::encrypt(&s.config.token_encryption_key, &refresh).map_err(AppError::Internal)?;
    // best-effort email lookup
    let email_addr = if let Some(e)=input.email_address { Some(e) } else {
        #[derive(Deserialize)] struct U { email: Option<String> }
        match reqwest::Client::new().get("https://openidconnect.googleapis.com/v1/userinfo").bearer_auth(&access).send().await {
            Ok(r) => r.json::<U>().await.ok().and_then(|u| u.email),
            Err(_) => None,
        }
    };
    let id = sqlx::query_scalar::<_, Uuid>("INSERT INTO connected_email_accounts (user_id,provider,email_address,encrypted_refresh_token,scopes,status) VALUES ($1,'gmail',$2,$3,$4,'connected') RETURNING id")
        .bind(user.id).bind(email_addr).bind(enc).bind("gmail.readonly").fetch_one(&s.db).await?;
    Ok(Json(ApiResponse::new(serde_json::json!({"account_id":id,"connected":true}))))
}

async fn sync_account(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let n = super::gmail::sync_account(&s.config, &s.db, id, user.id).await.map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(ApiResponse::new(serde_json::json!({"synced_messages": n}))))
}

async fn disconnect(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    // delete synced data for account
    sqlx::query("DELETE FROM email_events WHERE account_id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    let r = sqlx::query("DELETE FROM connected_email_accounts WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&s.db).await?;
    if r.rows_affected()==0 { return Err(AppError::NotFound("account not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"disconnected":true,"data_deleted":true}))))
}

async fn activity(State(s): State<AppState>, user: AuthUser) -> AppResult<Json<ApiResponse<Vec<serde_json::Value>>>> {
    let rows = sqlx::query_as::<_, (Uuid,Option<String>,Option<String>,Option<String>,Option<String>,Option<chrono::DateTime<chrono::Utc>>,Option<Uuid>,Option<String>,Option<String>,Uuid)>(
        "SELECT e.id,e.from_address,e.subject,e.snippet,e.event_type,e.event_timestamp,m.application_id,m.status,e.provider,m.id as match_id FROM email_events e LEFT JOIN application_email_matches m ON m.email_event_id=e.id WHERE e.user_id=$1 ORDER BY e.created_at DESC LIMIT 100"
    ).bind(user.id).fetch_all(&s.db).await?;
    Ok(Json(ApiResponse::new(rows.into_iter().map(|(id,from,subj,snip,et,ts,app,ms,provider,mid)| serde_json::json!({"id":id,"match_id":mid,"from":from,"subject":subj,"snippet":snip,"event_type":et,"event_timestamp":ts,"application_id":app,"match_status":ms,"provider":provider})).collect())))
}

async fn correct_match(State(s): State<AppState>, user: AuthUser, Path(id): Path<Uuid>, Json(input): Json<CorrectInput>) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    let status = if input.status=="confirmed" { "confirmed" } else { "rejected" };
    let r = sqlx::query("UPDATE application_email_matches SET application_id=COALESCE($3,application_id), status=$4, corrected_by_user=true, updated_at=now() WHERE id=$1 AND user_id=$2")
        .bind(id).bind(user.id).bind(input.application_id).bind(status).execute(&s.db).await?;
    if r.rows_affected()==0 { return Err(AppError::NotFound("match not found".into())); }
    Ok(Json(ApiResponse::new(serde_json::json!({"ok":true}))))
}
