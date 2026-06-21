// loan_machine_web/src/server_fns/vinculation.rs

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use loan_machine_models::responses::{CoopInfo, VinculationBundle};

/// Look up the coop (if any) the caller's wallet is bound to.
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_wallet_coop() -> Result<Option<CoopInfo>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::vinculation::get_wallet_coop_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state = expect_context::<AppState>();

    get_wallet_coop_logic(&state.subgraph, &state.blockchain_service, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Build the calldata + gas estimate the caller needs to sign in order
/// to bind their wallet to the cooperative on-chain.
/// The browser signs + broadcasts via the Privy bridge — the server never holds the key.
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_first_vinculation(
    coop_id: String,
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
        smart_wallet,
        coop_id,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
