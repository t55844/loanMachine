// loan_machine_core/src/server_logic/coop_view.rs

use loan_machine_models::responses::{CooperativeView, CoopViewerState, ViewerRole};
use thiserror::Error;

use crate::services::subgraph::SubgraphService;
use crate::server_logic::subgraph_queries::cooperative_by_id::fetch_one_coop;

use alloy::primitives::{Address, B256};

use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainService};
use crate::server_logic::subgraph_queries::pending_approval::fetch_pending_approval;

// ── ERROR ─────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum CoopViewError {
    #[error("cooperative not found: {0}")]
    NotFound(String),

    #[error("subgraph error: {0}")]
    Subgraph(#[from] crate::services::subgraph::SubgraphError),

    #[error("subgraph string to number convertion failed. number: {0}")]
    StringToNumber(String),
}



pub async fn get_viewer_state_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<CoopViewerState, CoopViewError> {
    let row = fetch_one_coop(subgraph, coop_id_hex)
        .await?
        .ok_or_else(|| CoopViewError::NotFound(coop_id_hex.to_string()))?;

    let registered_at_u64 = row.registered_at.parse()
        .map_err(|_| CoopViewError::StringToNumber(row.registered_at.to_string()))?;

    let coop = CooperativeView {
        id:            row.id,
        coop_id:       row.coop_id,
        name:          row.name,
        loan_machine:  row.loan_machine,
        active:        row.active,
        registered_at: registered_at_u64,
    };

    let role = derive_role(&coop, &wallet, subgraph, blockchain).await;
    Ok(CoopViewerState { coop, role })
}

async fn derive_role(
    coop:       &CooperativeView,
    wallet:     &WalletAddress,
    subgraph:   &SubgraphService,
    blockchain: &BlockchainService,
) -> ViewerRole {
    // Parse the LM address once. If it's malformed (shouldn't happen, but
    // defensive), drop to Visitor so the user at least sees something.
    let Ok(lm_addr): Result<Address, _> = coop.loan_machine.parse() else {
        return ViewerRole::Visitor;
    };
    let provider     = &blockchain.coop_registry.provider;
    let contract     = LoanMachine::new(lm_addr, provider.clone());
    let wallet_addr  = wallet_address::to_alloy(wallet);

    // 1. Admin?  Errors here collapse to Visitor (safe default — the user
    //    will just see the request-approval form, no privilege leaked).
    if let Ok(r) = contract.isAdmin(wallet_addr).call().await {
        if r._0 { return ViewerRole::Admin; }
    }

    // 2. Vinculated?  memberId of bytes32(0) = no.
    let member_id = contract.getMemberId(wallet_addr).call().await
        .map(|r| r._0)
        .unwrap_or(B256::ZERO);

    if member_id != B256::ZERO {
        if let Ok(r) = contract.isModerator(member_id).call().await {
            if r._0 { return ViewerRole::Moderator; }
        }
        return ViewerRole::Member;
    }

    // 3. Approved wallet but not yet vinculated → ready for first vinculation.
    if let Ok(r) = contract.isWalletApproved(wallet_addr).call().await {
        if r._0 { return ViewerRole::Approved; }
    }

    // 4. Has an open approval proposal in flight?
    if let Ok(Some(_)) = fetch_pending_approval(subgraph, lm_addr, wallet_addr).await {
        return ViewerRole::ApprovalPending;
    }

    // 5. None of the above.
    ViewerRole::Visitor
}