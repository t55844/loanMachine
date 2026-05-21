// src/models/responses.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApprovalStatus {
    None,
    Pending {
        proposal_id:        u64,
        created_at:         u64,
        confirmations:      u32,
        threshold:          u32,
        total_admins:       u32,
        moderator_cosigned: bool,
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
    pub join_calldata:        String,   // encoded joinCoop(memberId, wallet, accessCode)
    pub loan_machine_address: String,   // target contract for the tx
    pub coop_registry_address:      String,   // coop_registry address (for reference/display)
    pub gas_join:             String,   // gas estimate as string
}

/// Donation transaction data
#[derive(Serialize, Deserialize, Clone)]
pub struct DonationBundle {
    pub approve_calldata:     String,   // USDT.approve(loanMachine, amount)
    pub usdt_address:         String,
    pub donate_calldata:      String,   // LoanMachine.donate(amount, memberId)
    pub loan_machine_address: String,
    pub gas_approve:          String,
    pub gas_donate:           String,
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
    /// Calldata for `initializeMultisig(admins, threshold, accessCode)`.
    /// The founder calls this AGAINST the address from the deploy receipt.
    pub initialize_data:    String,
    pub gas_initialize:     String,
    /// Server-generated access code. Shown to founder ONCE.
    /// They must save it before continuing — admins use it to approve members.
    pub access_code:        String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopRegistrationResult {
    pub coop_id_hex:          String,    // bytes32 returned by registry
    pub loan_machine_address: String,
    pub registration_tx_hash: String,
}