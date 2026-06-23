// src/models/responses.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApproveWalletProposalRow {
    pub proposal_id:      u64,
    pub proposer:         String,
    pub created_at:       u64,
    pub confirmations:    u32,
    pub viewer_confirmed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApproveWalletPending {
    pub threshold:    u32,
    pub total_admins: u32,
    pub proposals:    Vec<ApproveWalletProposalRow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfileCoop {
    pub id:            String,          // lowercased LoanMachine addr
    pub coop_id:       String,          // bytes32 hex
    pub name:          String,
    pub loan_machine:  String,
    pub active:        bool,
    pub member_id:     String,  // bytes32 hex if vinculated in this coop
    pub is_member:     bool,
    pub is_admin:      bool,
    pub is_moderator:  bool,
    pub is_approved:          bool,   // NEW: approved wallet (set even when vinculated)
    pub has_pending_approval: bool, 
    pub pending_count: u32,             // actionable items for *this* viewer
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    pub wallet: String,
    pub coops:  Vec<UserProfileCoop>,
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApprovalStatus {
    None,
    Pending {
        proposal_id:   u64,
        created_at:    u64,
        confirmations: u32,
        threshold:     u32,
        total_admins:  u32,
    },
    Approved,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RequestApprovalBundle {
    pub to:      String,
    pub data:    String,
    pub gas_hex: String,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserCoop {
    /// Lowercased LoanMachine address (matches the subgraph Cooperative.id).
    pub id: String,

    /// bytes32 hex from the registry (the on-chain coopId).  This is what
    /// you pass to server fns that take `coop_id: String`.
    #[serde(rename = "coopId")]
    pub coop_id: String,

    pub name: String,

    /// Same value as `id` but in mixed-case checksummed form when the
    /// subgraph stores it that way.  Use `id` for keys, this for display.
    #[serde(rename = "loanMachine")]
    pub loan_machine: String,

    pub active: bool,
}
 
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenElectionBundle {
    pub to:      String, // "0x..." LoanMachine address
    pub data:    String, // "0x..." ABI-encoded openElection(candidateId, opponentId)
    pub gas_hex: String, // "0x..." pinned gas limit
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteBundle  {
    pub to:      String, 
    pub data:    String, 
    pub gas_hex: String, 
}
 
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElectionView {
    pub id:               u32,
    /// Each entry is a bytes32 hex string, e.g. "0xab12...".
    pub candidates:       Vec<String>,
    pub start_time:       u64,
    pub end_time:         u64,
    pub is_active:        bool,
    /// "0x000...000" when election hasn't been decided.
    pub winner_id:        String,
    pub winning_votes:    i32,
    pub total_votes_cast: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoopViewerState {
    pub coop: CooperativeView,   // reuse what the list already returns
    pub role: ViewerRole,
    pub is_admin:     bool,
    pub is_moderator: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViewerRole {
    Visitor,             // wallet has no relation to this coop
    ApprovalPending,     // open admin proposal exists to approve this wallet
    Approved,            // wallet approved on-chain, not yet vinculated
    Member,              // vinculated member
    Moderator,           // member who's been elected moderator
    Admin,               // coop admin (may or may not also be a member)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CooperativeView {
    pub id: String,
    pub coop_id: String,
    pub name: String,
    pub loan_machine: String,
    pub active: bool,
    pub registered_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionResponse {
    pub to:           String,
    pub data:         String,
    pub value:        String,
    pub gas_estimate: String,
}

/// Info about a cooperative instance
#[derive(Serialize, Deserialize, Clone)]
pub struct CoopInfo {
    pub coop_id:      String,   // bytes32 as "0x..." hex string
    pub name:         String,
    pub loan_machine: String,   // contract address
    pub active:       bool,
}

/// Everything the browser needs to submit the vinculation transaction.
/// joinCoop now handles both joining AND vinculation in one tx,
/// so we only need one calldata instead of two.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VinculationBundle {
    pub join_calldata:        String,   // encoded joinCoop(memberId, wallet)
    pub loan_machine_address: String,   // target contract for the tx
    pub coop_registry_address:      String,   // coop_registry address (for reference/display)
    pub gas_join:             String,   // gas estimate as string
}

/// Donation transaction data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DonationBundle {
    pub approve_calldata:     String,   // USDT.approve(loanMachine, amount)
    pub usdt_address:         String,
    pub donate_calldata:      String,   // LoanMachine.donate(amount, memberId)
    pub loan_machine_address: String,
    pub gas_approve:          String,
    pub gas_donate:           String,
}

/// Withdrawal transaction data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WithdrawalBundle {
    pub withdraw_calldata:    String,   // LoanMachine.withdraw(amount, memberId)
    pub loan_machine_address: String,
    pub gas_withdraw:         String,
}

/// Loan requisition transaction data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoanRequisitionBundle {
    pub calldata:             String,   // LoanMachine.createLoanRequisition(...)
    pub loan_machine_address: String,
    pub gas_hex:              String,
}

/// A single loan requisition, enriched with on-chain state.
///
/// `status` is the raw `BorrowStatus` discriminant:
///   0=Pending, 1=PartiallyCovered, 2=FullyCovered, 3=Active,
///   4=Repaid, 5=Defaulted, 6=Cancelled
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoanRequisitionItem {
    pub requisition_id:   String,  // decimal string
    pub amount:           String,  // raw USDT units
    pub parcels_count:    u32,
    pub status:           u8,
    pub current_coverage: u32,     // 0–100 from getRequisitionInfo
    pub created_at:       u64,     // Unix seconds from contract creationTime
}

/// A single open-market requisition (status Pending or PartiallyCovered).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenMarketItem {
    pub requisition_id:   String,  // decimal string
    pub amount:           String,  // raw USDT units
    pub parcels_count:    u32,
    pub current_coverage: u32,     // 0–100 (on-chain authoritative)
    pub created_at:       u64,     // Unix seconds
    pub borrower:         String,  // 0x-prefixed wallet address
}

/// Payload for the OPEN MARKET tab.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenMarketResponse {
    pub items:             Vec<OpenMarketItem>,
    pub user_withdrawable: String,  // raw USDT units, decimal string
}

/// Bundle returned to the founder for the deploy + initialize flow.
/// The client signs `deploy_tx` first, gets the deployed address,
/// then signs `initialize_calldata` against that address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopDeployBundle {
    /// Deployment transaction. `to` is null for deployments —
    /// `data` is bytecode + ABI-encoded constructor args.
    pub deploy_data:        String,    // 0x-prefixed hex
    pub gas_deploy:         String,    // estimated gas 0x-prefixed hex
    /// Calldata for `initializeMultisig(admins, threshold, founderMemberId)`.
    /// The founder calls this AGAINST the address from the deploy receipt.
    pub initialize_data:    String,
    pub gas_initialize:     String,
    }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopRegistrationResult {
    pub coop_id_hex:          String,    // bytes32 returned by registry
    pub loan_machine_address: String,
    pub registration_tx_hash: String,
}

/// All per-member financial data for a single cooperative, as seen by that member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberFinancials {
    pub wallet:            String,   // display address
    pub member_id:         String,   // bytes32 hex
    pub reputation:        i32,
    /// Current donation balance (raw token units as decimal string).
    pub donation:          String,
    /// Outstanding borrowed amount.
    pub borrowing:         String,
    /// Unix timestamp of the last borrow; 0 if the member has never borrowed.
    pub last_borrow_time:  u64,
    /// Portion of donation locked in active loan coverage.
    pub in_coverage:       String,
    /// donation − in_coverage (free to withdraw).
    pub withdrawable:      String,
    /// USDT allowance granted to the LoanMachine contract.
    pub allowance:         String,
    pub loan_machine_address: String,
}

/// Cooperative-wide money flow and status, as seen on the coop control panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopFinancials {
    /// Lifetime sum of all donations (raw token units).
    pub total_donations:    String,
    /// Outstanding amount currently lent out (raw token units).
    pub total_borrowed:     String,
    /// Donations not currently locked in active loans (raw token units).
    pub available_balance:  String,
    /// USDT actually held by the LoanMachine contract (raw token units).
    pub contract_balance:   String,
    pub is_active:           bool,
    pub active_member_count: u32,
    pub average_reputation:  i32,
}