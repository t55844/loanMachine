//! Integration tests for `prepare_add_candidate_logic`.
//!
//! The function flow is:
//!   1. resolve candidate wallet → member_id (subgraph + chain fallback)
//!   2. encode `addCandidate(electionId, candidateId)`
//!   3. estimate gas from caller → catches contract reverts before the tx is sent
//!
//! Error paths tested:
//!   - invalid coop_id                → InvalidCoopId (before any I/O)
//!   - candidate not vinculated       → WalletNotVinculated
//!   - election_id not active         → RS_ElectionNotActive (via gas estimate revert)
//!   - candidate already in election  → RS_InvalidCandidate  (via gas estimate revert)
//!
//! Happy path:
//!   - vinculate a third member, open an election, add that member as a candidate
//!     → bundle carries correct selector, election_id encoding, and candidate member_id

mod common;

use crate::common::{
    bootstrap_active_election, member_events_empty, server_returning,
    vinculate_second_admin, vinculate_third_admin, admin3_member_id,
};

use alloy::primitives::keccak256;

use loan_machine_core::server_logic::elections::{
    prepare_add_candidate_logic, ElectionError,
};
use loan_machine_core::services::blockchain::BlockchainError;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

#[track_caller]
fn expect_contract_revert(err: ElectionError) -> String {
    match err {
        ElectionError::Blockchain(BlockchainError::ContractRevert { message, .. }) => message,
        other => panic!("expected ContractRevert, got {other:?}"),
    }
}

// ── InvalidCoopId ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn prepare_add_candidate_invalid_coop_id() {
    let env = common::get_deployed().await;
    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_add_candidate_logic(
        &subgraph, &env.blockchain,
        "not-a-valid-hex-coop-id",
        0,
        from_alloy(env.approved_wallet),
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    assert!(
        matches!(err, ElectionError::InvalidCoopId(_)),
        "expected InvalidCoopId, got {err:?}"
    );
}

// ── WalletNotVinculated ───────────────────────────────────────────────────────

#[tokio::test]
async fn prepare_add_candidate_non_vinculated_candidate() {
    let env = common::get_deployed().await;
    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    // unapproved_wallet is a stranger never joined to the coop.
    let err = prepare_add_candidate_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        0,
        from_alloy(env.unapproved_wallet), // not vinculated
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    match err {
        ElectionError::Blockchain(BlockchainError::WalletNotVinculated(w)) => {
            assert_eq!(
                w.to_string().to_lowercase(),
                format!("{:#x}", env.unapproved_wallet),
            );
        }
        other => panic!("expected WalletNotVinculated, got {other:?}"),
    }
}

// ── RS_ElectionNotActive ──────────────────────────────────────────────────────

/// Using `election_id = u32::MAX` ensures the election ID will never be active
/// regardless of what other tests have run before this one.
#[tokio::test]
async fn prepare_add_candidate_election_not_active() {
    let env = common::get_deployed().await;
    vinculate_second_admin(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    // Candidate resolves fine (founder is vinculated); gas estimate reverts.
    let err = prepare_add_candidate_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        u32::MAX,                          // non-existent election
        from_alloy(env.second_admin),      // vinculated candidate
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    assert_eq!(
        expect_contract_revert(err),
        "No active election at this time."
    );
}

// ── RS_InvalidCandidate (already in election) ─────────────────────────────────

#[tokio::test]
async fn prepare_add_candidate_already_a_candidate() {
    let env = common::get_deployed().await;
    let (election_id, _) = bootstrap_active_election(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    // The founder is already one of the two initial candidates.
    let err = prepare_add_candidate_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        election_id,
        from_alloy(env.approved_wallet), // already in candidates list
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    assert_eq!(
        expect_contract_revert(err),
        "Invalid candidate."
    );
}

// ── Happy path ────────────────────────────────────────────────────────────────

/// Vinculate admin3, open an election, then add admin3 as a third candidate.
/// Verifies the returned bundle carries the correct selector, calldata layout,
/// and a realistic gas estimate.
#[tokio::test]
async fn prepare_add_candidate_succeeds() {
    let env = common::get_deployed().await;

    // `bootstrap_active_election` opens founder vs admin2.
    // `vinculate_third_admin` approves + vinculates admin3 (not a candidate yet).
    let (election_id, _) = bootstrap_active_election(&env).await;
    let admin3_id = vinculate_third_admin(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let bundle = prepare_add_candidate_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        election_id,
        from_alloy(env.third_admin),     // new candidate (vinculated, not yet in election)
        from_alloy(env.approved_wallet), // caller: founder
    ).await.expect("third_admin is vinculated and not yet a candidate → must succeed");

    // ── bundle shape ──────────────────────────────────────────────────────────
    assert!(bundle.to.starts_with("0x"), "to address should be 0x-prefixed");
    assert_eq!(
        bundle.to.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
        "bundle.to must be the LoanMachine address"
    );
    assert!(bundle.gas_hex.starts_with("0x"), "gas_hex should be 0x-prefixed");

    let gas = u64::from_str_radix(bundle.gas_hex.trim_start_matches("0x"), 16)
        .expect("gas_hex is valid hex");
    assert!(gas > 21_000, "gas estimate should be above the bare-tx minimum");

    // ── calldata correctness ──────────────────────────────────────────────────
    // addCandidate(uint32 electionId, bytes32 candidateId)
    //   selector (4) + electionId padded to 32 + candidateId (32) = 68 bytes
    let raw = hex::decode(bundle.data.trim_start_matches("0x"))
        .expect("bundle.data is valid hex");
    assert_eq!(raw.len(), 4 + 32 + 32, "addCandidate calldata: selector + 2 × 32-byte args");

    let expected_selector = {
        let h = keccak256(b"addCandidate(uint32,bytes32)");
        [h[0], h[1], h[2], h[3]]
    };
    assert_eq!(&raw[..4], &expected_selector, "selector must be addCandidate");

    // election_id is ABI-encoded as a left-zero-padded uint32 in 32 bytes.
    let mut expected_election_id = [0u8; 32];
    expected_election_id[28..].copy_from_slice(&election_id.to_be_bytes());
    assert_eq!(&raw[4..36], &expected_election_id, "first arg must be the election_id");

    // candidateId must be admin3's member_id.
    assert_eq!(
        &raw[36..68],
        admin3_id.as_slice(),
        "second arg must be admin3's member_id"
    );

    // Sanity: admin3_member_id() is the same value we stored on-chain.
    assert_eq!(admin3_id, admin3_member_id());
}
