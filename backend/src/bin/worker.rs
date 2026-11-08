use std::sync::Arc;
use applytrack::{config::Config, database, email};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let config = Arc::new(Config::from_env()?);
    let db = database::connect(&config.database_url).await?;
    database::migrate(&db).await?;
    tracing::info!("starting worker");
    loop {
        let accounts = sqlx::query_as::<_, (uuid::Uuid, uuid::Uuid)>("SELECT id, user_id FROM connected_email_accounts WHERE provider='gmail' AND status='connected'")
            .fetch_all(&db).await.unwrap_or_default();
        for (account_id, user_id) in accounts {
            match email::gmail::sync_account(&config, &db, account_id, user_id).await {
                Ok(n) => tracing::info!(%account_id, n, "gmail sync ok"),
                Err(e) => {
                    tracing::warn!(%account_id, ?e, "gmail sync failed");
                    let _ = sqlx::query("UPDATE email_sync_states SET last_error=$2, updated_at=now() WHERE account_id=$1").bind(account_id).bind(e.to_string()).execute(&db).await;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(config.worker_poll_seconds)).await;
    }
}
