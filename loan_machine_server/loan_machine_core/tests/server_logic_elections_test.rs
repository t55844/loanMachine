mod common;

use crate::common::payloads::member_events_empty;
use crate::common::subgraph_mock::server_returning;
use crate::common::vinculate_second_admin;

use alloy::primitives::Address;
use serde_json::json;
use wiremock::MockServer;

use loan_machine_core::server_logic::elections::{
    get_current_election_logic, prepare_open_election_logic, prepare_vote_logic, ElectionError,
};
use loan_machine_core::services::blockchain::BlockchainError;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

#[tokio::test]
async fn prepare_open_election_rejects_same_candidate_and_opponent() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    // Subgraph wired up but never reached — the same-candidate check
    // short-circuits before any I/O.
    let server   = MockServer::start().await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_open_election_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        wallet.clone(), wallet.clone(), wallet,
    ).await.unwrap_err();

    assert!(matches!(err, ElectionError::SameCandidates));
}

#[tokio::test]
async fn prepare_open_election_errors_when_candidate_not_vinculated() {
    let env       = common::get_deployed().await;
    let candidate = from_alloy(env.unapproved_wallet);
    let opponent_addr: Address =
        "0x000000000000000000000000000000000000bEEF".parse().unwrap();
    let opponent  = from_alloy(opponent_addr);

    // Subgraph miss for both → chain fallback returns B256::ZERO → resolver
    // returns None → ok_or_else surfaces WalletNotVinculated.
    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_open_election_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        candidate.clone(), opponent, candidate,
    ).await.unwrap_err();

    match err {
        ElectionError::Blockchain(BlockchainError::WalletNotVinculated(w)) => {
            assert_eq!(w.to_string().to_lowercase(),
                       format!("{:#x}", env.unapproved_wallet));
        }
        other => panic!("expected WalletNotVinculated, got {other:?}"),
    }
}

#[tokio::test]
async fn get_current_election_returns_none_on_fresh_coop() {
    let env = common::get_deployed().await;

    let payload = json!({
        "data": { "memberRegisteredEvents": [{
            "cooperative": {
                "coopId":   "0xefcbd6de3a895f71f3888329edcba05543ab0bd0f5833a5d84bd4352ee45fec9",
                "memberId": "0xdcb6857f0be97113138f58260b76a23ec1b831907bfd709276888f6a24ad41e4",
                "isFirstWallet": true,
            },
            "wallet": "0x000000000000000000000000000000000000bEEF",
        }] }
    });
    let server   = server_returning(payload).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_current_election_logic(&subgraph, &env.blockchain, &env.coop_id_hex)
        .await
        .expect("read should not error on a healthy contract");

    assert!(result.is_none(), "no election should be active on a fresh coop");
}

// ── prepare_open_election_logic — success ────────────────────────────

#[tokio::test]
async fn prepare_open_election_succeeds() {
    let env = common::get_deployed().await;
    vinculate_second_admin(&env).await;

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let bundle = prepare_open_election_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), // candidate: founder (vinculated)
        from_alloy(env.second_admin),    // opponent: admin2 (vinculated)
        from_alloy(env.approved_wallet), // caller: founder
    ).await.expect("both vinculated, no active election → should succeed");

    assert!(bundle.to.starts_with("0x"));
    assert!(bundle.gas_hex.starts_with("0x"));
    assert_eq!(bundle.to.to_lowercase(), env.loan_machine_address.to_lowercase());

    let raw = hex::decode(bundle.data.trim_start_matches("0x"))
        .expect("data is valid hex");
    assert_eq!(raw.len(), 4 + 32 + 32, "openElection: selector + 2 × 32-byte args");

    let expected_selector = {
        let h = alloy::primitives::keccak256(b"openElection(bytes32,bytes32)");
        [h[0], h[1], h[2], h[3]]
    };
    assert_eq!(&raw[..4], &expected_selector, "selector is openElection");
    assert_eq!(&raw[4..36], env.founder_member_id.as_slice(), "first arg is founder member_id");
    assert_ne!(&raw[36..68], &[0u8; 32], "second arg (opponent member_id) is non-zero");
}

// ── prepare_vote_logic — vinculation error paths ─────────────────────

#[tokio::test]
async fn prepare_vote_rejects_non_vinculated_candidate() {
    let env     = common::get_deployed().await;
    let server  = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_vote_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        0,
        from_alloy(env.unapproved_wallet), // candidate: not vinculated
        from_alloy(env.approved_wallet),   // voter: vinculated (never checked)
    ).await.unwrap_err();

    match err {
        ElectionError::Blockchain(BlockchainError::WalletNotVinculated(w)) => {
            assert_eq!(w.to_string().to_lowercase(),
                       format!("{:#x}", env.unapproved_wallet));
        }
        other => panic!("expected WalletNotVinculated for candidate, got {other:?}"),
    }
}

#[tokio::test]
async fn prepare_vote_rejects_non_vinculated_voter() {
    let env     = common::get_deployed().await;
    let server  = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_vote_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        0,
        from_alloy(env.approved_wallet),   // candidate: vinculated
        from_alloy(env.unapproved_wallet), // voter: not vinculated
    ).await.unwrap_err();

    match err {
        ElectionError::Blockchain(BlockchainError::WalletNotVinculated(w)) => {
            assert_eq!(w.to_string().to_lowercase(),
                       format!("{:#x}", env.unapproved_wallet));
        }
        other => panic!("expected WalletNotVinculated for voter, got {other:?}"),
    }
}

// ── prepare_vote_logic — election not active ─────────────────────────

#[tokio::test]
async fn prepare_vote_rejects_election_not_active() {
    let env     = common::get_deployed().await;
    let server  = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    // Both candidate and voter resolve to founder (vinculated) — the vinculation
    // checks pass. Gas estimation for a non-existent election_id triggers the revert.
    let err = prepare_vote_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex,
        u32::MAX,
        from_alloy(env.approved_wallet),
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    match err {
        ElectionError::Blockchain(BlockchainError::ContractRevert { message, .. }) => {
            assert_eq!(message, "No active election at this time.");
        }
        other => panic!("expected ContractRevert(ElectionNotActive), got {other:?}"),
    }
}