// tests/elections_test.rs

mod common;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use alloy::primitives::Address;

use loan_machine_core::server_logic::elections::{
    get_current_election_logic, prepare_open_election_logic, ElectionError,
};
use loan_machine_core::services::blockchain::BlockchainError;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

#[tokio::test]
async fn prepare_open_election_rejects_same_candidate_and_opponent() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    // Subgraph is wired up but never reached — the same-candidate check
    // short-circuits before any I/O.
    let server   = MockServer::start().await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_open_election_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        wallet.clone(),
        wallet.clone(),
        wallet,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, ElectionError::SameCandidates));
}

#[tokio::test]
async fn prepare_open_election_errors_when_candidate_not_vinculated() {
    let env       = common::get_deployed().await;
    let candidate = from_alloy(env.unapproved_wallet);
    // Any distinct unvinculated address works for the second slot.
    let opponent_addr: Address =
        "0x000000000000000000000000000000000000bEEF".parse().unwrap();
    let opponent  = from_alloy(opponent_addr);

    // Subgraph miss for both → chain fallback returns B256::ZERO → resolver
    // returns None → ok_or_else surfaces BlockchainError::WalletNotVinculated,
    // which #[from] lifts into ElectionError::Blockchain.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "memberRegisteredEvents": [] }
        })))
        .mount(&server)
        .await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_open_election_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        candidate.clone(),
        opponent,
        candidate,
    )
    .await
    .unwrap_err();

    match err {
        ElectionError::Blockchain(BlockchainError::WalletNotVinculated(w)) => {
            // Candidate is resolved first, so it should be the one that surfaced.
            assert_eq!(w.to_string().to_lowercase(),
                       format!("{:#x}", env.unapproved_wallet));
        }
        other => panic!("expected WalletNotVinculated, got {other:?}"),
    }
}

#[tokio::test]
async fn get_current_election_returns_none_on_fresh_coop() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "memberRegisteredEvents": [{
                    "cooperative": {
                        "coopId":  "0xefcbd6de3a895f71f3888329edcba05543ab0bd0f5833a5d84bd4352ee45fec9",
                        "memberId":  "0xdcb6857f0be97113138f58260b76a23ec1b831907bfd709276888f6a24ad41e4",
                        "isFirstWallet":      true,
                    },
                    "wallet": "0x000000000000000000000000000000000000bEEF",
                }] }
        })))
        .mount(&server)
        .await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_current_election_logic(&subgraph, &env.blockchain, &env.coop_id_hex)
        .await
        .expect("read should not error on a healthy contract");

    assert!(result.is_none(), "no election should be active on a fresh coop");
}