// src/config.rs

use std::sync::Arc;
use axum::extract::FromRef;
use leptos_config::LeptosOptions;

use crate::services::blockchain::{BlockchainService};
use crate::services::identity::IdentityService;
use crate::services::privy::PrivyService;
use crate::services::chain_config::ChainConfig;
use crate::services::privy_auth::PrivyAuthService;
use crate::services::subgraph::SubgraphService;
use crate::services::coop_deployment::CoopDeploymentService;

use secrecy::SecretString;

#[derive(Clone)]
pub struct FactoryAddress(pub String);

#[derive(Clone)]
pub struct SubgraphUrl(pub String);


#[derive(Clone, FromRef)]
pub struct AppState {    
    pub leptos_options:     LeptosOptions,
    pub factory_address:    FactoryAddress,
    pub blockchain_service: Arc<BlockchainService>,
    pub identity:           Arc<IdentityService>,
    pub chain_config:       Arc<ChainConfig>,
    pub subgraph:           Arc<SubgraphService>,     
    pub privy:              Arc<PrivyService>,
    pub privy_auth:         Arc<PrivyAuthService>, 
    pub coop_deployment:    Arc<CoopDeploymentService>,  

}

impl AppState {
    pub async fn new(leptos_options: LeptosOptions) -> Self {

        let identity = IdentityService::from_env()
            .expect("COOP_SALT must be set");
        let chain_config = ChainConfig::from_env();

        let rpc_url = chain_config.rpc_url.clone();

    
        let factory_address = std::env::var("FACTORY_ADDRESS")
            .expect("FACTORY_ADDRESS must be set in .env");

        let blockchain_service : BlockchainService = BlockchainService::init(&rpc_url, &factory_address)
            .await
            .expect("Failed to initialize BlockchainService");

        let subgraph_url = std::env::var("SUBGRAPH_URL")
            .expect("SUBGRAPH_URL must be set in .env");
        let subgraph = SubgraphService::new(subgraph_url);


        let privy = PrivyService::from_env()
            .map_err(|e| format!("Failed to initialize PrivyService: {}", e))
            .expect("Failed to initialize PrivyService. Check environment variables.");

        let privy_app_id = privy.app_id();
        let privy_auth = PrivyAuthService::new(privy_app_id);

        let platform_admin_key: SecretString  = std::env::var("PLATFORM_ADMIN_PRIVATE_KEY")
            .expect("PLATFORM_ADMIN_PRIVATE_KEY must be set in .env")
            .into();
        let usdc_address = std::env::var("USDC_ADDRESS")
            .expect("USDC_ADDRESS must be set in .env");
        let loan_machine_bytecode_path = std::env::var("LOAN_MACHINE_BYTECODE_PATH")
            .expect("LOAN_MACHINE_BYTECODE_PATH must be set in .env");

        let bytecode_json = std::fs::read_to_string(&loan_machine_bytecode_path)
            .expect("Failed to read LoanMachine bytecode artifact");
        let artifact: serde_json::Value = serde_json::from_str(&bytecode_json)
            .expect("Failed to parse LoanMachine artifact JSON");
        let bytecode_hex = artifact["bytecode"].as_str()
            .expect("artifact.bytecode missing")
            .trim_start_matches("0x");
        let loan_machine_bytecode = hex::decode(bytecode_hex)
            .expect("LoanMachine bytecode is not valid hex");

        let coop_deployment = CoopDeploymentService::new(
            platform_admin_key,
            &factory_address,           // CoopRegistry address (your "factory")
            &rpc_url,
            loan_machine_bytecode,
            &usdc_address,
        ).expect("Failed to initialize CoopDeploymentService");

        Self {
            leptos_options,
            factory_address:    FactoryAddress(factory_address),
            blockchain_service: Arc::new(blockchain_service ),
            identity:           Arc::new(identity),
            privy:              Arc::new(privy),
            chain_config:       Arc::new(chain_config),
            privy_auth:         Arc::new(privy_auth),
            subgraph:           Arc::new(subgraph),
            coop_deployment:    Arc::new(coop_deployment),
        }
    }
}