// src/services/blockchain/factory.rs
//
// All interactions with LoanMachineFactory and the
// per-cooperative LoanMachine instances it deploys.
//
// Responsible for:
//   - Finding which coop a wallet belongs to
//   - Getting coop info and loan machine address
//   - Verifying access codes
//   - Checking wallet approval status
//   - Encoding calldata for join/vinculation
//   - Estimating gas for those calls

use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::sol_types::SolCall;
use std::sync::Arc;

use super::abis::{LoanMachine, LoanMachineFactory};
use super::provider::Provider;
use super::BlockchainError; 
use loan_machine_models::responses::CoopInfo;

pub struct FactoryService{
    pub provider: Arc<Provider>,
    pub factory_address: Address,
}

impl FactoryService{
    pub fn new(provider: Arc<Provider>, factory_address: Address) -> Self {
        Self {
            provider,
            factory_address,
        }
    }

    // ── QUERIES ──────────────────────────────────────────────

    /// Scans all coops to find which one this wallet is a member of.
    /// Returns FixedBytes::ZERO if not found.
    ///
    /// NOTE: This is O(n) over coop count. Fine for small numbers of coops.
    /// For large scale, use the subgraph instead (it indexes MemberJoined events).

    pub async fn get_wallet_coop(&self,wallet: Address) -> Result<FixedBytes<32>, BlockchainError> {
        let factory = LoanMachineFactory::new(self.factory_address, self.provider.clone());

        let coop_ids = factory
        .getAllCoops()
        .call()
        .await.map_err(|e| BlockchainError::Call(e.to_string()))?
        ._0;

        for coop_id in coop_ids{
            let loan_machine_addr = factory
            .getCoopInstance(coop_id)
            .call()
            .await
            .map_err(|e| BlockchainError::Call(e.to_string()))?
            ._0;

            let loan_machine = LoanMachine::new(loan_machine_addr, self.provider.clone());

            let is_member = loan_machine
            .isMember(wallet)
            .call()
            .await
            .map_err(|e| BlockchainError::Call(e.to_string()))?
            ._0;

            if is_member {
                return Ok(coop_id);
            }
        }
        Ok(FixedBytes::<32>::ZERO)
    }

    pub async fn get_coop_info(&self, coop_id: FixedBytes<32>) -> Result<CoopInfo, BlockchainError>{
        let factory = LoanMachineFactory::new(self.factory_address, self.provider.clone());

        let record = factory
        .coops(coop_id)
        .call()
        .await
        .map_err(|e| BlockchainError::Call(e.to_string()))?;

        if !record.exists{
            return Err(BlockchainError::CoopNotFound);
        }

        let loan_machine = LoanMachine::new(record.loanMachine, self.provider.clone());
        let active_loan_machine = loan_machine
        .isCoopActive()
        .call()
        .await
        .map_err(|e| BlockchainError::Call(e.to_string()))?
        ._0;

        Ok(CoopInfo{
            coop_id: format!("0x{}", hex::encode(coop_id)),
            name: record.name,
            loan_machine: record.loanMachine.to_string(),
            active: active_loan_machine,
        })
        

    }

    /// Returns the LoanMachine contract address for a given coopId.
    pub async fn get_loan_machine(&self, coop_id: FixedBytes<32>) 
    -> Result<Address, BlockchainError>{
        let factory = LoanMachineFactory::new(self.factory_address, self.provider.clone());

        factory.getCoopInstance(coop_id)
        .call()
        .await
        .map(|r| r._0)
        .map_err(|e| BlockchainError::Call(e.to_string()))
    }

    pub async fn verify_access_code(&self, coop_id: FixedBytes<32>, access_code: &str) 
    -> Result<bool, BlockchainError>{
        let loan_machine_addr = self.get_loan_machine(coop_id).await?;
        let loan_machine = LoanMachine::new(loan_machine_addr, self.provider.clone());

        let active = loan_machine
        .isCoopActive()
        .call()
        .await
        .map_err(|e| BlockchainError::Call(e.to_string()))?
        ._0;
        
        if !active{
            return Ok(false);
        }

        // The access code hash lives inside the LoanMachine.
        // We verify by attempting to call isWalletApproved with a known address,
        // but the actual hash check happens on-chain when joinCoop is submitted.
        //
        // For server-side pre-validation, we replicate the hash check here:

        use alloy::primitives::keccak256;

        let code_bytes = access_code.as_bytes();
        let _hash = keccak256(code_bytes);

        // Since we can't read private storage, we trust the on-chain revert.
        // This function exists to catch obviously wrong codes early (via a
        // separate public view you can add to LoanMachine if desired).
        // For now: return true and let the on-chain tx revert if wrong.
        Ok(true)
    }

    /// Returns whether a wallet has been pre-approved by the coop admin.
    pub async fn is_wallet_approved(&self, loan_machine_addr: Address, wallet: Address)
    -> Result<bool, BlockchainError>{
        LoanMachine::new(loan_machine_addr, self.provider.clone())
        .isWalletApproved(wallet)
        .call()
        .await
        .map(|r| r._0)
        .map_err(|e| BlockchainError::Call(e.to_string()))
    }

    // ── CALLDATA ENCODERS ────────────────────────────────────
    //
    // These produce the raw bytes the browser sends to the bundler.
    // They do NOT submit anything — that happens client-side via Privy.

    /// Encodes LoanMachine.joinCoop(memberId, wallet, accessCode)
    /// This single call handles: access code check + member vinculation.
    pub fn encode_join_coop(&self, member_id: u32, wallet: Address, access_code: &str)
    -> Bytes {
        Bytes::from(
            LoanMachine::joinCoopCall{
                memberId: member_id,
                wallet,
                accessCode: access_code.to_string()
            }
            .abi_encode(),
        )
    }

    /// Encodes LoanMachine.donate(amount, memberId)
    pub fn encode_donate(&self, amount: U256, member_id: u32) -> Bytes {
        Bytes::from(
            LoanMachine::donateCall {
                amount,
                memberId: member_id,
            }
            .abi_encode(),
        )
    }

    // ── GAS ESTIMATORS ───────────────────────────────────────

    pub async fn estimate_join_coop_gas(
        &self,
        loan_machine_addr: Address,
        member_id:         u32,
        wallet:            Address,
        access_code:       &str,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, self.provider.clone())
            .joinCoop(member_id, wallet, access_code.to_string())
            .from(wallet)
            .estimate_gas()
            .await
            .map_err(|e| BlockchainError::GasEstimate(e.to_string()))?;

        Ok(U256::from(gas))
    }

    pub async fn estimate_donate_gas(
        &self,
        loan_machine_addr: Address,
        amount:            U256,
        member_id:         u32,
        from:              Address,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, self.provider.clone())
            .donate(amount, member_id)
            .from(from)
            .estimate_gas()
            .await
            .map_err(|e| BlockchainError::GasEstimate(e.to_string()))?;

        Ok(U256::from(gas))
    }

}