use alloy::primitives::{B256, U256};
use thiserror::Error;

use loan_machine_models::responses::{
    ApproveWalletPending, ApproveWalletProposalRow, RequestApprovalBundle,
};
use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError, BlockchainService};
use crate::services::subgraph::SubgraphService;
use crate::server_logic::subgraph_queries::approve_wallet_proposals::fetch_pending_approve_wallet_proposals;

use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum AdminApprovalError {
    #[error("invalid coop id: {0}")] InvalidCoopId(String),
    #[error("viewer is not vinculated; cannot cosign")] NotVinculated,
    #[error(transparent)] Blockchain(#[from] BlockchainError),
    #[error(transparent)] Subgraph(#[from] crate::services::subgraph::SubgraphError),
}

pub async fn list_pending_approvals_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<ApproveWalletPending, AdminApprovalError> {
    let coop_id_b32 = coop_id_hex.parse::<B256>()
        .map_err(|_| AdminApprovalError::InvalidCoopId(coop_id_hex.into()))?;
    let lm_addr  = blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
    let provider = &blockchain.coop_registry.provider;
    let contract = LoanMachine::new(lm_addr, provider.clone());
    let wallet_addr = wallet_address::to_alloy(&wallet);

    let admins    = contract.getAdmins().call().await.map_err(BlockchainError::from_call)?._0;
    let threshold = contract.adminThreshold().call().await.map_err(BlockchainError::from_call)?._0;

    let raw = fetch_pending_approve_wallet_proposals(subgraph, lm_addr, wallet_addr).await?;

    let proposals = raw.into_iter().map(|p| ApproveWalletProposalRow {
        proposal_id: p.proposal_id, proposer: p.proposer, created_at: p.created_at,
        confirmations: p.confirmations, moderator_cosigned: p.moderator_cosigned,
        viewer_confirmed: p.viewer_confirmed,
    }).collect();

    Ok(ApproveWalletPending {
        threshold:    threshold.try_into().unwrap_or(u32::MAX),
        total_admins: admins.len() as u32,
        proposals,
    })
}

pub async fn prepare_confirm_proposal_logic(
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    proposal_id: u64,
    wallet:      WalletAddress,
) -> Result<RequestApprovalBundle, AdminApprovalError> {
    let coop_id_b32 = coop_id_hex.parse::<B256>()
        .map_err(|_| AdminApprovalError::InvalidCoopId(coop_id_hex.into()))?;
    let lm_addr  = blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
    let provider = &blockchain.coop_registry.provider;
    let contract = LoanMachine::new(lm_addr, provider.clone());
    let wallet_addr = wallet_address::to_alloy(&wallet);

    let call = contract.confirmProposal(U256::from(proposal_id));
    let gas  = call.clone().from(wallet_addr).estimate_gas().await
        .map_err(BlockchainError::from_call)?;
    let data_bytes = call.calldata().clone();

    Ok(RequestApprovalBundle {
        to:      format!("{lm_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}

pub async fn prepare_cosign_proposal_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    proposal_id: u64,
    wallet:      WalletAddress,
) -> Result<RequestApprovalBundle, AdminApprovalError> {
    let coop_id_b32 = coop_id_hex.parse::<B256>()
        .map_err(|_| AdminApprovalError::InvalidCoopId(coop_id_hex.into()))?;
    let lm_addr  = blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
    let provider = &blockchain.coop_registry.provider;
    let contract = LoanMachine::new(lm_addr, provider.clone());
    let wallet_addr = wallet_address::to_alloy(&wallet);

    let member_id_b32 = resolve_member_id( 
            subgraph, provider, lm_addr, &wallet
            ).await?
            .unwrap_or(B256::ZERO);
    let is_member = member_id_b32 != B256::ZERO;
    
    if !is_member {
        return Err(AdminApprovalError::NotVinculated);
    }

    let call = contract.cosignProposal(U256::from(proposal_id), member_id_b32);
    let gas  = call.clone().from(wallet_addr).estimate_gas().await
        .map_err(BlockchainError::from_call)?;
    let data_bytes = call.calldata().clone();

    Ok(RequestApprovalBundle {
        to:      format!("{lm_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}