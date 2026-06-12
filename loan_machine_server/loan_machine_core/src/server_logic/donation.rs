use alloy::primitives::{B256, U256};
use thiserror::Error;

use loan_machine_models::responses::{
    DonationBundle,
};
use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError, BlockchainService};
use crate::server_logic::subgraph_queries::approve_wallet_proposals::fetch_pending_approve_wallet_proposals;

use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum AdminApprovalError {
    #[error("invalid coop id: {0}")] InvalidCoopId(String),
    #[error("viewer is not vinculated; cannot cosign")] NotVinculated,
    #[error(transparent)] Blockchain(#[from] BlockchainError),
    #[error(transparent)] Subgraph(#[from] crate::services::subgraph::SubgraphError),
}

pub async fn donate_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<DonationBundle, AdminApprovalError> {
    
}
