// loan_machine_core/src/server_logic/coop_approval.rs

use alloy::primitives::{Address, B256, U256};
use thiserror::Error;

use loan_machine_models::responses::{ApprovalStatus, RequestApprovalBundle};
use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError, BlockchainService};
use crate::services::subgraph::SubgraphService;
use crate::server_logic::subgraph_queries::pending_approval::fetch_pending_approval;

#[derive(Debug, Error)]
pub enum ApprovalError {
    #[error("invalid coop id: {0}")]
    InvalidCoopId(String),

    #[error(transparent)]
    Blockchain(#[from] BlockchainError),
}

/// What the panel needs to render the right state for a Visitor / ApprovalPending viewer.
pub async fn get_approval_status_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<ApprovalStatus, ApprovalError> {
    let coop_id_b32       = parse_coop(coop_id_hex)?;
    let loan_machine_addr = blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
    let provider          = &blockchain.coop_registry.provider;
    let contract          = LoanMachine::new(loan_machine_addr, provider.clone());
    let wallet_addr: Address = wallet_address::to_alloy(&wallet);

    // Already approved? Short-circuit.
    let approved = contract.isWalletApproved(wallet_addr).call().await
        .map_err(BlockchainError::from_call)?._0;
    if approved {
        return Ok(ApprovalStatus::Approved);
    }

    // Look for a pending proposal in the subgraph.
    let pending = fetch_pending_approval(subgraph, loan_machine_addr, wallet_addr)
        .await
        .map_err(BlockchainError::from)?;

    let Some(p) = pending else { return Ok(ApprovalStatus::None); };

    // Hydrate the live state from chain (subgraph only knows it was created).
    let info = contract.getProposal(U256::from(p.proposal_id)).call().await
        .map_err(BlockchainError::from_call)?;

    if info.executed {
        // Edge case: subgraph hasn't indexed the executed flag → wallet IS approved,
        // we just hit the isWalletApproved check before this state propagated.
        return Ok(ApprovalStatus::Approved);
    }

    let admins         = contract.getAdmins().call().await
        .map_err(BlockchainError::from_call)?._0;
    let threshold      = contract.adminThreshold().call().await
        .map_err(BlockchainError::from_call)?._0;

    Ok(ApprovalStatus::Pending {
        proposal_id:           p.proposal_id,
        created_at:            p.created_at,
        confirmations:         info.confirmations.try_into().unwrap_or(u32::MAX),
        threshold:             threshold.try_into().unwrap_or(u32::MAX),
        total_admins:          admins.len() as u32,
        moderator_cosigned:    info.moderatorCosignedBy != B256::ZERO,
    })
}

/// Builds the calldata + gas estimate for `proposeWalletApproval(wallet)`.
/// The wallet is the caller's own — we trust it from the JWT, not the request body.
pub async fn prepare_request_approval_logic(
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<RequestApprovalBundle, ApprovalError> {
    let coop_id_b32       = parse_coop(coop_id_hex)?;
    let loan_machine_addr = blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
    let provider          = &blockchain.coop_registry.provider;
    let contract          = LoanMachine::new(loan_machine_addr, provider.clone());
    let wallet_addr: Address = wallet_address::to_alloy(&wallet);

    let call = contract.proposeWalletApproval(wallet_addr);
    let gas  = call.clone().from(wallet_addr).estimate_gas().await
        .map_err(BlockchainError::from_call)?;
    let data_bytes = call.calldata().clone();

    Ok(RequestApprovalBundle {
        to:      format!("{loan_machine_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}

fn parse_coop(hex_str: &str) -> Result<B256, ApprovalError> {
    hex_str.parse().map_err(|_| ApprovalError::InvalidCoopId(hex_str.to_string()))
}