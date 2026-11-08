use std::sync::Arc;
use applytrack::{auth::jwt::JwtVerifier, build_router, config::Config, database, AppState};
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
    let jwt = JwtVerifier::new(config.jwks_url(), config.keycloak_issuer.clone(), config.keycloak_audience.clone());
    let state = AppState { db, config: config.clone(), jwt };
    let app = build_router(state);

    tracing::info!(addr=%config.bind_addr, "starting API");
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
