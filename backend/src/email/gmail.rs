use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{common::crypto, config::Config, email::{classifier, matcher}};

#[derive(Deserialize)]
struct TokenResp { access_token: String, #[serde(default)] refresh_token: Option<String> }
#[derive(Deserialize)]
struct ListResp { #[serde(default)] messages: Vec<MsgRef>, #[serde(default, rename="nextPageToken")] _next_page_token: Option<String>, #[serde(default, rename="historyId")] history_id: Option<String> }
#[derive(Deserialize)]
struct MsgRef { id: String, #[serde(default, rename="threadId")] _thread_id: Option<String> }
#[derive(Deserialize)]
struct Msg { id: String, #[serde(default, rename="threadId")] thread_id: Option<String>, #[serde(default)] snippet: Option<String>, #[serde(default, rename="internalDate")] internal_date: Option<String>, #[serde(default)] payload: Option<Payload> }
#[derive(Deserialize, Default)]
struct Payload { #[serde(default)] headers: Vec<Header> }
#[derive(Deserialize)]
struct Header { name: String, value: String }

pub fn auth_url(cfg: &Config, state: &str) -> Result<String> {
    let client_id = cfg.google_client_id.as_deref().ok_or_else(|| anyhow!("GOOGLE_CLIENT_ID not configured"))?;
    let redirect = cfg.google_redirect_uri.as_deref().ok_or_else(|| anyhow!("GOOGLE_REDIRECT_URI not configured"))?;
    let scope = urlencoding::encode("https://www.googleapis.com/auth/gmail.readonly openid email");
    Ok(format!("https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={}",
        urlencoding::encode(client_id), urlencoding::encode(redirect), scope, urlencoding::encode(state)))
}

pub async fn exchange_code(cfg: &Config, code: &str) -> Result<(String, Option<String>)> {
    let client = Client::new();
    let resp = client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", cfg.google_client_id.as_deref().unwrap_or("")),
            ("client_secret", cfg.google_client_secret.as_deref().unwrap_or("")),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", cfg.google_redirect_uri.as_deref().unwrap_or("")),
        ]).send().await?.error_for_status()?.json::<TokenResp>().await?;
    Ok((resp.access_token, resp.refresh_token))
}

async fn access_token(cfg: &Config, refresh: &str) -> Result<String> {
    let client = Client::new();
    let resp = client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", cfg.google_client_id.as_deref().unwrap_or("")),
            ("client_secret", cfg.google_client_secret.as_deref().unwrap_or("")),
            ("refresh_token", refresh),
            ("grant_type", "refresh_token"),
        ]).send().await?.error_for_status()?.json::<TokenResp>().await?;
    Ok(resp.access_token)
}

pub async fn sync_account(cfg: &Config, pool: &PgPool, account_id: Uuid, user_id: Uuid) -> Result<usize> {
    let row = sqlx::query_as::<_, (Option<String>,)>("SELECT encrypted_refresh_token FROM connected_email_accounts WHERE id=$1 AND user_id=$2")
        .bind(account_id).bind(user_id).fetch_optional(pool).await?.ok_or_else(|| anyhow!("account not found"))?;
    let enc = row.0.ok_or_else(|| anyhow!("no refresh token"))?;
    let refresh = crypto::decrypt(&cfg.token_encryption_key, &enc)?;
    let token = access_token(cfg, &refresh).await?;
    let client = Client::new();
    // Simple recent-message sync (incremental historyId can be layered on via email_sync_states).
    let list = client.get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
        .bearer_auth(&token)
        .query(&[("maxResults","25"),("q","newer_than:30d (application OR interview OR assessment OR offer OR unfortunately OR recruiter)")])
        .send().await?.error_for_status()?.json::<ListResp>().await?;
    let mut count = 0usize;
    for m in list.messages.iter().take(25) {
        let msg = client.get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", m.id))
            .bearer_auth(&token).query(&[("format","metadata"),("metadataHeaders","Subject"),("metadataHeaders","From"),("metadataHeaders","Date")])
            .send().await?.error_for_status()?.json::<Msg>().await?;
        let headers = msg.payload.unwrap_or_default().headers;
        let subject = headers.iter().find(|h| h.name.eq_ignore_ascii_case("Subject")).map(|h| h.value.clone()).unwrap_or_default();
        let from = headers.iter().find(|h| h.name.eq_ignore_ascii_case("From")).map(|h| h.value.clone()).unwrap_or_default();
        let snippet = msg.snippet.unwrap_or_default();
        let cls = classifier::classify(&subject, &snippet, &from);
        let ts = msg.internal_date.and_then(|d| d.parse::<i64>().ok()).map(|ms| chrono::DateTime::from_timestamp_millis(ms)).flatten();
        let event_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO email_events (user_id,account_id,provider,provider_message_id,thread_id,from_address,subject,snippet,event_type,classification_confidence,event_timestamp) VALUES ($1,$2,'gmail',$3,$4,$5,$6,$7,$8,$9,$10)
             ON CONFLICT (user_id,provider,provider_message_id) DO UPDATE SET event_type=EXCLUDED.event_type, classification_confidence=EXCLUDED.classification_confidence, snippet=EXCLUDED.snippet RETURNING id"
        ).bind(user_id).bind(account_id).bind(&msg.id).bind(&msg.thread_id).bind(&from).bind(&subject).bind(&snippet).bind(&cls.event_type).bind(cls.confidence).bind(ts)
        .fetch_one(pool).await?;
        if let Some((app_id, conf)) = matcher::suggest_application(pool, user_id, &subject, &from, &snippet).await? {
            let status = if cls.confidence >= 0.8 && conf >= 0.7 { "auto_applied" } else { "suggested" };
            sqlx::query("INSERT INTO application_email_matches (user_id,email_event_id,application_id,match_confidence,status) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (email_event_id) DO UPDATE SET application_id=COALESCE(application_email_matches.application_id,EXCLUDED.application_id), match_confidence=EXCLUDED.match_confidence, status=CASE WHEN application_email_matches.corrected_by_user THEN application_email_matches.status ELSE EXCLUDED.status END")
                .bind(user_id).bind(event_id).bind(app_id).bind(conf).bind(status).execute(pool).await?;
            if status == "auto_applied" {
                if let Some(new_status) = matcher::status_for_event(&cls.event_type) {
                    let cur: Option<(String,)> = sqlx::query_as("SELECT status FROM applications WHERE id=$1 AND user_id=$2").bind(app_id).bind(user_id).fetch_optional(pool).await?;
                    if let Some((from_status,)) = cur {
                        if from_status != new_status {
                            sqlx::query("UPDATE applications SET status=$3, updated_at=now() WHERE id=$1 AND user_id=$2").bind(app_id).bind(user_id).bind(new_status).execute(pool).await?;
                            sqlx::query("INSERT INTO application_status_history (application_id,user_id,from_status,to_status,source,note) VALUES ($1,$2,$3,$4,'email',$5)")
                                .bind(app_id).bind(user_id).bind(from_status).bind(new_status).bind(format!("Auto-detected from Gmail (confidence {:.2}/{:.2})", cls.confidence, conf)).execute(pool).await?;
                        }
                    }
                }
            }
        }
        count += 1;
    }
    sqlx::query("INSERT INTO email_sync_states (account_id,user_id,last_synced_at) VALUES ($1,$2,now()) ON CONFLICT (account_id) DO UPDATE SET last_synced_at=now(), last_error=NULL, updated_at=now()")
        .bind(account_id).bind(user_id).execute(pool).await?;
    if let Some(h) = list.history_id { let _ = sqlx::query("UPDATE email_sync_states SET history_id=$2 WHERE account_id=$1").bind(account_id).bind(h).execute(pool).await; }
    Ok(count)
}
