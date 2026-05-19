use alloy::primitives::Address;
use alloy::providers::Provider; 
use thiserror::Error;

use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};

#[derive(Debug, Error)]
pub enum WalletError {
    #[error(transparent)]
    Blockchain(#[from] BlockchainError),
}

pub async fn get_wallet_balance_logic(
    blockchain: &BlockchainService,
    wallet:     WalletAddress,
) -> Result<String, WalletError> {
    let addr: Address = wallet_address::to_alloy(&wallet);

    let balance = blockchain.raw_provider
        .get_balance(addr)
        .await
        .map_err(|e| BlockchainError::Provider(e.to_string()))?;

    Ok(balance.to_string())
}

// in the same file as get_wallet_balance_logic:

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;
    use alloy::node_bindings::Anvil;
    use alloy::primitives::U256;
    use alloy::providers::ProviderBuilder;

    use loan_machine_models::wallet_address::from_alloy;
    use crate::services::blockchain::coop_registry::CoopRegistryService;

    /// Spins up an ephemeral anvil and wraps it in a minimal BlockchainService.
    /// The CoopRegistry inside is never touched by these tests — the address
    /// is just a placeholder.
    fn make_env() -> (alloy::node_bindings::AnvilInstance, BlockchainService) {
        let anvil    = Anvil::new().spawn();
        let provider = Arc::new(
            ProviderBuilder::new()
                .on_http(anvil.endpoint().parse().unwrap()),
        );
        let blockchain = BlockchainService {
            raw_provider:  provider.clone(),
            coop_registry: CoopRegistryService::new(provider, Address::ZERO),
        };
        (anvil, blockchain)
    }

    #[tokio::test]
    async fn returns_balance_for_funded_wallet() {
        let (anvil, blockchain) = make_env();

        // Anvil pre-funds its default accounts with 10_000 ETH.
        let funded = anvil.addresses()[0];
        let result = get_wallet_balance_logic(&blockchain, from_alloy(funded))
            .await
            .unwrap();

        let parsed: U256 = result.parse().expect("balance should parse as U256");
        assert!(parsed > U256::ZERO, "funded account should have non-zero balance");
    }

    #[tokio::test]
    async fn returns_zero_for_unfunded_wallet() {
        let (_anvil, blockchain) = make_env();

        let fresh: Address =
            "0x000000000000000000000000000000000000dEaD".parse().unwrap();
        let result = get_wallet_balance_logic(&blockchain, from_alloy(fresh))
            .await
            .unwrap();

        assert_eq!(result, "0");
    }
}