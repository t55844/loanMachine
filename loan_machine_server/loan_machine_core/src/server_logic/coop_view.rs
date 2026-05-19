// loan_machine_core/src/server_logic/coop_view.rs
//
// Single responsibility: given a coopId + a wallet address,
// return what that wallet can see and do in that cooperative.
//
// Two data sources, two concerns:
//
//   1. Subgraph  → CooperativeView (the coop's own data)
//   2. Chain     → ViewerRole      (this wallet's relationship to the contract)
//
// The split is intentional. The subgraph is fast and indexed; the chain
// is the source of truth for per-wallet state that changes with each tx.
// Neither leaks into the other.
//
// ── PHASE TRACKING ────────────────────────────────────────────────────────
//
//   Phase 1 (now):  derive_role always returns Visitor.
//                   The panel renders; subgraph data is live.
//
//   Phase 2 (next): derive_role reads the LoanMachine contract at
//                   coop.loan_machine and checks wallet state.
//                   Only this function changes — everything above is stable.

use loan_machine_models::responses::{CooperativeView, CoopViewerState, ViewerRole};
use loan_machine_models::wallet_address::WalletAddress;
use thiserror::Error;

use crate::services::subgraph::SubgraphService;
use crate::server_logic::subgraph_queries::cooperative_by_id::fetch_one_coop;

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

// ── PUBLIC ENTRY POINT ────────────────────────────────────────────────────

/// Builds the viewer state the control panel renders.
///
/// Called by the `get_coop_viewer_state` server fn.
/// The `blockchain_service` parameter is unused in Phase 1 — it's present
/// here already so the server fn signature never needs to change when
/// Phase 2 plugs in.
pub async fn get_viewer_state_logic(
    subgraph: &SubgraphService,
    coop_id:  &str,
    wallet:   WalletAddress,
) -> Result<CoopViewerState, CoopViewError> {
    // Step 1 — coop data from the index.
    let row = fetch_one_coop(subgraph, coop_id)
        .await?
        .ok_or_else(|| CoopViewError::NotFound(coop_id.to_string()))?;

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

    // Step 2 — derive role from chain.
    // Phase 1: always Visitor.
    // Phase 2: pass blockchain_service + coop.loan_machine here.
    let role = derive_role(&coop, wallet).await;

    Ok(CoopViewerState { coop, role })
}


// ── ROLE DERIVATION ───────────────────────────────────────────────────────
//
// Phase 1: stub.
//
// Phase 2 implementation (what goes here):
//
//   let address: Address = coop.loan_machine.parse()?;
//   let contract = LoanMachine::new(address, blockchain_service.provider());
//   let wallet_alloy = wallet_address::to_alloy(&wallet);
//
//   // First match wins — ordered from most to least privileged.
//   if contract.isAdmin(wallet_alloy).call().await?.is_admin { return Admin }
//
//   let member_id = contract.getMemberId(wallet_alloy).call().await?._0;
//   if member_id != B256::ZERO {
//       if contract.isModerator(member_id).call().await?._0 { return Moderator }
//       return Member;
//   }
//
//   if contract.isWalletApproved(wallet_alloy).call().await?._0 { return Approved }
//
//   // Check open admin proposals for this wallet → ApprovalPending.
//   // (proposal scan logic goes here)
//
//   Visitor

async fn derive_role(_coop: &CooperativeView, _wallet: WalletAddress) -> ViewerRole {
    ViewerRole::Visitor
}