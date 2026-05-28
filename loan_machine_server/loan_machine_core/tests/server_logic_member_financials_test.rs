mod common;
use crate::common::server_returning;

use loan_machine_core::server_logic::member_financials::{
    get_member_financials_logic, MemberFinancialsError,
};
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;
use serde_json::json;

// Subgraph returns empty arrays for every lookup → all values fall through to chain.
fn empty_subgraph() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [], "ReputationChangedEvents": [] } })
}

#[tokio::test]
async fn happy_path_returns_financials_for_member() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_member_financials_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        wallet.clone(),
    ).await.expect("approved (founder) wallet should return financials");

    assert_eq!(result.wallet, wallet.to_string());
    assert!(result.member_id.starts_with("0x"), "member_id must be 0x-prefixed hex");
    assert_eq!(result.member_id.len(), 66, "member_id must be 32 bytes (64 hex + 0x prefix)");
    assert!(!result.loan_machine_address.is_empty());
}

#[tokio::test]
async fn non_member_wallet_returns_wallet_not_member_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_member_financials_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        wallet,
    ).await;

    assert!(
        matches!(result, Err(MemberFinancialsError::WalletNotMember)),
        "expected WalletNotMember, got {:?}", result,
    );
}

#[tokio::test]
async fn invalid_coop_id_returns_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_member_financials_logic(
        &subgraph,
        &env.blockchain,
        "not-bytes32",
        wallet,
    ).await;

    assert!(
        matches!(result, Err(MemberFinancialsError::InvalidCoopId)),
        "expected InvalidCoopId, got {:?}", result,
    );
}
