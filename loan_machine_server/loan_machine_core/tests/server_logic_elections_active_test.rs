//! E2E tests for election flows that require an active election.
//!
//! `bootstrap_active_election` opens an election between the founder (admin1)
//! and admin2 on the first call; subsequent calls are no-ops (idempotent
//! chain read). All tests share the same anvil instance and the same open
//! election for the lifetime of the test binary.

mod common;
use crate::common::{
    bootstrap_active_election, cast_vote_as_second_admin,
    member_events_empty, server_returning,
};

use alloy::primitives::keccak256;

use loan_machine_core::server_logic::elections::{
    get_current_election_logic, prepare_open_election_logic, prepare_vote_logic, ElectionError,
};
use loan_machine_core::services::blockchain::BlockchainError;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

#[track_caller]
fn expect_revert_msg(err: ElectionError) -> String {
    match err {
        ElectionError::Blockchain(BlockchainError::ContractRevert { message, .. }) => message,
        other => panic!("expected ContractRevert, got {other:?}"),
    }
}

// ── prepare_open_election_logic ──────────────────────────────────────

/// RS_ActiveElectionExists path: the server logic checks `getCurrentElectionId`
/// before encoding, so the guard fires before any gas estimation.
#[tokio::test]
async fn prepare_open_election_rejects_active_election() {
    let env = common::get_deployed().await;
    bootstrap_active_election(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_open_election_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet),
        from_alloy(env.second_admin),
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    assert!(
        matches!(err, ElectionError::ActiveElectionExists),
        "expected ActiveElectionExists, got {err:?}"
    );
}

// ── get_current_election_logic ───────────────────────────────────────

#[tokio::test]
async fn get_current_election_returns_election_view() {
    let env = common::get_deployed().await;
    let (election_id, _) = bootstrap_active_election(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let view = get_current_election_logic(&subgraph, &env.blockchain, &env.coop_id_hex)
        .await
        .expect("read should not error with active election")
        .expect("election is active → result should be Some");

    assert_eq!(view.id, election_id);
    assert!(view.is_active, "election is open");
    assert_eq!(view.candidates.len(), 2, "founder vs admin2");
    assert_eq!(view.winner_id, "", "no winner while election is active");
}

// ── prepare_vote_logic ───────────────────────────────────────────────

/// Happy path: the founder is vinculated and is a candidate in the active
/// election. Gas estimation for self-voting must succeed.
#[tokio::test]
async fn prepare_vote_logic_succeeds() {
    let env = common::get_deployed().await;
    let (election_id, _) = bootstrap_active_election(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let bundle = prepare_vote_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        election_id,
        from_alloy(env.approved_wallet), // candidate: founder (in candidates list)
        from_alloy(env.approved_wallet), // voter: founder (vinculated)
    ).await.expect("founder is vinculated + in candidates → gas estimate must succeed");

    assert!(bundle.to.starts_with("0x"));
    assert!(bundle.gas_hex.starts_with("0x"));
    assert_eq!(bundle.to.to_lowercase(), env.loan_machine_address.to_lowercase());

    let gas = u64::from_str_radix(bundle.gas_hex.trim_start_matches("0x"), 16)
        .expect("gas_hex is valid hex");
    assert!(gas > 21_000, "gas estimate above bare-tx minimum");

    // voteForModerator(uint32 electionId, bytes32 candidateId, bytes32 memberId)
    //   selector (4) + electionId padded (32) + candidateId (32) + memberId (32) = 100
    let raw = hex::decode(bundle.data.trim_start_matches("0x"))
        .expect("data is valid hex");
    assert_eq!(raw.len(), 4 + 32 + 32 + 32, "voteForModerator: selector + 3 × 32-byte args");

    let expected_selector = {
        let h = keccak256(b"voteForModerator(uint32,bytes32,bytes32)");
        [h[0], h[1], h[2], h[3]]
    };
    assert_eq!(&raw[..4], &expected_selector, "selector is voteForModerator");
}

/// RS_MemberAlreadyVoted: admin2 casts a real vote on-chain, then the server
/// logic's gas estimation for a second vote by admin2 triggers the revert.
/// admin2 has 0 reputation (weight 1) so a single vote does not trigger
/// unbeatable-majority auto-close — the election remains open.
#[tokio::test]
async fn prepare_vote_rejects_already_voted() {
    let env = common::get_deployed().await;
    let (election_id, admin2_id) = bootstrap_active_election(&env).await;

    cast_vote_as_second_admin(&env, election_id, admin2_id, admin2_id).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_vote_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        election_id,
        from_alloy(env.second_admin), // candidate: admin2 (in candidates list)
        from_alloy(env.second_admin), // voter: admin2 (already voted on-chain)
    ).await.unwrap_err();

    assert_eq!(
        expect_revert_msg(err),
        "This member has already voted in this election."
    );
}
