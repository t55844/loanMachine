use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::WithdrawalBundle;

/// Build the calldata + gas estimate the caller needs to sign in order to
/// withdraw `amount` (raw token units, as a decimal string) from their
/// donation balance in the cooperative's LoanMachine. A single signature
/// is required on the client: `LoanMachine.withdraw`.
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_withdrawal(
    coop_id: String,
    amount:  String,
) -> Result<WithdrawalBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::withdrawal::withdraw_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? {
        return Err(ServerFnError::new("404 not found"));
    }
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    withdraw_logic(
        &state.subgraph,
        &state.blockchain_service,
        &coop_id,
        wallet,
        amount,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}
