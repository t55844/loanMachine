// loan_machine_web/src/server_fns/cooperatives.rs
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use loan_machine_models::responses::{CooperativeView, CoopViewerState};

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn list_cooperatives() -> Result<Vec<CooperativeView>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::subgraph_queries::cooperatives::fetch_cooperatives;
    use crate::server_fns::auth::Authenticated;

    let _auth = Authenticated::require().await?;        // ← single line, all gating done
    let state = expect_context::<AppState>();

    let rows = fetch_cooperatives(&state.subgraph)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows.into_iter().map(|r| CooperativeView {
        id: r.id,
        coop_id: r.coop_id,
        name: r.name,
        loan_machine: r.loan_machine,
        active: r.active,
        registered_at: r.registered_at.parse().unwrap_or(0),
    }).collect())
}


#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn get_coop_viewer_state(
    coop_id: String,
) -> Result<CoopViewerState, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::coop_view::get_viewer_state_logic;
    use crate::server_fns::auth::Authenticated;

    let auth   = Authenticated::require().await?;
    let wallet = auth.wallet().await?;
    let state  = expect_context::<AppState>();

    get_viewer_state_logic(&state.subgraph, &state.blockchain_service,&coop_id, wallet)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}