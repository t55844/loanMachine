// loan_machine_web/src/server_fns/vinculation.rs
//
// HTTP boundary for the vinculation flow.  Two endpoints:
//
//   get_wallet_coop          — "is the caller already vinculated, and if so to which coop?"
//   prepare_first_vinculation — build the signed-but-not-yet-broadcast tx bundle.
//
// Pattern, applied uniformly (cf. `cooperatives.rs`):
//   1. `client = AuthedBrowserClient` so the request carries an
//      `Authorization: Bearer <token>` header.
//   2. `Authenticated::require().await?` as the first line — without
//      this, anyone with the URL could call the endpoint.
//   3. `auth.wallet().await?` instead of trusting a client-supplied
//      wallet field.  The wallet derived from the JWT-verified `sub`
//      is the only wallet the server is willing to act on.
//   4. `expect_context::<AppState>()` — AppState is never optional
//      inside a server fn body; manual `use_context` + `ok_or_else`
//      is noise.
//   5. `e.to_string()` is fine *here* because this is the wire
//      boundary (auth.md §3, errors.md §VI: "stringify at the exit").
//      Anything below this layer keeps typed errors.

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use loan_machine_models::responses::{CoopInfo, VinculationBundle};
use loan_machine_models::requests::DocKind;

/// Look up the coop (if any) the caller's wallet is bound to.
///
/// Currently unused on the client (the old `VinculationGate` that
/// consumed it was removed).  Kept as a building block for future
/// "show me my coop" features.
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_wallet_coop() -> Result<Option<CoopInfo>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::vinculation::get_wallet_coop_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state = expect_context::<AppState>();

    get_wallet_coop_logic(&state.blockchain_service, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Build the calldata + gas estimate the caller needs to sign in order
/// to bind their (document, access code, coop) tuple to their wallet
/// on-chain.  The browser signs + broadcasts via the Privy bridge —
/// the server never holds the key.
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_first_vinculation(
    doc_kind:    DocKind,
    document:    String,
    coop_id:     String,
    access_code: String,
) -> Result<VinculationBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::vinculation::prepare_first_vinculation_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let smart_wallet = auth.wallet().await?;
    let state = expect_context::<AppState>();

    prepare_first_vinculation_logic(
        &state.identity,
        &state.blockchain_service,
        &state.coop_registry_address.0,
        doc_kind,
        document,
        smart_wallet,
        coop_id,
        access_code,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}