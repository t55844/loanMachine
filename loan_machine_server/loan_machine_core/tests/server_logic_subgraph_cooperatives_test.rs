mod common;
use crate::common::{
    cooperatives_with, empty_payload, graphql_error_payload, one_cooperative_payload,
    server_returning, server_returning_status,
};

use loan_machine_core::server_logic::subgraph_queries::cooperatives::fetch_cooperatives;
use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};
use serde_json::json;

#[tokio::test]
async fn fetch_cooperatives_returns_parsed_rows() {
    let server = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let rows = fetch_cooperatives(&subgraph).await.expect("expected Ok");

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id, "0xaaa");
    assert_eq!(row.coop_id, "0xbbb");
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
    let server = server_returning(cooperatives_with(vec![
        json!({ "id": "0x1", "coopId": "0xa", "name": "A",
                "loanMachine": "0x10", "active": true, "registeredAt": "100" }),
        json!({ "id": "0x2", "coopId": "0xb", "name": "B",
                "loanMachine": "0x20", "active": false, "registeredAt": "200" }),
    ])).await;

    let subgraph = SubgraphService::new(server.uri());
    let rows = fetch_cooperatives(&subgraph).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "A");
    assert_eq!(rows[1].name, "B");
    assert!(rows[0].active);
    assert!(!rows[1].active);
}

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
    assert!(matches!(err, SubgraphError::Network(_)));
}

#[tokio::test]
async fn fetch_cooperatives_fails_on_bad_url() {
    let subgraph = SubgraphService::new("http://127.0.0.1:1".to_string());
    let err = fetch_cooperatives(&subgraph).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)));
}

#[tokio::test]
async fn fetch_cooperatives_decode_error_on_wrong_shape() {
    let server = server_returning(json!({
        "data": { "cooperatives": "not an array" }
    })).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_cooperatives(&subgraph).await.unwrap_err();
    assert!(matches!(err, SubgraphError::Decode(_)));
}