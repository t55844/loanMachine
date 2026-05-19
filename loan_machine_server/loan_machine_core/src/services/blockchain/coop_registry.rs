// loan_machine_core/src/services/blockchain/coop_registry.rs
use alloy::primitives::{Address, FixedBytes};
use std::sync::Arc;

use super::abis::{LoanMachine, CoopRegistry};
use super::provider::Provider;
use super::BlockchainError;
use loan_machine_models::responses::CoopInfo;

pub struct CoopRegistryService {
    pub provider: Arc<Provider>,
    pub coop_registry_address: Address,
}

impl CoopRegistryService {
    pub fn new(provider: Arc<Provider>, coop_registry_address: Address) -> Self {
        Self { provider, coop_registry_address }
    }

    // ── QUERIES ──────────────────────────────────────────────


    pub async fn get_coop_info(&self, coop_id: FixedBytes<32>)
        -> Result<CoopInfo, BlockchainError>
    {
        let registry = CoopRegistry::new(self.coop_registry_address, self.provider.clone());
        let record = registry
        .coops(coop_id)
        .call()
        .await
            .map_err(BlockchainError::from_call)?;
        if !record.exists { return Err(BlockchainError::CoopNotFound); }

        let stats = LoanMachine::new(record.loanMachine, self.provider.clone())
            .getCoopStats().call().await
            .map_err(BlockchainError::from_call)?;

        Ok(CoopInfo {
            coop_id: format!("0x{}", hex::encode(coop_id)),
            name:    record.name,
            loan_machine: record.loanMachine.to_string(),
            active:  stats.isActive,
        })
    }

    pub async fn get_loan_machine(&self, coop_id: FixedBytes<32>)
        -> Result<Address, BlockchainError>
    {
        CoopRegistry::new(self.coop_registry_address, self.provider.clone())
            .getCoopInstance(coop_id).call().await
            .map(|r| r._0)
            .map_err(BlockchainError::from_call)
    }

    pub async fn is_coop_joinable(&self, coop_id: FixedBytes<32>)
        -> Result<bool, BlockchainError>
    {
        let registry = CoopRegistry::new(self.coop_registry_address, self.provider.clone());
        let record = registry
        .coops(coop_id)
        .call()
        .await
            .map_err(BlockchainError::from_call)?;
        if !record.exists { return Ok(false); }

        let stats = LoanMachine::new(record.loanMachine, self.provider.clone())
            .getCoopStats()
            .call()
            .await
            .map_err(BlockchainError::from_call)?;
        Ok(stats.isActive)
    }



}