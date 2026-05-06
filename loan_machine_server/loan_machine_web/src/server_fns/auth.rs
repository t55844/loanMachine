#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos_axum::extract;
use axum::http::HeaderMap;

use loan_machine_core::config::AppState;
use loan_machine_core::services::privy_auth::PrivyClaims;


/// Pulls the bearer token from the `Authorization` header,
/// verifies it against Privy's JWKS, and returns the claims.
///
/// Use this at the top of every gated server function.
pub async fn require_auth() -> Result<PrivyClaims, ServerFnError> {
eprintln!("enter require_auth");
    let state = expect_context::<AppState>();
eprintln!("enter require_auth expect_context");

    let headers: HeaderMap = extract()
        .await
        .map_err(|e| ServerFnError::new(format!("headers: {e}")))?;
eprintln!("enter require_auth headers");

    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| ServerFnError::new("missing_auth_header"))?;
eprintln!("enter require_auth token");

    state
        .privy_auth
        .verify(token)
        .await
        .map_err(|e| ServerFnError::new(format!("auth: {e}")))
}