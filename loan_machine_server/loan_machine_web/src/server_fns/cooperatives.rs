// loan_machine_web/src/server_fns/cooperatives.rs

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

use loan_machine_models::responses::CooperativeView;

#[server]
pub async fn list_cooperatives(token: String) -> Result<Vec<CooperativeView>, ServerFnError> {
    use loan_machine_core::config::AppState;
    use loan_machine_core::server_logic::subgraph_queries::cooperatives::fetch_cooperatives;

    let state = expect_context::<AppState>();

    // Verify the token directly — no header fetching, no extract().
    let _claims = state
        .privy_auth
        .verify(&token)
        .await
        .map_err(|e| ServerFnError::new(format!("auth: {e}")))?;

    let rows = fetch_cooperatives(&state.subgraph)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| CooperativeView {
            id: r.id,
            coop_id: r.coop_id,
            name: r.name,
            loan_machine: r.loan_machine,
            active: r.active,
            registered_at: r.registered_at.parse().unwrap_or(0),
        })
        .collect())
}