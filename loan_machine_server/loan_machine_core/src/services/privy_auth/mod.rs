// src/services/privy_auth/mod.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

use loan_machine_models::wallet_address::{WalletAddress, WalletAddressParseError};

const DEFAULT_JWKS_TTL:   Duration = Duration::from_secs(300); // 5 min
const DEFAULT_WALLET_TTL: Duration = Duration::from_secs(900); // 15 min
const PRIVY_ISSUER: &str = "privy.io";

#[derive(Debug, Deserialize)]
pub struct PrivyClaims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
}

#[derive(Debug, Error)]
pub enum PrivyAuthError {
    #[error("failed to fetch JWKS: {0}")]
    FetchJwks(#[from] reqwest::Error),
    #[error("JWT header missing `kid`")]
    MissingKid,
    #[error("no key found for kid: {0}")]
    KeyNotFound(String),
    #[error("invalid token: {0}")]
    InvalidToken(#[from] jsonwebtoken::errors::Error),
    #[error("user has no embedded wallet linked")]
    NoEmbeddedWallet,
    #[error("Privy returned invalid wallet address: {0}")]
    InvalidWalletFromPrivy(#[from] WalletAddressParseError),
}

struct CachedJwks   { jwks: JwkSet,        fetched_at: Instant }
struct CachedWallet { wallet: WalletAddress, fetched_at: Instant }

pub struct PrivyAuthService {
    pub(crate) jwks_url: String,
    pub(crate) users_base_url: String,
    app_id:   String,
    app_secret: SecretString,
    jwks_ttl:   Duration,
    wallet_ttl: Duration,
    http: Client,
    jwks_cache:   RwLock<Option<CachedJwks>>,
    wallet_cache: RwLock<HashMap<String, CachedWallet>>,
}

impl PrivyAuthService {
    pub fn new(app_id: String, app_secret: SecretString) -> Self {
        let jwks_url = format!("https://auth.privy.io/api/v1/apps/{app_id}/jwks.json");
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client builds");
        Self {
            jwks_url,
            users_base_url: "https://auth.privy.io/api/v1".into(),
            app_id,
            app_secret,
            jwks_ttl:   DEFAULT_JWKS_TTL,
            wallet_ttl: DEFAULT_WALLET_TTL,
            http,
            jwks_cache:   RwLock::new(None),
            wallet_cache: RwLock::new(HashMap::new()),
        }
    }

    // ── JWKS + JWT verification (existing, slightly renamed) ─────

    async fn get_jwks(&self) -> Result<JwkSet, PrivyAuthError> {
        {
            let cache = self.jwks_cache.read().await;
            if let Some(c) = cache.as_ref() {
                if c.fetched_at.elapsed() < self.jwks_ttl {
                    return Ok(c.jwks.clone());
                }
            }
        }
        let jwks: JwkSet = self.http
            .get(&self.jwks_url)
            .send().await?
            .error_for_status()?
            .json().await?;
        let mut cache = self.jwks_cache.write().await;
        *cache = Some(CachedJwks { jwks: jwks.clone(), fetched_at: Instant::now() });
        Ok(jwks)
    }

    pub async fn verify(&self, token: &str) -> Result<PrivyClaims, PrivyAuthError> {
        let header = decode_header(token)?;
        let kid = header.kid.ok_or(PrivyAuthError::MissingKid)?;
        let jwks = self.get_jwks().await?;
        let jwk = jwks.find(&kid).ok_or_else(|| PrivyAuthError::KeyNotFound(kid.clone()))?;
        let decoding_key = DecodingKey::from_jwk(jwk)?;
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_audience(&[&self.app_id]);
        validation.set_issuer(&[PRIVY_ISSUER]);
        let data = decode::<PrivyClaims>(token, &decoding_key, &validation)?;
        Ok(data.claims)
    }

    // ── User-record lookup (Path 2) ──────────────────────────────

    /// Resolve a Privy DID (`sub`) to its linked embedded wallet address.
    /// Cached per `sub` for `wallet_ttl`; one HTTP round-trip on miss.
    pub async fn fetch_user_wallet(&self, sub: &str) -> Result<WalletAddress, PrivyAuthError> {
        // cache fast-path
        {
            let cache = self.wallet_cache.read().await;
            if let Some(c) = cache.get(sub) {
                if c.fetched_at.elapsed() < self.wallet_ttl {
                    return Ok(c.wallet.clone());
                }
            }
        }

        let url = format!(
            "{}/users/{}",
            self.users_base_url,
            urlencoding::encode(sub),
        );
        let basic = BASE64.encode(format!(
            "{}:{}", self.app_id, self.app_secret.expose_secret()
        ));

        let resp: PrivyUserResponse = self.http
            .get(&url)
            .header("Authorization", format!("Basic {basic}"))
            .header("privy-app-id", &self.app_id)
            .send().await?
            .error_for_status()?
            .json().await?;

        let raw = resp.linked_accounts.iter()
            .find_map(|a| match a {
                LinkedAccount::Wallet { address, wallet_client_type, .. }
                    if wallet_client_type.as_deref() == Some("privy") =>
                    Some(address.clone()),
                _ => None,
            })
            .ok_or(PrivyAuthError::NoEmbeddedWallet)?;

        let wallet: WalletAddress = raw.parse()?;   // From<WalletAddressParseError>

        let mut cache = self.wallet_cache.write().await;
        cache.insert(sub.to_string(), CachedWallet {
            wallet: wallet.clone(),
            fetched_at: Instant::now(),
        });
        Ok(wallet)
    }
}

#[derive(Debug, Deserialize)]
struct PrivyUserResponse {
    linked_accounts: Vec<LinkedAccount>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum LinkedAccount {
    #[serde(rename = "wallet")]
    Wallet {
        address: String,
        #[serde(default)]
        wallet_client_type: Option<String>,
        #[serde(flatten)]
        _rest: serde_json::Value,
    },
    #[serde(other)]
    Other,
}

impl std::fmt::Debug for PrivyAuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivyAuthService")
            .field("app_id", &self.app_id)
            .field("jwks_url", &"[REDACTED]")
            .field("app_secret", &"[REDACTED]")
            .finish()
    }
}