mod common;
use crate::common::{
    cooperative_minimal, member_events_with, server_returning, server_returning_status,
};

use loan_machine_core::server_logic::subgraph_queries::user_coops::get_user_coops_logic;
use loan_machine_core::services::subgraph::{SubgraphError, SubgraphService};
use serde_json::json;

const WALLET: &str = "0xabc0000000000000000000000000000000000001";

fn events_for(coops: Vec<serde_json::Value>) -> serde_json::Value {
    let events: Vec<_> = coops.into_iter()
        .map(|c| json!({ "cooperative": c }))
        .collect();
    member_events_with(events)
}

#[tokio::test]
async fn user_coops_returns_parsed_rows() {
    let server   = server_returning(events_for(vec![
        cooperative_minimal("0x1", "Coop A"),
        cooperative_minimal("0x2", "Coop B"),
    ])).await;
    let subgraph = SubgraphService::new(server.uri());

    let coops = get_user_coops_logic(&subgraph, WALLET).await.unwrap();

    assert_eq!(coops.len(), 2);
    assert_eq!(coops[0].name, "Coop A");
    assert_eq!(coops[1].name, "Coop B");
}

#[tokio::test]
async fn user_coops_returns_empty_when_wallet_unknown() {
    let server   = server_returning(events_for(vec![])).await;
    let subgraph = SubgraphService::new(server.uri());

    let coops = get_user_coops_logic(&subgraph, WALLET).await.unwrap();
    assert!(coops.is_empty());
}

#[tokio::test]
async fn user_coops_dedups_across_wallets_in_same_coop() {
    // A member with two wallets registered in the same coop → two events,
    // same `cooperative.id`. Function must collapse to one row.
    let server   = server_returning(events_for(vec![
        cooperative_minimal("0x1", "Coop A"),
        cooperative_minimal("0x1", "Coop A"),
        cooperative_minimal("0x2", "Coop B"),
    ])).await;
    let subgraph = SubgraphService::new(server.uri());

    let coops = get_user_coops_logic(&subgraph, WALLET).await.unwrap();

    assert_eq!(coops.len(), 2);
    let ids: Vec<_> = coops.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"0x1"));
    assert!(ids.contains(&"0x2"));
}

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