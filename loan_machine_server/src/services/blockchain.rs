#![cfg(feature = "ssr")]  // add this at the top of blockchain.rs

use alloy::{
    primitives::{Address, Bytes, U256},
    providers::{ ProviderBuilder, RootProvider},
    sol,
    transports::http::{Client, Http},

};
use std::sync::Arc;

// sol! replaces abigen!, and accepts plain Solidity syntax
sol! {
    #[sol(rpc)]
    contract LoanContract {
        function donate(uint256 amount, uint32 memberId) external;
        function vinculationMemberToWallet(uint32 memberId, address wallet) external;
    }
}

pub struct BlockchainService {
    pub provider: Arc<RootProvider<Http<Client>>>,
    pub contract_address: Address,
}

impl BlockchainService {
    pub async fn init(rpc_url: &str, contract_addr: &str) -> Result<Self, BlockchainError> {
        let provider = ProviderBuilder::new()
            .on_http(rpc_url.parse().map_err(|e| BlockchainError::Provider(format!("{e}")))?)
            ;

        let address: Address = contract_addr
            .parse()
            .map_err(|_| BlockchainError::InvalidAddress)?;

        Ok(Self {
            provider: Arc::new(provider),
            contract_address: address,
        })
    }

    pub fn encode_vinculation_member(&self, member_id: u32, wallet: Address) -> Bytes {
        let call = LoanContract::vinculationMemberToWalletCall {
            memberId: member_id,
            wallet,
        };
        use alloy::sol_types::SolCall;
        Bytes::from(call.abi_encode())
    }

    pub async fn estimate_vinculation_member_to_wallet_gas(
        &self,
        member_id: u32,
        from: Address,
    ) -> Result<U256, BlockchainError> {
        let contract = LoanContract::new(self.contract_address, self.provider.clone());
        let gas = contract
            .vinculationMemberToWallet(member_id, from)
            .from(from)
            .estimate_gas()
            .await
            .map_err(|e| BlockchainError::GasEstimate(e.to_string()))?;
        Ok(U256::from(gas))
    }

    pub fn encode_donate(&self, amount: U256, member_id: u32) -> Bytes {
        let call = LoanContract::donateCall {
            amount,
            memberId: member_id,
        };
        use alloy::sol_types::SolCall;
        Bytes::from(call.abi_encode())
    }

    pub async fn estimate_donate_gas(
        &self,
        amount: U256,
        member_id: u32,
        from: Address,
    ) -> Result<U256, BlockchainError> {
        let contract = LoanContract::new(self.contract_address, self.provider.clone());
        let gas = contract
            .donate(amount, member_id)
            .from(from)
            .estimate_gas()
            .await
            .map_err(|e| BlockchainError::GasEstimate(e.to_string()))?;
        Ok(U256::from(gas))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Invalid contract address")]
    InvalidAddress,
    #[error("Gas estimation failed: {0}")]
    GasEstimate(String),
}