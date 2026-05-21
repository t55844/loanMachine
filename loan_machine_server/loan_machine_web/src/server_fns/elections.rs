// loan_machine_web/src/server_fns/elections.rs
//
// Four #[server] functions, all thin wrappers around server_logic::elections.
//
//   get_current_election    — read.  Anyone authenticated can call it.
//   get_wallet_reputation   — read.  Auth-gated; wallet comes from JWT, not body.
//   prepare_open_election   — write bundle.  Caller wallet from JWT.
//   prepare_vote            — write bundle.  Voter wallet from JWT.

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::{ElectionView, OpenElectionBundle, VoteBundle};
use loan_machine_models::wallet_address::WalletAddress;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_current_election(
    coop_id: String,
) -> Result<Option<ElectionView>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::elections::get_current_election_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? { return Err(ServerFnError::new("erro 404 pagina não encontrada")); }


    let state = expect_context::<AppState>();

    get_current_election_logic(&state.subgraph, &state.blockchain_service, &coop_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_wallet_reputation(
    coop_id: String,
) -> Result<i32, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::elections::get_wallet_reputation_logic;
    use crate::server_fns::auth::Authenticated;

    // Wallet from JWT — the client never gets to ask about someone else's rep.
    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? { return Err(ServerFnError::new("erro 404 pagina não encontrada")); }
    
    let wallet = auth.wallet().await?;


    let state  = expect_context::<AppState>();

    get_wallet_reputation_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        wallet,
    )
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

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? { return Err(ServerFnError::new("erro 404 pagina não encontrada")); }

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

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_vote(
    coop_id:          String,
    election_id:      u32,
    candidate_wallet: WalletAddress,
) -> Result<VoteBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::elections::prepare_vote_logic;
    use crate::server_fns::auth::Authenticated;

    // Voter wallet from JWT — body only carries who you're voting *for*.
    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? { return Err(ServerFnError::new("erro 404 pagina não encontrada")); }

    
    let voter = auth.wallet().await?;
    let state = expect_context::<AppState>();

    prepare_vote_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        election_id,
        candidate_wallet,
        voter,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}


#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_last_closed_election(
    coop_id: String,
) -> Result<Option<ElectionView>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::elections::get_last_closed_election_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? { return Err(ServerFnError::new("erro 404 pagina não encontrada")); }

    let state = expect_context::<AppState>();

    get_last_closed_election_logic(&state.subgraph, &state.blockchain_service, &coop_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}