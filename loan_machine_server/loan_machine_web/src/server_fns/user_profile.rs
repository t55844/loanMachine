use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::UserProfile;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_user_profile() -> Result<UserProfile, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::user_profile::get_user_profile_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    get_user_profile_logic(&state.subgraph, &state.blockchain_service, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}