// loan_machine_core/src/services/privy_auth/mod.rs

use std::time::{Duration, Instant};
use jsonwebtoken::{
    decode, decode_header,
    jwk::JwkSet,
    Algorithm, DecodingKey, Validation,
};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;

const JWKS_TTL: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Deserialize)]
pub struct PrivyClaims {
    pub sub: String,        // "did:privy:..."  Privy user id
    pub aud: String,        // your app id
    pub iss: String,        // "privy.io"
    pub exp: usize,
}

#[derive(Debug)]
pub enum PrivyAuthError {
    FetchJwks(String),
    MissingKid,
    KeyNotFound(String),
    InvalidToken(jsonwebtoken::errors::Error),
}

impl std::fmt::Display for PrivyAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivyAuthError::FetchJwks(e)   => write!(f, "failed to fetch JWKS: {e}"),
            PrivyAuthError::MissingKid     => write!(f, "JWT header missing kid"),
            PrivyAuthError::KeyNotFound(k) => write!(f, "no key found for kid: {k}"),
            PrivyAuthError::InvalidToken(e)=> write!(f, "invalid token: {e}"),
        }
    }
}

impl std::error::Error for PrivyAuthError {}

struct CachedJwks {
    jwks:       JwkSet,
    fetched_at: Instant,
}

pub struct PrivyAuthService {
    jwks_url: String,
    app_id:   String,
    http:     Client,
    cache:    RwLock<Option<CachedJwks>>,
}

impl PrivyAuthService {
    pub fn new(app_id: String) -> Self {
        Self {
            jwks_url: format!("https://auth.privy.io/api/v1/apps/{app_id}/jwks.json"),
            app_id,
            http:  Client::new(),
            cache: RwLock::new(None),
        }
    }

    /// Returns a fresh-enough JwkSet, fetching only if cache is empty or stale.
    async fn get_jwks(&self) -> Result<JwkSet, PrivyAuthError> {
        // Fast path: read lock, return cached copy if still valid.
        {
            let cache = self.cache.read().await;
            if let Some(c) = cache.as_ref() {
                if c.fetched_at.elapsed() < JWKS_TTL {
                    return Ok(c.jwks.clone());
                }
            }
        }

        // Slow path: fetch and update cache.
        let jwks: JwkSet = self.http
            .get(&self.jwks_url)
            .send().await
            .map_err(|e| PrivyAuthError::FetchJwks(e.to_string()))?
            .error_for_status()
            .map_err(|e| PrivyAuthError::FetchJwks(e.to_string()))?
            .json().await
            .map_err(|e| PrivyAuthError::FetchJwks(e.to_string()))?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedJwks {
            jwks:       jwks.clone(),
            fetched_at: Instant::now(),
        });

        Ok(jwks)
    }

    pub async fn verify(&self, token: &str) -> Result<PrivyClaims, PrivyAuthError> {
        // 1. Read kid from JWT header.
        let header = decode_header(token).map_err(PrivyAuthError::InvalidToken)?;
        let kid    = header.kid.ok_or(PrivyAuthError::MissingKid)?;

        // 2. Find matching key in (cached) JWKS.
        let jwks = self.get_jwks().await?;
        let jwk  = jwks.find(&kid)
            .ok_or_else(|| PrivyAuthError::KeyNotFound(kid.clone()))?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(PrivyAuthError::InvalidToken)?;

        // 3. Verify signature + claims.
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_audience(&[&self.app_id]);
        validation.set_issuer(&["privy.io"]);

        let data = decode::<PrivyClaims>(token, &decoding_key, &validation)
            .map_err(PrivyAuthError::InvalidToken)?;

        Ok(data.claims)
    }
}

impl std::fmt::Debug for PrivyAuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivyAuthService")
            .field("app_id",   &self.app_id)
            .field("jwks_url", &"[REDACTED]")
            .finish()
    }
}