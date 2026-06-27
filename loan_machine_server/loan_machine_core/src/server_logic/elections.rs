// loan_machine_core/src/server_logic/elections.rs
//

use alloy::primitives::{Address, B256};
use thiserror::Error;

use loan_machine_models::responses::{ElectionView, OpenElectionBundle};
use loan_machine_models::wallet_address::{self, WalletAddress};

use crate::services::blockchain::{abis::LoanMachine, BlockchainError, BlockchainService};
use crate::services::subgraph::{ SubgraphError, SubgraphService};

use loan_machine_models::responses::VoteBundle;
use crate::server_logic::helpers::{resolve_member_id, resolve_member_wallet,resolve_member_reputation};

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
    #[error(transparent)]
    Subgraph(#[from] SubgraphError),
}


pub async fn get_current_election_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
) -> Result<Option<ElectionView>, ElectionError> {
    let (_coop_id_b32, loan_machine_addr, _provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: ElectionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let current_id: i32 = contract.getCurrentElectionId().call().await
        .map_err(BlockchainError::from_call)?._0;
    if current_id < 0 { return Ok(None); }

    let info = contract.getElectionInfo(current_id as u32).call().await
        .map_err(BlockchainError::from_call)?;

    // Translate every candidate memberId → wallet. Fall back to hex if the
    // subgraph hasn't indexed the member yet.
    let mut candidates_wallets = Vec::with_capacity(info.candidates.len());
    for c in &info.candidates {
        let pretty = match resolve_member_wallet(subgraph, loan_machine_addr, *c).await {
            Some(w) => w.to_string(),
            None    => b32_to_hex(c),
        };
        candidates_wallets.push(pretty);
    }

    let winner_wallet = if info.winnerId == B256::ZERO {
        String::new()
    } else {
        resolve_member_wallet(subgraph, loan_machine_addr, info.winnerId)
            .await
            .map(|w| w.to_string())
            .unwrap_or_else(|| b32_to_hex(&info.winnerId))
    };

    Ok(Some(ElectionView {
        id:               info.id,
        candidates:       candidates_wallets,
        start_time:       info.startTime.try_into().unwrap_or(u64::MAX),
        end_time:         info.endTime.try_into().unwrap_or(u64::MAX),
        is_active:        info.isActive,
        winner_id:        winner_wallet,
        winning_votes:    info.winningVotes,
        total_votes_cast: info.totalVotesCast,
    }))
}



pub async fn prepare_vote_logic(
    subgraph:         &SubgraphService,
    blockchain:       &BlockchainService,
    coop_id_hex:      &str,
    election_id:      u32,
    candidate_wallet: WalletAddress,
    voter_wallet:     WalletAddress,
) -> Result<VoteBundle, ElectionError> {
    let (_coop_id_b32, loan_machine_addr, provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: ElectionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );


    let candidate_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &candidate_wallet)
        .await?
        .ok_or_else(|| BlockchainError::WalletNotVinculated(candidate_wallet.clone()))?;

    let voter_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &voter_wallet)
        .await?
        .ok_or_else(|| BlockchainError::WalletNotVinculated(voter_wallet.clone()))?;

    let _reputation = resolve_member_reputation(
        subgraph, provider.as_ref(), loan_machine_addr, voter_id,
    ).await?;

    let voter_addr: Address = wallet_address::to_alloy(&voter_wallet);
    let call = contract.voteForModerator(election_id, candidate_id, voter_id);

    let gas = call.clone().from(voter_addr).estimate_gas().await
        .map_err(BlockchainError::from_call)?;
    let data_bytes = call.calldata().clone();

    Ok(VoteBundle {
        to:      format!("{loan_machine_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}


pub async fn get_wallet_reputation_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
    wallet:      WalletAddress,
) -> Result<i32, ElectionError> {
    let (_coop_id_b32, loan_machine_addr, provider, _contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: ElectionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let member_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &wallet)
        .await?
        .ok_or_else(|| BlockchainError::WalletNotVinculated(wallet))?;

    let contract = LoanMachine::new(loan_machine_addr, provider.clone());
    Ok(contract.getReputation(member_id).call().await
        .map_err(BlockchainError::from_call)?._0)
}


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

    let (_coop_id_b32, loan_machine_addr, provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: ElectionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );


    let candidate_id = resolve_member_id(
        subgraph, provider.as_ref(), loan_machine_addr, &candidate_wallet,
    )
    .await?
    .ok_or_else(|| BlockchainError::WalletNotVinculated(candidate_wallet.clone()))?;

    let opponent_id = resolve_member_id(
        subgraph, provider.as_ref(), loan_machine_addr, &opponent_wallet,
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

pub async fn prepare_add_candidate_logic(
    subgraph:         &SubgraphService,
    blockchain:       &BlockchainService,
    coop_id_hex:      &str,
    election_id:      u32,
    candidate_wallet: WalletAddress,
    caller_wallet:    WalletAddress,
) -> Result<OpenElectionBundle, ElectionError> {
    let (_coop_id_b32, loan_machine_addr, provider, contract) = coop_context!(
        blockchain:      blockchain,
        coop_id_hex:     coop_id_hex,
        invalid_coop_id: ElectionError::InvalidCoopId(coop_id_hex.into()),
        contract,
    );

    let candidate_id = resolve_member_id(subgraph, provider.as_ref(), loan_machine_addr, &candidate_wallet)
        .await?
        .ok_or_else(|| BlockchainError::WalletNotVinculated(candidate_wallet.clone()))?;

    let caller_addr: Address = wallet_address::to_alloy(&caller_wallet);
    let call = contract.addCandidate(election_id, candidate_id);

    let gas = call.clone()
        .from(caller_addr)
        .estimate_gas()
        .await
        .map_err(BlockchainError::from_gas_estimate)?;

    let data_bytes = call.calldata().clone();

    Ok(OpenElectionBundle {
        to:      format!("{loan_machine_addr:#x}"),
        data:    format!("0x{}", hex::encode(&data_bytes)),
        gas_hex: format!("0x{gas:x}"),
    })
}

use crate::server_logic::subgraph_queries::last_closed_election::fetch_last_closed_election;

pub async fn get_last_closed_election_logic(
    subgraph:    &SubgraphService,
    blockchain:  &BlockchainService,
    coop_id_hex: &str,
) -> Result<Option<ElectionView>, ElectionError> {
    let coop_id_b32       = parse_bytes32_coop(coop_id_hex)?;
    let loan_machine_addr = blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
    Ok(fetch_last_closed_election(subgraph, loan_machine_addr).await?)
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