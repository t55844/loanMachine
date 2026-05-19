// Tests for fetch_cooperatives against a mocked GraphQL endpoint.
// No network, no graph-node, no anvil — just an in-process HTTP mock.

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use loan_machine_core::server_logic::subgraph_queries::cooperatives::fetch_cooperatives;
use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};

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

fn one_cooperative_payload() -> serde_json::Value {
    json!({
        "data": {
            "cooperatives": [
                {
                    "id": "0xaaa",
                    "coopId": "0xbbb",
                    "name": "Coop One",
                    "loanMachine": "0xccc",
                    "active": true,
                    "registeredAt": "1700000000"
                }
            ]
        }
    })
}

fn empty_payload() -> serde_json::Value {
    json!({ "data": { "cooperatives": [] } })
}

fn graphql_error_payload() -> serde_json::Value {
    json!({
        "errors": [
            { "message": "Type Cooperative not found" }
        ]
    })
}

// ── happy path ───────────────────────────────────────────────

#[tokio::test]
async fn fetch_cooperatives_returns_parsed_rows() {
    let server = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let rows = fetch_cooperatives(&subgraph)
        .await
        .expect("expected Ok");

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id, "0xaaa");
    assert_eq!(row.coop_id, "0xbbb");                  // camelCase -> snake_case
    assert_eq!(row.name, "Coop One");
    assert_eq!(row.loan_machine, "0xccc");
    assert!(row.active);
    assert_eq!(row.registered_at, "1700000000");
}

#[tokio::test]
async fn fetch_cooperatives_returns_empty_vec_when_no_data() {
    let server = server_returning(empty_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let rows = fetch_cooperatives(&subgraph).await.unwrap();
    assert!(rows.is_empty());
}

#[tokio::test]
async fn fetch_cooperatives_parses_multiple_rows() {
    let server = server_returning(json!({
        "data": {
            "cooperatives": [
                { "id": "0x1", "coopId": "0xa", "name": "A",
                  "loanMachine": "0x10", "active": true,
                  "registeredAt": "100" },
                { "id": "0x2", "coopId": "0xb", "name": "B",
                  "loanMachine": "0x20", "active": false,
                  "registeredAt": "200" }
            ]
        }
    })).await;

    let subgraph = SubgraphService::new(server.uri());
    let rows = fetch_cooperatives(&subgraph).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "A");
    assert_eq!(rows[1].name, "B");
    assert!(rows[0].active);
    assert!(!rows[1].active);
}

// ── error paths ──────────────────────────────────────────────

#[tokio::test]
async fn fetch_cooperatives_propagates_graphql_error() {
    let server = server_returning(graphql_error_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_cooperatives(&subgraph).await.unwrap_err();
    match err {
        SubgraphError::Graphql(msg) => {
            assert!(msg.contains("Type Cooperative not found"),
                "expected message in error, got: {msg}");
        }
        other => panic!("expected Graphql error, got {other:?}"),
    }
}

#[tokio::test]
async fn fetch_cooperatives_propagates_http_500() {
    let server = server_returning_status(500).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_cooperatives(&subgraph).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)),
        "expected Network error, got {err:?}");
}

#[tokio::test]
async fn fetch_cooperatives_fails_on_bad_url() {
    // Port 1 is reserved + nothing should be listening — guaranteed connection failure.
    let subgraph = SubgraphService::new("http://127.0.0.1:1".to_string());
    let err = fetch_cooperatives(&subgraph).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)));
}

#[tokio::test]
async fn fetch_cooperatives_decode_error_on_wrong_shape() {
    // Server returns valid JSON but wrong field types.
    let server = server_returning(json!({
        "data": { "cooperatives": "not an array" }
    })).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_cooperatives(&subgraph).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Decode(_)),
        "expected Decode error, got {err:?}");
}

