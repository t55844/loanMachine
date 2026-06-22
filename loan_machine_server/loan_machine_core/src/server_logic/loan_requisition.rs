use alloy::primitives::{Address, U256};
use thiserror::Error;

use loan_machine_models::responses::{LoanRequisitionBundle, LoanRequisitionItem};
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::loan_machine_fns_helpers::{
    encode_create_loan_requisition, estimate_create_loan_requisition_gas,
    encode_cancel_loan_requisition, estimate_cancel_loan_requisition_gas,
    get_requisition_info,
};
use crate::services::subgraph::{SubgraphError, SubgraphService};
use crate::server_logic::helpers::resolve_member_id;
use crate::server_logic::subgraph_queries::my_requisitions::fetch_my_requisitions;

#[derive(Debug, Error)]
pub enum LoanRequisitionError {
    #[error("invalid coop id: {0}")]                           InvalidCoopId(String),
    #[error("invalid amount: {0}")]                            InvalidAmount(String),
    #[error("payment interval must be between 1 and 30 days")] InvalidDaysInterval,
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
    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: LoanRequisitionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let wallet_hex = format!("{:#x}", to_alloy(wallet)).to_lowercase();
    let raw        = fetch_my_requisitions(subgraph, &coop_id, &wallet_hex).await?;

    // Enrich each item with current_coverage and creationTime from on-chain.
    // Users have at most 3 open requisitions so the call count is bounded.
    let mut items = Vec::with_capacity(raw.len());
    for r in raw {
        let req_id = r.requisition_id.parse::<U256>().unwrap_or(U256::ZERO);
        let (current_coverage, created_at) =
            get_requisition_info(provider.as_ref(), loan_machine_addr, req_id)
                .await
                .unwrap_or((0, 0));

        items.push(LoanRequisitionItem {
            requisition_id:   r.requisition_id,
            amount:           r.amount,
            parcels_count:    r.parcels_count,
            status:           r.status,
            current_coverage,
            created_at,
        });
    }
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
