
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::requests::{CreateCoopRequest,RegisterDeployedCoopRequest};
use loan_machine_models::responses::{CoopDeployBundle, CoopRegistrationResult};

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn prepare_create_coop(req: CreateCoopRequest) -> Result<CoopDeployBundle, ServerFnError>{
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::create_coop::prepare_create_coop_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let _ = auth.wallet().await?;

    let state = expect_context::<AppState>();

    let bundle = prepare_create_coop_logic(
        &state.identity,
        &state.coop_deployment,
        req.name,
        req.founder_wallet,
        req.admin_wallets,
        req.threshold,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(bundle)
}

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn register_deployed_coop(
    req: RegisterDeployedCoopRequest,
) -> Result<CoopRegistrationResult, ServerFnError>{
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::create_coop::register_deployed_coop_logic;
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    let _ = auth.wallet().await?;

    let state = expect_context::<AppState>();

    let result = register_deployed_coop_logic(
        &state.coop_deployment,
        req.name,
        req.loan_machine_address,
        req.founder_wallet,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;


    Ok(CoopRegistrationResult{
        coop_id_hex: result.coop_id_hex,
        loan_machine_address: result.loan_machine_address,
        registration_tx_hash: result.registration_tx_hash,
    })
}