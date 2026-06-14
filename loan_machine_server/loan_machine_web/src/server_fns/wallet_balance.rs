// loan_machine_web/src/server_fns/wallet_balance.rs
//
// Single #[server] fn for AuthBar's USD display.
// Read, so it just needs an authenticated caller; the wallet
// parameter is what gets queried, not what authenticates.

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_wallet_balance_wei() -> Result<String, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::wallet_balance::get_wallet_balance_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state = expect_context::<AppState>();

    get_wallet_balance_logic(&state.blockchain_service, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_wallet_balance_usdc() -> Result<String, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::wallet_balance::get_wallet_usdc_balance_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state = expect_context::<AppState>();

    get_wallet_usdc_balance_logic(&state.blockchain_service, &state.usdc_address.0, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}


/*API/server integration tests — the missing rung. Start the Axum server, hit it with real HTTP requests,
get realresponses back. Still no browser. Catches: serialization bugs, route wiring,
server-fn argument parsing, auth middleware, the seams between loan_machine_web and loan_machine_core. */