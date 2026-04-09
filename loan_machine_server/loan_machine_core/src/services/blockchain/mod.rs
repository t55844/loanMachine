// src/services/blockchain/mod.rs
//
// Public surface of the blockchain service.
// Imports all sub-modules and re-exports what the rest of the app needs.
//
// Adding a new contract in the future:
//   1. Add its sol! ABI to abis.rs
//   2. Create a new file (e.g. loans.rs) with its logic
//   3. Add pub mod loans; here
//   4. Add the new service to BlockchainService struct
//   5. Wire it up in BlockchainService::init()

pub mod abis;
pub mod factory;
pub mod provider;

// Future modules (uncomment when implemented):
// pub mod loans;
// pub mod account;
// pub mod reputation;

use alloy::primitives::Address;
use std::sync::Arc;

pub use factory::FactoryService;
pub use provider::Provider;

// ── ERRORS ───────────────────────────────────────────────────
// One central error type for all blockchain operations.

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),

    #[error("Invalid contract address")]
    InvalidAddress,

    #[error("Contract call failed: {0}")]
    Call(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimate(String),

    #[error("Cooperative not found")]
    CoopNotFound,
}

// ── MAIN SERVICE STRUCT ───────────────────────────────────────
// One BlockchainService per AppState.
// Each domain gets its own sub-service.

pub struct BlockchainService {
    pub factory: FactoryService,
    // pub loans:   LoansService,      ← add when ready
    // pub account: AccountService,    ← add when ready
}

impl BlockchainService {
    pub async fn init(
        rpc_url:         &str,
        factory_addr:    &str,
    ) -> Result<Self, BlockchainError> {
        // Build a single shared provider — all sub-services use the same connection
        let provider = Arc::new(provider::build_provider(rpc_url)?);

        let factory_address: Address = factory_addr
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress)?;

        Ok(Self {
            factory: FactoryService::new(provider.clone(), factory_address),
        })
    }
}