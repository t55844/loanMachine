use alloy::primitives::{Address, U256};
use thiserror::Error;

use loan_machine_models::responses::{
    LoanRequisitionBundle, LoanRequisitionItem, OpenMarketItem, OpenMarketResponse,
};
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::loan_machine_fns_helpers::{
    encode_create_loan_requisition, estimate_create_loan_requisition_gas,
    encode_cancel_loan_requisition, estimate_cancel_loan_requisition_gas,
    encode_cover_loan, estimate_cover_loan_gas,
    get_requisition_info, get_user_withdrawable,
};
use crate::services::subgraph::{SubgraphError, SubgraphService};
use crate::server_logic::helpers::resolve_member_id;
use crate::server_logic::subgraph_queries::my_requisitions::fetch_my_requisitions;
use crate::server_logic::subgraph_queries::open_market::fetch_open_requisitions;

#[derive(Debug, Error)]
pub enum LoanRequisitionError {
    #[error("invalid coop id: {0}")]                           InvalidCoopId(String),
    #[error("invalid amount: {0}")]                            InvalidAmount(String),
    #[error("payment interval must be between 1 and 30 days")] InvalidDaysInterval,
    #[error("number of installments must be between 1 and 12")] InvalidParcelsCount,
    #[error("coverage must be between 1 and 100")]             InvalidCoverage,
    #[error("coverage would exceed 100% — the loan may have been partially covered since the page loaded; refresh and try again")] CoverageExceeds,
    #[error("this loan is no longer accepting coverage")]       LoanNotAvailable,
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
