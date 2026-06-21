use alloy::primitives::{Address, B256, U256};
use std::str::FromStr;
use thiserror::Error;

use loan_machine_models::responses::{
    ApproveWalletPending, ApproveWalletProposalRow, RequestApprovalBundle,
};
use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::subgraph::SubgraphService;
use crate::server_logic::subgraph_queries::approve_wallet_proposals::fetch_pending_approve_wallet_proposals;

#[derive(Debug, Error)]
pub enum AdminApprovalError {
    #[error("invalid coop id: {0}")] InvalidCoopId(String),
    #[error("invalid target wallet address")] InvalidTargetWallet,
    #[error("not authorized: admin or moderator required")] NotAuthorized,
    #[error(transparent)] Blockchain(#[from] BlockchainError),
    #[error(transparent)] Subgraph(#[from] crate::services::subgraph::SubgraphError),
}

pub async fn list_pending_approvals_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<ApproveWalletPending, AdminApprovalError> {
    let (_coop_id_b32, lm_addr, _provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: AdminApprovalError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );
    let wallet_addr = wallet_address::to_alloy(&wallet);

    let admins    = contract.getAdmins().call().await.map_err(BlockchainError::from_call)?._0;
    let threshold = contract.adminThreshold().call().await.map_err(BlockchainError::from_call)?._0;

    let raw = fetch_pending_approve_wallet_proposals(subgraph, lm_addr, wallet_addr).await?;

    let proposals = raw.into_iter().map(|p| ApproveWalletProposalRow {
        proposal_id: p.proposal_id, proposer: p.proposer, created_at: p.created_at,
        confirmations: p.confirmations, viewer_confirmed: p.viewer_confirmed,
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
    let (_coop_id_b32, lm_addr, _provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: AdminApprovalError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );
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

pub async fn prepare_propose_wallet_as_admin_logic(
    blockchain:    &BlockchainService,
    coop_id_hex:   &str,
    target_wallet: &str,
    caller:        WalletAddress,
) -> Result<RequestApprovalBundle, AdminApprovalError> {
    let (_coop_id_b32, lm_addr, _provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: AdminApprovalError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );
    let caller_addr = wallet_address::to_alloy(&caller);
    let target_addr = Address::from_str(target_wallet)
        .map_err(|_| AdminApprovalError::InvalidTargetWallet)?;

    let is_admin = contract.isAdmin(caller_addr).call().await
        .map(|r| r._0).map_err(BlockchainError::from_call)?;
    if !is_admin {
        let mid = contract.getMemberId(caller_addr).call().await
            .map(|r| r._0).map_err(BlockchainError::from_call)?;
        let is_mod = mid != B256::ZERO && contract.isModerator(mid).call().await
            .map(|r| r._0).map_err(BlockchainError::from_call)?;
        if !is_mod {
            return Err(AdminApprovalError::NotAuthorized);
        }
    }

    let call = contract.proposeApproveWalletAsAdmin(target_addr);
    let gas  = call.clone().from(caller_addr).estimate_gas().await
        .map_err(BlockchainError::from_call)?;
    let data_bytes = call.calldata().clone();

    Ok(RequestApprovalBundle {
        to:      format!("{lm_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}