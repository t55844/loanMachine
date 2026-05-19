// loan_machine_web/src/server_fns/elections.rs
//
// Two #[server] functions, both thin wrappers around server_logic::elections.
//
//   get_current_election  — read.  Anyone authenticated can call it.
//   prepare_open_election — write bundle.  Caller wallet comes from the JWT
//                           via Authenticated::require(); never trusted from
//                           the request body.

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::{ElectionView, OpenElectionBundle};
use loan_machine_models::wallet_address::WalletAddress;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_current_election(
    coop_id: String,
) -> Result<Option<ElectionView>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::elections::get_current_election_logic;
    use crate::server_fns::auth::Authenticated;

    // We don't need the wallet for a read; require() is here so the endpoint
    // is auth-gated and shows up consistently in tracing/logging.
    let _auth = Authenticated::require().await?;
    let state = expect_context::<AppState>();

    get_current_election_logic(&state.blockchain_service, &coop_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_open_election(
    coop_id:          String,
    candidate_wallet: WalletAddress,
    opponent_wallet:  WalletAddress,
) -> Result<OpenElectionBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::elections::prepare_open_election_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let caller = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    prepare_open_election_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        candidate_wallet,
        opponent_wallet,
        caller,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}