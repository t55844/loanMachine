// src/config.rs

use std::sync::Arc;
use axum::extract::FromRef;
use leptos_config::LeptosOptions;
use crate::services::blockchain::BlockchainService;


#[derive(Clone)]
pub struct SubgraphUrl(pub String);

#[derive(Clone)]
pub struct ContractAddr(pub String);

#[derive(Clone, FromRef)]
pub struct AppState {
    pub subgraph_url:       SubgraphUrl,
    pub factory_address:    ContractAddr,
    pub blockchain_service: Arc<BlockchainService>,
    pub leptos_options:     LeptosOptions,
    pub privy_app_id:      String,
    pub chain_id:         u64,
}

impl AppState {
    pub async fn new(leptos_options: LeptosOptions) -> Self {
        dotenvy::dotenv().ok();

        let subgraph_url    = std::env::var("SUBGRAPH_URL")
            .expect("SUBGRAPH_URL must be set in .env");
        let rpc_url         = std::env::var("RPC_URL")
            .expect("RPC_URL must be set in .env");
        let factory_address = std::env::var("FACTORY_ADDRESS")
            .expect("FACTORY_ADDRESS must be set in .env");
        let privy_app_id = std::env::var("PRIVY_APP_ID")
            .expect("PRIVY_APP_ID must be set in .env");
        let chain_id = std::env::var("CHAIN_ID")
            .expect("CHAIN_ID must be set in .env")
            .parse::<u64>()
            .expect("CHAIN_ID must be a valid u64 integer");

         // Initialize the blockchain service with explicit type annotation.
        // Explicit type annotation — Rust sometimes needs help inferring
        // the return type of async functions through .await chains.
        let service: BlockchainService = BlockchainService::init(&rpc_url, &factory_address)
            .await
            .expect("Failed to initialize BlockchainService");

        Self {
            subgraph_url:       SubgraphUrl(subgraph_url),
            factory_address:    ContractAddr(factory_address),
            blockchain_service: Arc::new(service),
            leptos_options,
            privy_app_id,
            chain_id,
        }
    }
}