use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub keycloak_issuer: String,
    pub keycloak_audience: Option<String>,
    pub keycloak_jwks_url: Option<String>,
    pub cors_origin: String,
    pub token_encryption_key: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: Option<String>,
    pub worker_poll_seconds: u64,
    pub dev_auth_bypass: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://applytrack:applytrack@localhost:5432/applytrack".into()),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            keycloak_issuer: env::var("KEYCLOAK_ISSUER")
                .unwrap_or_else(|_| "http://localhost:8180/realms/applytrack".into()),
            keycloak_audience: env::var("KEYCLOAK_AUDIENCE").ok(),
            keycloak_jwks_url: env::var("KEYCLOAK_JWKS_URL").ok(),
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:4200".into()),
            token_encryption_key: env::var("TOKEN_ENCRYPTION_KEY")
                .unwrap_or_else(|_| "dev-only-32-byte-key-change-me!!".into()),
            google_client_id: env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").ok(),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI").ok(),
            worker_poll_seconds: env::var("WORKER_POLL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            dev_auth_bypass: env::var("DEV_AUTH_BYPASS").ok().as_deref() == Some("true"),
        })
    }

    pub fn jwks_url(&self) -> String {
        self.keycloak_jwks_url
            .clone()
            .unwrap_or_else(|| format!("{}/protocol/openid-connect/certs", self.keycloak_issuer.trim_end_matches('/')))
    }
}
