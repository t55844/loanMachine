
use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use alloy::primitives::Address;

use loan_machine_core::server_logic::subgraph_queries::last_closed_election::fetch_last_closed_election;
use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};

const LM_ADDR: &str = "0x1111111111111111111111111111111111111111";

fn addr() -> Address { LM_ADDR.parse().unwrap() }

// ── happy path: close event + winner translation both succeed ────────

#[tokio::test]
async fn last_closed_returns_event_with_translated_winner() {
    let server = MockServer::start().await;

    let winner_member  = "0x645810b37c4e1f1b9bbe4cd49bf37afdd19a44eb81c515042f2fa38df3cb2655";
    let winner_wallet  = "0x9535424e6f3f82c9c3aca0d747c59e187f216ac4";

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("electionClosedEvents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "electionClosedEvents": [{
                    "electionId":     0,
                    "winnerId":       winner_member,
                    "winningVotes":   1,
                    "blockTimestamp": "1716135121",
                }]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("memberRegisteredEvents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "memberRegisteredEvents": [{ "wallet": winner_wallet }]
            }
        })))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());

    let view = fetch_last_closed_election(&subgraph, addr())
        .await
        .unwrap()
        .expect("should return Some(ElectionView)");

    assert_eq!(view.id,                    0);
    assert_eq!(view.winning_votes,         1);
    assert_eq!(view.end_time,              1_716_135_121);
    assert!(!view.is_active);
    assert_eq!(view.winner_id.to_lowercase(), winner_wallet);
}

// ── happy path: close event found, translation misses → falls back to hex ──

#[tokio::test]
async fn last_closed_falls_back_to_raw_hex_when_translation_misses() {
    let server = MockServer::start().await;

    let raw_winner = "0x645810b37c4e1f1b9bbe4cd49bf37afdd19a44eb81c515042f2fa38df3cb2655";

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("electionClosedEvents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "electionClosedEvents": [{
                    "electionId":     0,
                    "winnerId":       raw_winner,
                    "winningVotes":   1,
                    "blockTimestamp": "1716135121",
                }]
            }
        })))
        .mount(&server)
        .await;

    // Translation subgraph hits an empty result — simulates the indexer
    // hasn't caught up with the member registration yet.
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("memberRegisteredEvents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "memberRegisteredEvents": [] }
        })))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());

    let view = fetch_last_closed_election(&subgraph, addr())
        .await
        .unwrap()
        .expect("should return Some");

    assert_eq!(view.winner_id, raw_winner,
        "winner_id should fall back to raw memberId hex when translation misses");
}

// ── empty: no closed elections yet ───────────────────────────────────

#[tokio::test]
async fn last_closed_returns_none_when_no_events() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "electionClosedEvents": [] }
        })))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());
    let result = fetch_last_closed_election(&subgraph, addr()).await.unwrap();
    assert!(result.is_none());
}

// ── error propagation ────────────────────────────────────────────────

#[tokio::test]
async fn last_closed_propagates_graphql_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "errors": [{ "message": "indexer is down" }]
        })))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());
    let err = fetch_last_closed_election(&subgraph, addr()).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Graphql(_)));
}

#[tokio::test]
async fn last_closed_propagates_http_500() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());
    let err = fetch_last_closed_election(&subgraph, addr()).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)));
}