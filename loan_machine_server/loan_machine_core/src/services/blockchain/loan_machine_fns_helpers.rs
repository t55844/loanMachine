use alloy::primitives::{Address, Bytes, FixedBytes, U256,};
use alloy::sol_types::SolCall;
use super::provider::Provider;
use super::abis::{LoanMachine, IERC20};
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

pub fn encode_approve(spender: Address, amount: U256) -> Bytes {
    Bytes::from(IERC20::approveCall {
        spender,
        amount,
    }.abi_encode())
}

pub async fn estimate_approve_gas(
        provider: &Provider,
        usdc_addr: Address,
        spender:   Address,
        amount:    U256,
        from:      Address,
    ) -> Result<U256, BlockchainError> {
        let gas = IERC20::new(usdc_addr, provider.clone())
            .approve(spender, amount).from(from)
            .estimate_gas().await
            .map_err(BlockchainError::from_gas_estimate)?;
        Ok(U256::from(gas))
    }

pub async fn get_erc20_allowance(
        provider:   &Provider,
        token_addr: Address,
        owner:      Address,
        spender:    Address,
    ) -> Result<U256, BlockchainError> {
        IERC20::new(token_addr, provider.clone())
            .allowance(owner, spender)
            .call()
            .await
            .map(|r| r._0)
            .map_err(BlockchainError::from_call)
    }

pub async fn get_erc20_balance(
        provider:   &Provider,
        token_addr: Address,
        owner:      Address,
    ) -> Result<U256, BlockchainError> {
        IERC20::new(token_addr, provider.clone())
            .balanceOf(owner)
            .call()
            .await
            .map(|r| r._0)
            .map_err(BlockchainError::from_call)
    }

pub fn encode_donate(amount: U256, member_id: FixedBytes<32>) -> Bytes {
    Bytes::from(LoanMachine::donateCall {
        amount,
        memberId: member_id,
    }.abi_encode())
}

pub fn encode_withdraw(amount: U256, member_id: FixedBytes<32>) -> Bytes {
    Bytes::from(LoanMachine::withdrawCall {
        amount,
        memberId: member_id,
    }.abi_encode())
}

pub async fn estimate_withdraw_gas(
        provider: &Provider,
        loan_machine_addr: Address,
        amount:    U256,
        member_id: FixedBytes<32>,
        from:      Address,
    ) -> Result<U256, BlockchainError> {
        let gas = LoanMachine::new(loan_machine_addr, provider.clone())
            .withdraw(amount, member_id)
            .from(from)
            .estimate_gas().await
            .map_err(BlockchainError::from_gas_estimate)?;
        Ok(U256::from(gas))
    }


