// loan_machine_core/src/server_logic/coop_view.rs

use loan_machine_models::responses::{CooperativeView, CoopViewerState, ViewerRole};
use thiserror::Error;

use crate::services::subgraph::SubgraphService;
use crate::server_logic::subgraph_queries::cooperative_by_id::fetch_one_coop;

use alloy::primitives::{Address, B256};

use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainService};
use crate::server_logic::subgraph_queries::pending_approval::fetch_pending_approval;

use crate::server_logic::helpers::resolve_member_id;
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
        .map_err(|_| CoopViewError::StringToNumber(row.registered_at.clone()))?;

    let coop = CooperativeView {
        id: row.id, coop_id: row.coop_id, name: row.name,
        loan_machine: row.loan_machine, active: row.active,
        registered_at: registered_at_u64,
    };

    let (role, is_admin, is_moderator) =
        derive_role_and_flags(&coop, &wallet, subgraph, blockchain).await;

    Ok(CoopViewerState { coop, role, is_admin, is_moderator })
}

async fn derive_role_and_flags(
    coop:       &CooperativeView,
    wallet:     &WalletAddress,
    subgraph:   &SubgraphService,
    blockchain: &BlockchainService,
) -> (ViewerRole, bool, bool) {
    let Ok(lm_addr): Result<Address, _> = coop.loan_machine.parse() else {
        return (ViewerRole::Visitor, false, false);
    };
    let provider    = &blockchain.coop_registry.provider;
    let contract    = LoanMachine::new(lm_addr, provider.clone());
    let wallet_addr = wallet_address::to_alloy(wallet);

    // Independent capability reads.
    let is_admin = contract.isAdmin(wallet_addr).call().await
        .map(|r| r._0).unwrap_or(false);

     let member_id_b32 = resolve_member_id(subgraph, provider, lm_addr, wallet)
        .await
        .ok()           // Result<Option<B256>, _> → Option<Option<B256>>
        .flatten()      // → Option<B256>
        .unwrap_or(B256::ZERO);

    let is_member = member_id_b32 != B256::ZERO;

    let is_moderator = if is_member {
        contract.isModerator(member_id_b32).call().await
            .map(|r| r._0).unwrap_or(false)
    } else { false };

    // Membership ladder: ignores admin/moderator status; those are flags.
    let role = if is_member {
        ViewerRole::Member
    } else {
        let approved = contract.isWalletApproved(wallet_addr).call().await
            .map(|r| r._0).unwrap_or(false);
        if approved {
            ViewerRole::Approved
        } else if let Ok(Some(_)) = fetch_pending_approval(subgraph, lm_addr, wallet_addr).await {
            ViewerRole::ApprovalPending
        } else {
            ViewerRole::Visitor
        }
    };

    (role, is_admin, is_moderator)
}