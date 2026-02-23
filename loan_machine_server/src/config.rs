// config.rs

// These are only needed on the server
#[cfg(feature = "ssr")]
use std::sync::Arc;
#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use leptos_config::LeptosOptions;
#[cfg(feature = "ssr")]
use crate::services::blockchain::BlockchainService;

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct SubgraphUrl(pub String);

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct ContractAddr(pub String);

#[cfg(feature = "ssr")]
#[derive(Clone, FromRef)]
pub struct AppState {
    pub subgraph_url: SubgraphUrl,
    pub contract_address: ContractAddr,
    pub blockchain_service: Arc<BlockchainService>,
    pub leptos_options: LeptosOptions,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub async fn new(leptos_options: LeptosOptions) -> Self {
        dotenvy::dotenv().ok();
        let subgraph_url = std::env::var("SUBGRAPH_URL")
            .expect("SUBGRAPH_URL must be set in .env file");
        let rpc_url = std::env::var("RPC_URL")
            .expect("RPC_URL must be set in .env file");
        let contract_address = std::env::var("CONTRACT_ADDRESS")
            .expect("CONTRACT_ADDRESS must be set in .env file");

        let service = BlockchainService::init(&rpc_url, &contract_address)
            .await
            .expect("Failed to initialize Blockchain Service");

        Self {
            subgraph_url: SubgraphUrl(subgraph_url),
            contract_address: ContractAddr(contract_address),
            blockchain_service: Arc::new(service),
            leptos_options,
        }
    }
}