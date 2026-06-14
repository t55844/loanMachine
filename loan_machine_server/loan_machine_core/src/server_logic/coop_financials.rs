use thiserror::Error;

use loan_machine_models::responses::CoopFinancials;

use crate::services::blockchain::{BlockchainError, BlockchainService};

#[derive(Debug, Error)]
pub enum CoopFinancialsError {
    #[error("invalid coop id: {0}")]
    InvalidCoopId(String),

    #[error(transparent)]
    Blockchain(#[from] BlockchainError),
}

pub async fn get_coop_financials_logic(
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
) -> Result<CoopFinancials, CoopFinancialsError> {
    let (_coop_id_b32, _loan_machine_addr, _provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: CoopFinancialsError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let stats = contract.getCoopStats().call().await
        .map_err(BlockchainError::from_call)?;

    Ok(CoopFinancials {
        total_donations:     stats.totalDonations.to_string(),
        total_borrowed:      stats.totalBorrowed.to_string(),
        available_balance:   stats.availableBalance.to_string(),
        contract_balance:    stats.contractBalance.to_string(),
        is_active:           stats.isActive,
        active_member_count: stats.activeMemberCount,
        average_reputation:  stats.averageReputation,
    })
}
