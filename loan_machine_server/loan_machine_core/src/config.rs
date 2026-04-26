// src/config.rs

use std::sync::Arc;
use axum::extract::FromRef;
use leptos_config::LeptosOptions;
use crate::services::blockchain::{BlockchainService};
use crate::services::identity::IdentityService;
use crate::services::privy::PrivyService;
use crate::services::chain_config::ChainConfig;
#[derive(Clone)]
pub struct FactoryAddress(pub String);

#[derive(Clone)]
pub struct SubgraphUrl(pub String);


#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options:     LeptosOptions,
    pub subgraph_url:       SubgraphUrl,
    pub factory_address:    FactoryAddress,
    pub blockchain_service: Arc<BlockchainService>,
    pub privy:              Arc<PrivyService>,
    pub identity:           Arc<IdentityService>,
    pub chain_config:       Arc<ChainConfig>,     

}

impl AppState {
    pub async fn new(leptos_options: LeptosOptions) -> Self {

        let identity = IdentityService::from_env()
            .expect("COOP_SALT must be set");
        let privy = PrivyService::from_env()
            .expect("PRIVY_APP_ID must be set");
        let chain_config = ChainConfig::from_env();

        let rpc_url = chain_config.rpc_url.clone();

        let subgraph_url    = std::env::var("SUBGRAPH_URL")
            .expect("SUBGRAPH_URL must be set in .env");
        let factory_address = std::env::var("FACTORY_ADDRESS")
            .expect("FACTORY_ADDRESS must be set in .env");

        let service: BlockchainService = BlockchainService::init(&rpc_url, &factory_address)
            .await
            .expect("Failed to initialize BlockchainService");

        Self {
            leptos_options,
            subgraph_url:       SubgraphUrl(subgraph_url),
            factory_address:    FactoryAddress(factory_address),
            blockchain_service: Arc::new(service),
            identity:           Arc::new(identity),
            privy:              Arc::new(privy),
            chain_config:       Arc::new(chain_config),
        }
    }
}