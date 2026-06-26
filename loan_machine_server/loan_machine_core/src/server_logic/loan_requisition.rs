use alloy::primitives::{Address, U256};
use thiserror::Error;

use loan_machine_models::responses::{
    ActiveLoanItem, LoanParcel, LoanRequisitionBundle, LoanRequisitionItem,
    OpenMarketItem, OpenMarketResponse, RepaymentBundle,
};
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::loan_machine_fns_helpers::{
    encode_approve, estimate_approve_gas, get_erc20_allowance,
    encode_create_loan_requisition, estimate_create_loan_requisition_gas,
    encode_cancel_loan_requisition, estimate_cancel_loan_requisition_gas,
    encode_cover_loan, estimate_cover_loan_gas,
    encode_repay, get_active_loans, get_next_payment,
    get_requisition_info, get_user_withdrawable,
};
use crate::services::subgraph::{SubgraphError, SubgraphService};
use crate::server_logic::helpers::resolve_member_id;
use crate::server_logic::subgraph_queries::my_requisitions::fetch_my_requisitions;
use crate::server_logic::subgraph_queries::open_market::fetch_open_requisitions;

const GAS_REPAY_FALLBACK: u64 = 200_000;

#[derive(Debug, Error)]
pub enum LoanRequisitionError {
    #[error("invalid coop id: {0}")]                           InvalidCoopId(String),
    #[error("invalid amount: {0}")]                            InvalidAmount(String),
    #[error("payment interval must be between 1 and 30 days")] InvalidDaysInterval,
    #[error("number of installments must be between 1 and 12")] InvalidParcelsCount,
    #[error("coverage must be between 1 and 100")]             InvalidCoverage,
    #[error("coverage would exceed 100% — the loan may have been partially covered since the page loaded; refresh and try again")] CoverageExceeds,
    #[error("this loan is no longer accepting coverage")]       LoanNotAvailable,
    #[error("no payment is due on this loan right now")]        PaymentNotDue,
    #[error("wallet is not a member of this cooperative")]      WalletNotMember,
    #[error("subgraph error: {0}")]                            Subgraph(#[from] SubgraphError),
    #[error(transparent)] Blockchain(#[from] BlockchainError),
}

pub async fn create_loan_requisition_logic(
    subgraph:      &SubgraphService,
    blockchain:    &BlockchainService,
    coop_id_hex:   &str,
    wallet:        WalletAddress,
    amount:        String,
    parcels_count: u32,
    days_interval: u32,
) -> Result<LoanRequisitionBundle, LoanRequisitionError> {
    let amount: U256 = amount.parse()
        .map_err(|_| LoanRequisitionError::InvalidAmount(amount))?;

    if amount.is_zero() {
        return Err(LoanRequisitionError::InvalidAmount(amount.to_string()));
    }
    if parcels_count < 1 || parcels_count > 12 {
        return Err(LoanRequisitionError::InvalidParcelsCount);
    }
    if days_interval == 0 || days_interval > 30 {
        return Err(LoanRequisitionError::InvalidDaysInterval);
    }

    let wallet_addr: Address = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(LoanRequisitionError::WalletNotMember)?;

    let calldata = encode_create_loan_requisition(amount, parcels_count, member_id, days_interval);
    let gas = estimate_create_loan_requisition_gas(
        provider.as_ref(), loan_machine_addr, wallet_addr, amount, parcels_count, member_id, days_interval,
    ).await?;

    Ok(LoanRequisitionBundle {
        calldata:             format!("0x{}", hex::encode(calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        gas_hex:              format!("0x{:x}", gas),
    })
}

pub async fn fetch_my_requisitions_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      &WalletAddress,
) -> Result<Vec<LoanRequisitionItem>, LoanRequisitionError> {
    let (_coop_id_b32, loan_machine_addr, _provider) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
    );

    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let wallet_hex = format!("{:#x}", to_alloy(wallet)).to_lowercase();
    let raw        = fetch_my_requisitions(subgraph, &coop_id, &wallet_hex).await?;

    // The subgraph maintains status, currentCoverage, and creationTimestamp through
    // all lifecycle events — no chain calls needed here.
    let items = raw.into_iter().map(|r| {
        let created_at = r.creation_timestamp.parse::<u64>().unwrap_or(0);
        LoanRequisitionItem {
            requisition_id:  r.requisition_id,
            amount:          r.amount,
            parcels_count:   r.parcels_count,
            status:          r.status,
            current_coverage: r.current_coverage,
            created_at,
        }
    }).collect();

    Ok(items)
}

