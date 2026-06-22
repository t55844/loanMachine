use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::{LoanRequisitionBundle, LoanRequisitionItem};

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_loan_requisition(
    coop_id:       String,
    amount:        String,
    parcels_count: u32,
    days_interval: u32,
) -> Result<LoanRequisitionBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::loan_requisition::create_loan_requisition_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? {
        return Err(ServerFnError::new("404 not found"));
    }
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    create_loan_requisition_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        wallet,
        amount,
        parcels_count,
        days_interval,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_my_requisitions(
    coop_id: String,
) -> Result<Vec<LoanRequisitionItem>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::loan_requisition::fetch_my_requisitions_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? {
        return Ok(vec![]);
    }
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    fetch_my_requisitions_logic(&state.subgraph, &state.blockchain_service, &coop_id, &wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_cancel_requisition(
    coop_id:        String,
    requisition_id: String,
) -> Result<LoanRequisitionBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::loan_requisition::prepare_cancel_requisition_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    prepare_cancel_requisition_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        wallet,
        &requisition_id,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
