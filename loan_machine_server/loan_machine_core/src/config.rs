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
use crate::services::blockchain::deployable::{LoanMachine as LoanMachineBytecode};
use crate::services::cache::MembershipCache;
use secrecy::SecretString;

#[derive(Clone)]
pub struct CoopRegistryAddress(pub String);

#[derive(Clone)]
pub struct UsdcAddress(pub String);

#[derive(Clone)]
pub struct SubgraphUrl(pub String);


#[derive(Clone, FromRef)]
pub struct AppState {    
    pub leptos_options:     LeptosOptions,
    pub coop_registry_address:    CoopRegistryAddress,
    pub blockchain_service: Arc<BlockchainService>,
    pub identity:           Arc<IdentityService>,
    pub chain_config:       Arc<ChainConfig>,
    pub subgraph:           SubgraphService,     
    pub privy:              Arc<PrivyService>,
    pub privy_auth:         Arc<PrivyAuthService>, 
    pub coop_deployment:    Arc<CoopDeploymentService>,
    pub cache_moka:   MembershipCache,
    pub usdc_address: UsdcAddress,

}

impl AppState {
    pub async fn new(leptos_options: LeptosOptions) -> Self {

        let identity = IdentityService::from_env()
            .expect("COOP_SALT must be set");
        let chain_config = ChainConfig::from_env();

        let rpc_url = chain_config.rpc_url.clone();

    
        let coop_registry_address = std::env::var("COOP_REGISTRY_ADDRESS")
             .expect("COOP_REGISTRY_ADDRESS must be set in .env");

        let blockchain_service : BlockchainService = BlockchainService::init(&rpc_url, &coop_registry_address)
            .await
            .expect("Failed to initialize BlockchainService");

        let subgraph_url = std::env::var("SUBGRAPH_URL")
            .expect("SUBGRAPH_URL must be set in .env");
        let subgraph = SubgraphService::new(subgraph_url);


        let privy = PrivyService::from_env()
            .map_err(|e| format!("Failed to initialize PrivyService: {}", e))
            .expect("Failed to initialize PrivyService. Check environment variables.");

        let privy_app_id = privy.app_id();
        let privy_app_secret: SecretString = std::env::var("PRIVY_APP_SECRET")
            .expect("PRIVY_APP_SECRET must be set in .env")
            .into();
        let privy_auth = PrivyAuthService::new(privy_app_id, privy_app_secret);

        let platform_admin_key: SecretString  = std::env::var("PLATFORM_ADMIN_PRIVATE_KEY")
            .expect("PLATFORM_ADMIN_PRIVATE_KEY must be set in .env")
            .into();

        let usdc_address = std::env::var("USDC_ADDRESS")
            .expect("USDC_ADDRESS must be set in .env");

        let loan_machine_bytecode = LoanMachineBytecode::BYTECODE.to_vec();

        let coop_deployment = CoopDeploymentService::new(
            platform_admin_key,
            &coop_registry_address,           // CoopRegistry address (your "factory")
            &rpc_url,
            loan_machine_bytecode,
            &usdc_address,
        ).expect("Failed to initialize CoopDeploymentService");

        let cache_moka = MembershipCache::new();

        Self {
            leptos_options,
            coop_registry_address:    CoopRegistryAddress(coop_registry_address),
            blockchain_service: Arc::new(blockchain_service ),
            identity:           Arc::new(identity),
            privy:              Arc::new(privy),
            chain_config:       Arc::new(chain_config),
            privy_auth:         Arc::new(privy_auth),
            subgraph:           subgraph,
            coop_deployment:    Arc::new(coop_deployment),
            cache_moka: cache_moka,
            usdc_address: UsdcAddress(usdc_address),
        }
    }
}