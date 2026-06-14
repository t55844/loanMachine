use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::DonationBundle;

/// Build the calldata + gas estimates the caller needs to sign in order
/// to donate `amount` (raw token units, as a decimal string) to the
/// cooperative's LoanMachine. Two signatures are required on the client:
/// an ERC20 `approve` followed by `LoanMachine.donate`.
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_donation(
    coop_id: String,
    amount:  String,
) -> Result<DonationBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::donation::donate_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? {
        return Err(ServerFnError::new("404 not found"));
    }
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    donate_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        &state.usdc_address.0,
        wallet,
        amount,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
