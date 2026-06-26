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
        .map_err(BlockchainError::from_call)?
        ._0;
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

pub fn encode_cover_loan(
    requisition_id:    U256,
    coverage_pct:      u32,
    member_id:         FixedBytes<32>,
) -> Bytes {
    Bytes::from(LoanMachine::coverLoanCall {
        requisitionId:      requisition_id,
        coveragePercentage: coverage_pct,
        memberId:           member_id,
    }.abi_encode())
}

pub async fn estimate_cover_loan_gas(
    provider:          &Provider,
    loan_machine_addr: Address,
    from:              Address,
    requisition_id:    U256,
    coverage_pct:      u32,
    member_id:         FixedBytes<32>,
) -> Result<U256, BlockchainError> {
    let gas = LoanMachine::new(loan_machine_addr, provider.clone())
        .coverLoan(requisition_id, coverage_pct, member_id)
        .from(from)
        .estimate_gas().await
        .map_err(BlockchainError::from_gas_estimate)?;
    Ok(U256::from(gas))
}

/// Returns the user's withdrawable donation balance (donations – donationsInCoverage).
pub async fn get_user_withdrawable(
    provider:          &Provider,
    loan_machine_addr: Address,
    user:              Address,
) -> Result<U256, BlockchainError> {
    let f = LoanMachine::new(loan_machine_addr, provider.clone())
        .getUserFinancials(user)
        .call().await
        .map_err(BlockchainError::from_call)?;
    Ok(f.withdrawable)
}

pub fn encode_repay(
    requisition_id: U256,
    amount:         U256,
    member_id:      FixedBytes<32>,
) -> Bytes {
    Bytes::from(LoanMachine::repayCall {
        requisitionId: requisition_id,
        amount,
        memberId: member_id,
    }.abi_encode())
}

/// Returns all active (`status == 3`) loans for `borrower` in one RPC call.
/// Each entry is `(requisition_id, status, parcels_count, parcels_pending, payment_dates, parcel_amounts, creation_time)`.
/// Returns all active loans for `borrower` via `getActiveLoans` (one RPC call).
/// The contract filters for `status == ContractStatus.Active && parcelsPending > 0`.
/// Each entry: `(req_id, parcels_count, parcels_pending, parcels_values, payment_dates, parcel_amounts, created_at)`.
pub async fn get_active_loans(
    provider:          &Provider,
    loan_machine_addr: Address,
    borrower:          Address,
) -> Result<Vec<(U256, u32, u32, U256, Vec<u64>, Vec<String>, u64)>, BlockchainError> {
    let r = LoanMachine::new(loan_machine_addr, provider.clone())
        .getActiveLoans(borrower)
        .call().await
        .map_err(BlockchainError::from_call)?;

    Ok(r.activeLoans.into_iter().zip(r.ids).map(|(lc, req_id)| {
        let payment_dates  = lc.paymentDates.iter().map(|d| d.saturating_to::<u64>()).collect();
        let parcel_amounts = lc.parcelsAmounts.iter().map(|a| a.to_string()).collect();
        (req_id, lc.parcelsCount, lc.parcelsPending, lc.parcelsValues, payment_dates, parcel_amounts, lc.creationTime.saturating_to::<u64>())
    }).collect())
}

/// Returns `(status, parcels_count, parcels_pending, payment_dates, parcel_amounts, creation_time)`.
pub async fn get_loan_contract_data(
    provider:          &Provider,
    loan_machine_addr: Address,
    requisition_id:    U256,
) -> Result<(u8, u32, u32, Vec<u64>, Vec<String>, u64), BlockchainError> {
    let lc = LoanMachine::new(loan_machine_addr, provider.clone())
        .getLoanContract(requisition_id)
        .call().await
        .map_err(BlockchainError::from_call)?
        ._0;

    let payment_dates   = lc.paymentDates.iter().map(|d| d.saturating_to::<u64>()).collect();
    let parcel_amounts  = lc.parcelsAmounts.iter().map(|a| a.to_string()).collect();

    Ok((
        lc.status,
        lc.parcelsCount,
        lc.parcelsPending,
        payment_dates,
        parcel_amounts,
        lc.creationTime.saturating_to::<u64>(),
    ))
}

/// Returns `(next_payment_amount, can_pay)` from the contract.
pub async fn get_next_payment(
    provider:          &Provider,
    loan_machine_addr: Address,
    requisition_id:    U256,
) -> Result<(U256, bool), BlockchainError> {
    let r = LoanMachine::new(loan_machine_addr, provider.clone())
        .getNextPaymentAmount(requisition_id)
        .call().await
        .map_err(BlockchainError::from_call)?;
    Ok((r.paymentAmount, r.canPay))
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
