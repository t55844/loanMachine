use alloy::primitives::{Address, Bytes, FixedBytes, U256,};
use alloy::sol_types::SolCall;
use super::provider::Provider;
use super::abis::{LoanMachine, };
use super::BlockchainError;



 pub async fn is_wallet_approved(provider: &Provider, loan_machine_addr: Address, wallet: Address)
        -> Result<bool, BlockchainError>
    {
        LoanMachine::new(loan_machine_addr, provider.clone())
            .isWalletApproved(wallet)
            .call()
            .await
            .map(|r| r._0)
            .map_err(BlockchainError::from_call)
    }


pub fn encode_join_coop(
         member_id: FixedBytes<32>, wallet: Address, access_code: &str
    ) -> Bytes {
        Bytes::from(LoanMachine::joinCoopCall {
            memberId: member_id,
            wallet,
            accessCode: access_code.to_string(),
        }.abi_encode())
    }

    
pub async fn estimate_join_coop_gas(
        provider: &Provider,
        loan_machine_addr: Address,
        member_id:         FixedBytes<32>,
        wallet:            Address,
        access_code:       &str,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, provider.clone())
            .joinCoop(member_id, wallet, access_code.to_string())
            .from(wallet)
            .estimate_gas().await
            .map_err(BlockchainError::from_gas_estimate)?;
        Ok(U256::from(gas))
    }

pub async fn estimate_donate_gas(
        provider: &Provider,
        loan_machine_addr: Address,
        amount:            U256,
        member_id:         FixedBytes<32>,
        from:              Address,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, provider.clone())
            .donate(amount, member_id).from(from)
            .estimate_gas().await
            .map_err(BlockchainError::from_gas_estimate)?;
        Ok(U256::from(gas))
    }


