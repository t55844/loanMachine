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
pub mod coop_registry;
pub mod provider;
pub mod deployable;
pub mod loan_machine_fns_helpers;
// Future modules (uncomment when implemented):
// pub mod loans;
// pub mod account;
// pub mod reputation;

use alloy::primitives::Address;
use std::sync::Arc;

pub use coop_registry::CoopRegistryService;
pub use provider::Provider;

pub mod contract_errors;
pub use contract_errors::{translate_revert, friendly_from_error, extract_revert_data};

use loan_machine_models::wallet_address::WalletAddress;

use crate::services::subgraph::SubgraphError;

// ── ERRORS ───────────────────────────────────────────────────
// One central error type for all blockchain operations.

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Invalid RPC URL: {0}")]
    InvalidRpcUrl(String),

    #[error("Invalid contract address")]
    InvalidAddress,

    /// A known on-chain revert with a translated user-facing message.
    /// The original alloy error is preserved as the source.
    #[error("{message}")]
    ContractRevert {
        message: String,
        #[source]source: alloy::contract::Error,
    },

    /// Any other contract-layer failure: RPC error, decode error,
    /// untranslated revert, etc. Source preserved for debugging.
    #[error("contract call failed")]
    Call(#[source] alloy::contract::Error),

    #[error("gas estimation failed")]
    GasEstimate(#[source] alloy::contract::Error),

    #[error("Cooperative not found")]
    CoopNotFound,

    #[error("provider call failed: {0}")]
    Provider(String),

    #[error("carteira não vinculada a nenhum membro: {0}")]
    WalletNotVinculated(WalletAddress),

    #[error(transparent)]
    Subgraph(#[from] SubgraphError),
}

impl BlockchainError {
    /// Classify an alloy contract error: known reverts get a translated
    /// message; everything else falls through to Call. Either way, the
    /// original alloy error is preserved in source().
    pub fn from_call(err: alloy::contract::Error) -> Self {
        if let Some(bytes) = extract_revert_data(&err) {
            Self::ContractRevert {
                message: translate_revert(&bytes),
                source: err,
            }
        } else {
            Self::Call(err)
        }
    }

    pub fn from_gas_estimate(err: alloy::contract::Error) -> Self {
        if let Some(bytes) = extract_revert_data(&err) {
            Self::ContractRevert {
                message: translate_revert(&bytes),
                source: err,
            }
        } else {
            Self::GasEstimate(err)
        }
    }
}

// ── MAIN SERVICE STRUCT ───────────────────────────────────────
// One BlockchainService per AppState.
// Each domain gets its own sub-service.

pub struct BlockchainService {
    pub raw_provider :      Arc<Provider>,
    pub coop_registry: CoopRegistryService,
}

impl BlockchainService {
    pub async fn init(
        rpc_url:         &str,
        coop_registry_addr:    &str,
    ) -> Result<Self, BlockchainError> {
        let raw_provider = Arc::new(provider::build_provider(rpc_url)?);

        let coop_registry_address: Address = coop_registry_addr
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress)?;

        Ok(Self {
            raw_provider:      raw_provider.clone(),
            coop_registry: CoopRegistryService::new(raw_provider, coop_registry_address),
        })
    }
}