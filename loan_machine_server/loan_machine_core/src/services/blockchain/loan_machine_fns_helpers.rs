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


pub fn encode_join_coop(member_id: FixedBytes<32>, wallet: Address) -> Bytes {
    Bytes::from(LoanMachine::joinCoopCall {
        memberId: member_id,
        wallet,
    }.abi_encode())
}

pub async fn estimate_join_coop_gas(
    provider: &Provider,
    loan_machine_addr: Address,
    member_id: FixedBytes<32>,
    wallet: Address,
) -> Result<U256, BlockchainError> {
    let gas = LoanMachine::new(loan_machine_addr, provider.clone())
        .joinCoop(member_id, wallet)
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

pub fn encode_create_loan_requisition(
    amount:        U256,
    parcels_count: u32,
    member_id:     FixedBytes<32>,
    days_interval: u32,
) -> Bytes {
    Bytes::from(LoanMachine::createLoanRequisitionCall {
        amount,
        parcelscount:          parcels_count,
        memberId:              member_id,
        daysIntervalOfPayment: days_interval,
    }.abi_encode())
}

/// Returns `(currentCoverage, creationTime, status)` from the contract.
/// `status` is the authoritative on-chain value — always prefer it over the
/// subgraph's status, which can lag after a cancelLoanRequisition tx.
pub async fn get_requisition_info(
    provider:          &Provider,
    loan_machine_addr: Address,
    requisition_id:    U256,
) -> Result<(u32, u64, u8), BlockchainError> {
    let info = LoanMachine::new(loan_machine_addr, provider.clone())
        .getRequisitionInfo(requisition_id)
        .call().await
        .map_err(BlockchainError::from_call)?;
    Ok((info.currentCoverage, info.creationTime.saturating_to::<u64>(), info.status))
}

pub fn encode_cancel_loan_requisition(
    requisition_id: U256,
    member_id:      FixedBytes<32>,
) -> Bytes {
    Bytes::from(LoanMachine::cancelLoanRequisitionCall {
        requisitionId: requisition_id,
        memberId:      member_id,
    }.abi_encode())
}

pub async fn estimate_cancel_loan_requisition_gas(
    provider:          &Provider,
    loan_machine_addr: Address,
    from:              Address,
    requisition_id:    U256,
    member_id:         FixedBytes<32>,
) -> Result<U256, BlockchainError> {
    let gas = LoanMachine::new(loan_machine_addr, provider.clone())
        .cancelLoanRequisition(requisition_id, member_id)
        .from(from)
        .estimate_gas().await
        .map_err(BlockchainError::from_gas_estimate)?;
    Ok(U256::from(gas))
}

pub async fn estimate_create_loan_requisition_gas(
    provider:          &Provider,
    loan_machine_addr: Address,
    from:              Address,
    amount:            U256,
    parcels_count:     u32,
    member_id:         FixedBytes<32>,
    days_interval:     u32,
) -> Result<U256, BlockchainError> {
    let gas = LoanMachine::new(loan_machine_addr, provider.clone())
        .createLoanRequisition(amount, parcels_count, member_id, days_interval)
        .from(from)
        .estimate_gas().await
        .map_err(BlockchainError::from_gas_estimate)?;
    Ok(U256::from(gas))
}
