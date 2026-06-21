use alloy::primitives::{Address, U256};
use thiserror::Error;

use loan_machine_models::responses::LoanRequisitionBundle;
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::loan_machine_fns_helpers::{
    encode_create_loan_requisition, estimate_create_loan_requisition_gas,
};
use crate::services::subgraph::SubgraphService;
use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum LoanRequisitionError {
    #[error("invalid coop id: {0}")] InvalidCoopId(String),
    #[error("invalid amount: {0}")]  InvalidAmount(String),
    #[error("wallet is not a member of this cooperative")] WalletNotMember,
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
