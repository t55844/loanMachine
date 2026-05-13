// loan_machine_core/src/services/blockchain/coop_registry.rs
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::sol_types::SolCall;
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

    pub async fn get_wallet_coop(&self, wallet: Address)
        -> Result<FixedBytes<32>, BlockchainError>
    {
        let registry = CoopRegistry::new(self.coop_registry_address, self.provider.clone());
        let coop_ids = registry
        .getAllCoops()
        .call()
        .await
            .map_err(BlockchainError::from_call)?._0;

        for coop_id in coop_ids {
            let lm_addr = registry
            .getCoopInstance(coop_id)
            .call()
            .await
                .map_err(BlockchainError::from_call)?._0;
            let is_member = LoanMachine::new(lm_addr, self.provider.clone())
                .isMember(wallet).call().await
                .map_err(BlockchainError::from_call)?._0;
            if is_member { return Ok(coop_id); }
        }
        Ok(FixedBytes::<32>::ZERO)
    }

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

    pub async fn is_wallet_approved(&self, loan_machine_addr: Address, wallet: Address)
        -> Result<bool, BlockchainError>
    {
        LoanMachine::new(loan_machine_addr, self.provider.clone())
            .isWalletApproved(wallet)
            .call()
            .await
            .map(|r| r._0)
            .map_err(BlockchainError::from_call)
    }

    // ── CALLDATA ─────────────────────────────────────────────

    pub fn encode_join_coop(
        &self, member_id: FixedBytes<32>, wallet: Address, access_code: &str
    ) -> Bytes {
        Bytes::from(LoanMachine::joinCoopCall {
            memberId: member_id,
            wallet,
            accessCode: access_code.to_string(),
        }.abi_encode())
    }

    pub fn encode_donate(&self, amount: U256, member_id: FixedBytes<32>) -> Bytes {
        Bytes::from(LoanMachine::donateCall {
            amount,
            memberId: member_id,
        }.abi_encode())
    }

    // ── GAS ──────────────────────────────────────────────────

    pub async fn estimate_join_coop_gas(
        &self,
        loan_machine_addr: Address,
        member_id:         FixedBytes<32>,
        wallet:            Address,
        access_code:       &str,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, self.provider.clone())
            .joinCoop(member_id, wallet, access_code.to_string())
            .from(wallet)
            .estimate_gas().await
            .map_err(BlockchainError::GasEstimate)?;
        Ok(U256::from(gas))
    }

    pub async fn estimate_donate_gas(
        &self,
        loan_machine_addr: Address,
        amount:            U256,
        member_id:         FixedBytes<32>,
        from:              Address,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, self.provider.clone())
            .donate(amount, member_id).from(from)
            .estimate_gas().await
            .map_err(BlockchainError::GasEstimate)?;
        Ok(U256::from(gas))
    }
}