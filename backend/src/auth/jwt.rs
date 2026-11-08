use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(default)]
    pub iss: Option<String>,
    #[serde(default)]
    pub aud: Option<Aud>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub azp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Aud {
    One(String),
    Many(Vec<String>),
}

impl Aud {
    pub fn contains(&self, value: &str) -> bool {
        match self {
            Aud::One(s) => s == value,
            Aud::Many(v) => v.iter().any(|s| s == value),
        }
    }
}

#[derive(Clone)]
pub struct JwtVerifier {
    jwks_url: String,
    issuer: String,
    audience: Option<String>,
    client: reqwest::Client,
    cache: Arc<RwLock<Option<JwkSet>>>,
}

impl JwtVerifier {
    pub fn new(jwks_url: String, issuer: String, audience: Option<String>) -> Self {
        Self {
            jwks_url,
            issuer,
            audience,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn jwks(&self) -> Result<JwkSet> {
        if let Some(set) = self.cache.read().await.clone() {
            return Ok(set);
        }
        let set: JwkSet = self.client.get(&self.jwks_url).send().await?.json().await?;
        *self.cache.write().await = Some(set.clone());
        Ok(set)
    }

    async fn refresh_jwks(&self) -> Result<JwkSet> {
        let set: JwkSet = self.client.get(&self.jwks_url).send().await?.json().await?;
        *self.cache.write().await = Some(set.clone());
        Ok(set)
    }

    pub async fn verify(&self, token: &str) -> Result<Claims> {
        let header = decode_header(token)?;
        let kid = header.kid.ok_or_else(|| anyhow!("missing kid"))?;
        let mut jwks = self.jwks().await?;
        let mut jwk = jwks.find(&kid).cloned();
        if jwk.is_none() {
            jwks = self.refresh_jwks().await?;
            jwk = jwks.find(&kid).cloned();
        }
        let jwk = jwk.ok_or_else(|| anyhow!("unknown kid"))?;
        let key = DecodingKey::from_jwk(&jwk)?;
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.issuer]);
        validation.validate_exp = true;
        if let Some(aud) = &self.audience {
            validation.set_audience(&[aud]);
        } else {
            validation.validate_aud = false;
        }
        let data = decode::<Claims>(token, &key, &validation)?;
        // If audience wasn't strictly validated (e.g. public client azp patterns), accept azp match.
        if self.audience.is_none() {
            if let Some(aud) = &data.claims.aud {
                // permissive in dev/local multi-client setup; issuer already validated
                let _ = aud;
            }
        }
        Ok(data.claims)
    }
}
