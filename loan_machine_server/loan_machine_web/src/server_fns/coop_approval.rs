// loan_machine_web/src/server_fns/coop_approval.rs

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::{ApprovalStatus, RequestApprovalBundle};

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_approval_status(coop_id: String) -> Result<ApprovalStatus, ServerFnError> {

    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::coop_approval::get_approval_status_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    get_approval_status_logic(&state.subgraph, &state.blockchain_service, &coop_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_request_approval(coop_id: String) -> Result<RequestApprovalBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::coop_approval::prepare_request_approval_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    prepare_request_approval_logic(&state.blockchain_service, &coop_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}   