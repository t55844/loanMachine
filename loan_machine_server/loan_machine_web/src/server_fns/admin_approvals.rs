use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::{ApproveWalletPending, RequestApprovalBundle};

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_propose_wallet_as_admin(
    coop_id:       String,
    target_wallet: String,
) -> Result<RequestApprovalBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::admin_approvals::prepare_propose_wallet_as_admin_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let caller = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    prepare_propose_wallet_as_admin_logic(
        &state.blockchain_service, &coop_id, &target_wallet, caller,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn list_pending_approvals(coop_id: String) -> Result<ApproveWalletPending, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::admin_approvals::list_pending_approvals_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    list_pending_approvals_logic(&state.subgraph, &state.blockchain_service, &coop_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_confirm_proposal(coop_id: String, proposal_id: u64) -> Result<RequestApprovalBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::admin_approvals::prepare_confirm_proposal_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    prepare_confirm_proposal_logic(&state.blockchain_service, &coop_id, proposal_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_cosign_proposal(coop_id: String, proposal_id: u64) -> Result<RequestApprovalBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::admin_approvals::prepare_cosign_proposal_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    prepare_cosign_proposal_logic(&state.subgraph, &state.blockchain_service, &coop_id, proposal_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}