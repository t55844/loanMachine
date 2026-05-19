// tests/subgraph_queries_user_coops_test.rs

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use loan_machine_core::server_logic::subgraph_queries::user_coops::get_user_coops_logic;
use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};

const WALLET: &str = "0xabc0000000000000000000000000000000000001";

// ── helpers ──────────────────────────────────────────────────

async fn server_returning(body: serde_json::Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}

async fn server_returning_status(status: u16) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(status))
        .mount(&server)
        .await;
    server
}

fn coop(id: &str, name: &str) -> serde_json::Value {
    json!({
        "id":          id,
        "coopId":      id,
        "name":        name,
        "loanMachine": "0xdef0000000000000000000000000000000000000",
        "active":      true,
    })
}

fn events_payload(coops: Vec<serde_json::Value>) -> serde_json::Value {
    let events: Vec<_> = coops.into_iter().map(|c| json!({ "cooperative": c })).collect();
    json!({ "data": { "memberRegisteredEvents": events } })
}

// ── happy path ───────────────────────────────────────────────

#[tokio::test]
async fn user_coops_returns_parsed_rows() {
    let server   = server_returning(events_payload(vec![
        coop("0x1", "Coop A"),
        coop("0x2", "Coop B"),
    ])).await;
    let subgraph = SubgraphService::new(server.uri());

    let coops = get_user_coops_logic(&subgraph, WALLET).await.unwrap();

    assert_eq!(coops.len(), 2);
    assert_eq!(coops[0].name, "Coop A");
    assert_eq!(coops[1].name, "Coop B");
}

#[tokio::test]
async fn user_coops_returns_empty_when_wallet_unknown() {
    let server   = server_returning(events_payload(vec![])).await;
    let subgraph = SubgraphService::new(server.uri());

    let coops = get_user_coops_logic(&subgraph, WALLET).await.unwrap();
    assert!(coops.is_empty());
}

#[tokio::test]
async fn user_coops_dedups_across_wallets_in_same_coop() {
    // A member with two wallets registered in the same coop → two events,
    // same `cooperative.id`. Function must collapse to one row.
    let server   = server_returning(events_payload(vec![
        coop("0x1", "Coop A"),
        coop("0x1", "Coop A"),
        coop("0x2", "Coop B"),
    ])).await;
    let subgraph = SubgraphService::new(server.uri());

    let coops = get_user_coops_logic(&subgraph, WALLET).await.unwrap();

    assert_eq!(coops.len(), 2);
    let ids: Vec<_> = coops.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"0x1"));
    assert!(ids.contains(&"0x2"));
}

// ── error paths ──────────────────────────────────────────────

#[tokio::test]
async fn user_coops_propagates_graphql_error() {
    let server   = server_returning(json!({
        "errors": [{ "message": "Type Cooperative not found" }]
    })).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = get_user_coops_logic(&subgraph, WALLET).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Graphql(_)));
}

#[tokio::test]
async fn user_coops_propagates_http_500() {
    let server   = server_returning_status(500).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = get_user_coops_logic(&subgraph, WALLET).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)));
}

#[tokio::test]
async fn user_coops_decode_error_on_wrong_shape() {
    let server   = server_returning(json!({
        "data": { "memberRegisteredEvents": "not an array" }
    })).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = get_user_coops_logic(&subgraph, WALLET).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Decode(_)));
}