// tests/common/app.rs

use std::sync::Arc;
use axum::Router;
use loan_machine_core::{
    config::{AppState, SubgraphUrl, ContractAddr},
    services::blockchain::BlockchainService,
};
use super::get_deployed;

// Returns the router directly — no real server, no port, no version conflict.
pub async fn build_test_router() -> Router {
    let env = get_deployed();

    let service = BlockchainService::init(&env.rpc_url, &env.factory_address)
        .await
        .expect("failed to init BlockchainService");

    let leptos_options = leptos_config::LeptosOptions::builder()
        .output_name("test")
        .site_root(".")
        .build();

    let state = AppState {
        subgraph_url:       SubgraphUrl(String::from("http://localhost")),
        factory_address:    ContractAddr(env.factory_address.clone()),
        blockchain_service: Arc::new(service),
        leptos_options,
        privy_app_id:       String::from("test"),
        chain_id:           31337,
    };

    loan_machine_web::create_app().with_state(state)
}