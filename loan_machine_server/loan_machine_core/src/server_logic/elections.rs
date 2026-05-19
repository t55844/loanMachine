// loan_machine_core/src/server_logic/elections.rs
//
// Election action bundles + reads, mirroring the shape of coop_approval.rs.
//
// Two entry points so far:
//   prepare_open_election_logic — builds the calldata + gas estimate for
//                                 LoanMachine.openElection(candidate, opponent).
//                                 Inputs are WALLET ADDRESSES; the server
//                                 resolves each to its bytes32 memberId via
//                                 LoanMachine.getMemberId(wallet).
//   get_current_election_logic  — chain read.  Returns Some(view) when an
//                                 election is active, None when not.
//
// Out of scope this iteration (next functions in the same file):
//   prepare_vote_logic, prepare_add_candidate_logic, prepare_close_election_logic,
//   and the "last closed election" read (which belongs in subgraph_queries/elections.rs
//   because the on-chain API doesn't expose it).

use alloy::primitives::{Address, B256};
use thiserror::Error;

use loan_machine_models::responses::{ElectionView, OpenElectionBundle};
use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError, BlockchainService};
use crate::services::subgraph::{ SubgraphService};
use crate::server_logic::helpers::resolve_member_id;

#[derive(Debug, Error)]
pub enum ElectionError {
    #[error("cooperative not found in registry: {0}")]
    CoopNotFound(String),

    #[error("invalid coop id: {0}")]
    InvalidCoopId(String),

    #[error("an election is already active")]
    ActiveElectionExists,

    #[error("candidate and opponent must be different wallets")]
    SameCandidates,

    #[error(transparent)]
    Blockchain(#[from] BlockchainError),
}

// ─────────────────────────────────────────────────────────────
// READ: current election (if active)
// ─────────────────────────────────────────────────────────────

/// Returns the currently-active election, or None if no election is active.
/// Distinguishing "no active" from "never had one" is a job for the subgraph;
/// this read alone can't tell them apart (see file header).
pub async fn get_current_election_logic(
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
) -> Result<Option<ElectionView>, ElectionError> {
    let coop_id_b32        = parse_bytes32_coop(coop_id_hex)?;
    let coop_registry = &blockchain.coop_registry;
    let loan_machine_addr = coop_registry.get_loan_machine(coop_id_b32).await?;

    let coop_registry = &blockchain.coop_registry;

    let provider = &coop_registry.provider;
    let contract = LoanMachine::new(loan_machine_addr, provider.clone());

    // getCurrentElectionId returns -1 when there is no active election.
    let current_id: i32 = contract.getCurrentElectionId()
        .call()
        .await
        .map_err(BlockchainError::from_call)?
        ._0;

    if current_id < 0 {
        return Ok(None);
    }

    let info = contract.getElectionInfo(current_id as u32).call().await
        .map_err(BlockchainError::from_call)?;

    Ok(Some(ElectionView {
        id:               info.id,
        candidates:       info.candidates.iter().map(b32_to_hex).collect(),
        start_time:       info.startTime.try_into().unwrap_or(u64::MAX),
        end_time:         info.endTime.try_into().unwrap_or(u64::MAX),
        is_active:        info.isActive,
        winner_id:        b32_to_hex(&info.winnerId),
        winning_votes:    info.winningVotes,
        total_votes_cast: info.totalVotesCast,
    }))
}

// ─────────────────────────────────────────────────────────────
// WRITE: open election (bundle)
// ─────────────────────────────────────────────────────────────

/// Builds the calldata + gas estimate for `openElection(candidate, opponent)`.
///
/// Pre-checks (all caught here to avoid wasted gas):
///   • Coop exists in registry.
///   • Candidate ≠ opponent.
///   • Both wallets resolve to a non-zero memberId.
///   • No active election currently exists.
///
/// The contract itself will re-validate these on execution; we duplicate
/// them server-side only to fail fast with a meaningful error message.
pub async fn prepare_open_election_logic(
    subgraph: &SubgraphService,
    blockchain:       &BlockchainService,
    coop_id_hex:      &str,
    candidate_wallet: WalletAddress,
    opponent_wallet:  WalletAddress,
    caller_wallet:    WalletAddress,
) -> Result<OpenElectionBundle, ElectionError> {
    if candidate_wallet == opponent_wallet {
        return Err(ElectionError::SameCandidates);
    }

    let coop_id_b32       = parse_bytes32_coop(coop_id_hex)?;
    let coop_registry = &blockchain.coop_registry;
    let loan_machine_addr = coop_registry.get_loan_machine(coop_id_b32).await?;


    let provider = &blockchain.coop_registry.provider;
    let contract = LoanMachine::new(loan_machine_addr, provider.clone());

    // 1. Resolve wallets → memberIds.  Zero hash = not vinculated.
    //let candidate_addr: Address = wallet_address::to_alloy(&candidate_wallet); <-- deleted
    //let opponent_addr:  Address = wallet_address::to_alloy(&opponent_wallet); <-- deleted

    let candidate_id = resolve_member_id(
        subgraph, provider, loan_machine_addr, &candidate_wallet,
    )
    .await?
    .ok_or_else(|| BlockchainError::WalletNotVinculated(candidate_wallet.clone()))?;

let opponent_id = resolve_member_id(
        subgraph, provider, loan_machine_addr, &opponent_wallet,
    )
    .await?
    .ok_or_else(|| BlockchainError::WalletNotVinculated(opponent_wallet.clone()))?;

    // 2. No active election allowed.
    let current_id: i32 = contract.getCurrentElectionId().call().await
        .map_err(BlockchainError::from_call)?._0;
    if current_id >= 0 {
        return Err(ElectionError::ActiveElectionExists);
    }

    // 3. Encode + estimate gas, scoped to the caller wallet.
    let caller_addr: Address = wallet_address::to_alloy(&caller_wallet);
    let call = contract.openElection(candidate_id, opponent_id);

    let gas = call.clone()
        .from(caller_addr)
        .estimate_gas()
        .await
        .map_err(BlockchainError::from_call)?;

    let data_bytes = call.calldata().clone();

    Ok(OpenElectionBundle {
        to:      format!("{loan_machine_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}

// ─────────────────────────────────────────────────────────────
// helpers (private)
// ─────────────────────────────────────────────────────────────

fn parse_bytes32_coop(hex_str: &str) -> Result<B256, ElectionError> {
    hex_str.parse().map_err(|_| ElectionError::InvalidCoopId(hex_str.to_string()))
}

fn b32_to_hex(b: &B256) -> String {
    format!("0x{}", hex::encode(b.as_slice()))
}