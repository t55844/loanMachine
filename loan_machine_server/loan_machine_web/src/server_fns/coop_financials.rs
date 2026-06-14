use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::CoopFinancials;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_coop_financials(
    coop_id: String,
) -> Result<CoopFinancials, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::coop_financials::get_coop_financials_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    if !auth.is_member(&coop_id).await? {
        return Err(ServerFnError::new("404 not found"));
    }
    let state = expect_context::<AppState>();

    get_coop_financials_logic(&state.blockchain_service, &coop_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
