use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use loan_machine_models::responses::{CoopInfo, VinculationBundle};
use loan_machine_models::requests::{DocKind};
use loan_machine_models::wallet_address::WalletAddress;

#[server(GetWalletCoop, "/api")]
pub async fn get_wallet_coop(smart_wallet: WalletAddress)
    -> Result<Option<CoopInfo>, ServerFnError>
{
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::vinculation::get_wallet_coop_logic;

    let state = use_context::<AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found"))?;

    get_wallet_coop_logic(&state.blockchain_service, smart_wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}



#[server(PrepareFirstVinculation, "/api")]
pub async fn prepare_first_vinculation(
    doc_kind:     DocKind,
    document:     String,
    smart_wallet: WalletAddress,
    coop_id:      String,
    access_code:  String,
) -> Result<VinculationBundle, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::vinculation::prepare_first_vinculation_logic;

    let state = use_context::<AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found"))?;

    prepare_first_vinculation_logic(
        &state.identity,
        &state.blockchain_service,
        &state.coop_registry_address.0,
        doc_kind,
        document,
        smart_wallet,
        coop_id,
        access_code,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}