pub async fn prepare_cancel_requisition_logic(
    subgraph:       &SubgraphService,
    blockchain:     &BlockchainService,
    coop_id_hex:    &str,
    wallet:         WalletAddress,
    requisition_id: &str,
) -> Result<LoanRequisitionBundle, LoanRequisitionError> {
    let req_id: U256 = requisition_id.parse()
        .map_err(|_| LoanRequisitionError::InvalidCoopId("invalid requisition id".into()))?;

    let wallet_addr = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(LoanRequisitionError::WalletNotMember)?;

    let calldata = encode_cancel_loan_requisition(req_id, member_id);
    let gas = estimate_cancel_loan_requisition_gas(
        provider.as_ref(), loan_machine_addr, wallet_addr, req_id, member_id,
    ).await?;

    Ok(LoanRequisitionBundle {
        calldata:             format!("0x{}", hex::encode(calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        gas_hex:              format!("0x{:x}", gas),
    })
}

pub async fn fetch_open_market_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      &WalletAddress,
) -> Result<OpenMarketResponse, LoanRequisitionError> {
    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let coop_id = format!("{loan_machine_addr:#x}").to_lowercase();
    let raw     = fetch_open_requisitions(subgraph, &coop_id).await?;

    tracing::info!(
        coop_id    = %coop_id,
        item_count = raw.len(),
        "[open_market] subgraph returned {} item(s)",
        raw.len(),
    );

    // The subgraph filter (status_in: [0,1]) already excludes inactive loans.
    // currentCoverage and creationTimestamp come from the subgraph — no chain calls.
    let items: Vec<OpenMarketItem> = raw.into_iter().map(|r| {
        let created_at = r.creation_timestamp.parse::<u64>().unwrap_or(0);
        OpenMarketItem {
            requisition_id:  r.requisition_id,
            amount:          r.amount,
            parcels_count:   r.parcels_count,
            current_coverage: r.current_coverage,
            created_at,
            borrower:        r.borrower,
        }
    }).collect();

    let wallet_addr = to_alloy(wallet);
    match get_user_withdrawable(provider.as_ref(), loan_machine_addr, wallet_addr).await {
        Err(e) => {
            tracing::warn!(
                wallet = %wallet_addr,
                error  = %e,
                "[open_market] get_user_withdrawable failed — defaulting to 0",
            );
            Ok(OpenMarketResponse { items, user_withdrawable: "0".into() })
        }
        Ok(user_withdrawable) => {
            tracing::info!(
                wallet            = %wallet_addr,
                user_withdrawable = %user_withdrawable,
                "[open_market] user withdrawable balance",
            );
            Ok(OpenMarketResponse {
                items,
                user_withdrawable: user_withdrawable.to_string(),
            })
        }
    }
}

pub async fn prepare_cover_loan_logic(
    subgraph:       &SubgraphService,
    blockchain:     &BlockchainService,
    coop_id_hex:    &str,
    wallet:         WalletAddress,
    requisition_id: &str,
    coverage_pct:   u32,
) -> Result<LoanRequisitionBundle, LoanRequisitionError> {
    if coverage_pct == 0 || coverage_pct > 100 {
        return Err(LoanRequisitionError::InvalidCoverage);
    }

    let req_id: U256 = requisition_id.parse()
        .map_err(|_| LoanRequisitionError::InvalidCoopId("invalid requisition id".into()))?;

    let wallet_addr = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(LoanRequisitionError::WalletNotMember)?;

    // Read authoritative on-chain state before gas estimation.
    // The UI may show stale coverage (e.g. another lender just covered 97%),
    // which would make the gas estimate revert with LoanMachine_OverCoverage.
    // Catching it here gives a clear user-facing message instead.
    let (current_coverage, _, status) =
        get_requisition_info(provider.as_ref(), loan_machine_addr, req_id).await?;

    if status != 0 && status != 1 {
        return Err(LoanRequisitionError::LoanNotAvailable);
    }
    if current_coverage.saturating_add(coverage_pct) > 100 {
        return Err(LoanRequisitionError::CoverageExceeds);
    }

    let calldata = encode_cover_loan(req_id, coverage_pct, member_id);
    let gas = estimate_cover_loan_gas(
        provider.as_ref(), loan_machine_addr, wallet_addr, req_id, coverage_pct, member_id,
    ).await?;

    Ok(LoanRequisitionBundle {
        calldata:             format!("0x{}", hex::encode(calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        gas_hex:              format!("0x{:x}", gas),
    })
}

pub async fn fetch_my_active_loans_logic(
    _subgraph:   &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      &WalletAddress,
) -> Result<Vec<ActiveLoanItem>, LoanRequisitionError> {
    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let wallet_addr = to_alloy(wallet);

    // Single call: contract filters for Active status and parcelsPending > 0.
    let loans = get_active_loans(provider.as_ref(), loan_machine_addr, wallet_addr).await?;

    if loans.is_empty() {
        return Ok(vec![]);
    }

    // Fan out getNextPaymentAmount for all active loans concurrently.
    let payment_futs: Vec<_> = loans.iter().map(|(req_id, ..)| {
        let req_id   = *req_id;
        let provider = provider.clone();
        async move { get_next_payment(provider.as_ref(), loan_machine_addr, req_id).await }
    }).collect();

    let payments = futures::future::join_all(payment_futs).await;

    let items = loans.into_iter().zip(payments)
        .map(|((req_id, parcels_count, parcels_pending, parcels_values, payment_dates, parcel_amounts, created_at), payment_result)| {
            let (next_payment_amount, can_pay) = payment_result.unwrap_or_else(|e| {
                tracing::warn!(requisition_id = %req_id, error = %e, "getNextPaymentAmount failed");
                (U256::ZERO, false)
            });

            let paid_count = parcels_count.saturating_sub(parcels_pending) as usize;
            let parcels = payment_dates.iter().zip(parcel_amounts.iter()).enumerate()
                .map(|(i, (date, amount))| LoanParcel {
                    index:    i as u32,
                    due_date: *date,
                    amount:   amount.clone(),
                    is_paid:  i < paid_count,
                })
                .collect();

            // parcelsValues is the per-parcel amount; total = parcelsValues * parcelsCount.
            let total_amount = (parcels_values * U256::from(parcels_count)).to_string();

            ActiveLoanItem {
                requisition_id: req_id.to_string(),
                total_amount,
                parcels_count,
                parcels_pending,
                next_payment_amount: next_payment_amount.to_string(),
                can_pay,
                created_at,
                parcels,
            }
        })
        .collect();

    Ok(items)
}

pub async fn prepare_repayment_logic(
    subgraph:       &SubgraphService,
    blockchain:     &BlockchainService,
    coop_id_hex:    &str,
    usdc_address:   &str,
    wallet:         WalletAddress,
    requisition_id: &str,
) -> Result<RepaymentBundle, LoanRequisitionError> {
    let req_id: U256 = requisition_id.parse()
        .map_err(|_| LoanRequisitionError::InvalidCoopId("invalid requisition id".into()))?;

    let wallet_addr = to_alloy(&wallet);

    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let (payment_amount, can_pay) = get_next_payment(provider.as_ref(), loan_machine_addr, req_id).await?;
    if !can_pay {
        return Err(LoanRequisitionError::PaymentNotDue);
    }

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or(LoanRequisitionError::WalletNotMember)?;

    let usdc_addr: Address = usdc_address.parse()
        .map_err(|_| LoanRequisitionError::Blockchain(BlockchainError::InvalidAddress))?;

    let allowance = get_erc20_allowance(provider.as_ref(), usdc_addr, wallet_addr, loan_machine_addr).await?;
    let (approve_calldata, gas_approve) = if allowance >= payment_amount {
        (alloy::primitives::Bytes::new(), U256::ZERO)
    } else {
        let cd  = encode_approve(loan_machine_addr, payment_amount);
        let gas = estimate_approve_gas(provider.as_ref(), usdc_addr, loan_machine_addr, payment_amount, wallet_addr).await?;
        (cd, gas)
    };

    let repay_calldata = encode_repay(req_id, payment_amount, member_id);

    Ok(RepaymentBundle {
        approve_calldata:     format!("0x{}", hex::encode(approve_calldata.as_ref())),
        usdt_address:         usdc_addr.to_string(),
        repay_calldata:       format!("0x{}", hex::encode(repay_calldata.as_ref())),
        loan_machine_address: loan_machine_addr.to_string(),
        gas_approve:          format!("0x{:x}", gas_approve),
        gas_repay:            format!("0x{GAS_REPAY_FALLBACK:x}"),
        amount:               payment_amount.to_string(),
    })
}
