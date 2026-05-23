mod common;
use crate::common::{one_cooperative_payload, server_returning, server_returning_status};

use loan_machine_core::server_logic::subgraph_queries::cooperative_by_id::fetch_one_coop;
use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};
use serde_json::json;

#[tokio::test]
async fn fetch_one_coop_returns_some_when_found() {
    let server   = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let row = fetch_one_coop(&subgraph, "0xbbb")
        .await.expect("expected Ok")
        .expect("expected Some(row)");

    assert_eq!(row.id, "0xaaa");
    assert_eq!(row.coop_id, "0xbbb");
    assert_eq!(row.name, "Coop One");
    assert_eq!(row.loan_machine, "0xccc");
    assert!(row.active);
    assert_eq!(row.registered_at, "1700000000");
}

#[tokio::test]
async fn fetch_one_coop_returns_none_when_not_found() {
    let server   = server_returning(json!({ "data": { "cooperatives": [] } })).await;
    let subgraph = SubgraphService::new(server.uri());

    let row = fetch_one_coop(&subgraph, "0xdoesnotexist").await.unwrap();
    assert!(row.is_none());
}

#[tokio::test]
async fn fetch_one_coop_propagates_graphql_error() {
    let server = server_returning(json!({
        "errors": [{ "message": "Field coopId not found" }]
    })).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_one_coop(&subgraph, "0xbbb").await.unwrap_err();
    match err {
        SubgraphError::Graphql(msg) =>
            assert!(msg.contains("Field coopId not found"), "got: {msg}"),
        other => panic!("expected Graphql error, got {other:?}"),
    }
}

#[tokio::test]
async fn fetch_one_coop_propagates_http_500() {
    let server = server_returning_status(500).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_one_coop(&subgraph, "0xbbb").await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)));
}

#[tokio::test]
async fn fetch_one_coop_fails_on_bad_url() {
    let subgraph = SubgraphService::new("http://127.0.0.1:1".to_string());
    let err = fetch_one_coop(&subgraph, "0xbbb").await.unwrap_err();
    assert!(matches!(err, SubgraphError::Network(_)));
}

#[tokio::test]
async fn fetch_one_coop_decode_error_on_wrong_shape() {
    let server = server_returning(json!({
        "data": { "cooperatives": "not an array" }
    })).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = fetch_one_coop(&subgraph, "0xbbb").await.unwrap_err();
    assert!(matches!(err, SubgraphError::Decode(_)));
}