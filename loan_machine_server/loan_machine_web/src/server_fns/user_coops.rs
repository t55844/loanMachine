// loan_machine_web/src/server_fns/user_coops.rs
//
// One #[server] fn — get_user_coops().  Takes no args; the wallet comes
// from the JWT via Authenticated::require(), never trusted from the body.

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::UserCoop;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_user_coops() -> Result<Vec<UserCoop>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::subgraph_queries::user_coops::get_user_coops_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    // WalletAddress → "0x..." string (lowercased inside the query fn).
    let wallet_hex = format!("{wallet}");

    get_user_coops_logic(&state.subgraph, &wallet_hex)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}