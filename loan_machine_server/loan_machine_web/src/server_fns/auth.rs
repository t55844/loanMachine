// loan_machine_web/src/server_fns/auth.rs
#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos_axum::extract;
use axum::http::HeaderMap;

use loan_machine_core::config::AppState;
use loan_machine_core::services::privy_auth::PrivyClaims;
use loan_machine_models::wallet_address::WalletAddress;

/// Proof that the current request was verified against Privy's JWKS.
///
/// Construction is private to this module — the *only* way to obtain one
/// is `Authenticated::require().await`, which performs the verification.
/// Any server fn that wants the user's identity asks for one of these as
/// its first step. The compiler then enforces that verification happens.
#[derive(Debug)]
pub struct Authenticated {
    claims: PrivyClaims,
}

impl Authenticated {
    /// Extract bearer token from headers, verify, return proof.
    /// Single source of truth for "is this request authenticated."
    pub async fn require() -> Result<Self, ServerFnError> {
        let state = expect_context::<AppState>();

        let headers: HeaderMap = extract()
            .await
            .map_err(|e| ServerFnError::new(format!("headers: {e}")))?;

        let token = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| ServerFnError::new("missing_auth_header"))?;

        let claims = state
            .privy_auth
            .verify(token)
            .await
            .map_err(|e| ServerFnError::new(format!("auth: {e}")))?;

        Ok(Self { claims })
    }

    pub fn claims(&self)  -> &PrivyClaims { &self.claims }
    pub fn user_id(&self) -> &str         { &self.claims.sub }
    /// The embedded wallet linked to this authenticated user, per Privy.
    /// Costs one HTTP round-trip per user per cache TTL.
    pub async fn wallet(&self) -> Result<WalletAddress, ServerFnError> {
        let state = expect_context::<AppState>();
        state.privy_auth
            .fetch_user_wallet(&self.claims.sub)
            .await
            .map_err(|e| ServerFnError::new(format!("wallet lookup: {e}")))
    }
}