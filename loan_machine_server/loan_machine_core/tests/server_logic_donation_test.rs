mod common;

use crate::common::server_returning;

use loan_machine_core::server_logic::donation::{donate_logic, DonationLogicError};
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;
use serde_json::json;

// Subgraph returns empty arrays for every lookup → all values fall through to chain.
fn empty_subgraph() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [], "ReputationChangedEvents": [] } })
}

#[tokio::test]
async fn happy_path_returns_bundle_for_member() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = donate_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        &env.usdt_address,
        wallet,
        "1000000".to_string(), // 1 USDT (6 decimals)
    ).await.expect("approved (founder) wallet should return a donation bundle");

    assert_eq!(result.usdt_address.to_lowercase(), env.usdt_address.to_lowercase());
    assert!(!result.loan_machine_address.is_empty());

    // approve(address,uint256) selector
    assert!(result.approve_calldata.starts_with("0x095ea7b3"), "unexpected approve selector: {}", result.approve_calldata);
    assert!(result.donate_calldata.starts_with("0x"), "donate calldata must be 0x-prefixed hex");
    assert_ne!(result.approve_calldata, result.donate_calldata);

    assert!(u64::from_str_radix(result.gas_approve.trim_start_matches("0x"), 16).unwrap() > 0);
    assert!(u64::from_str_radix(result.gas_donate.trim_start_matches("0x"), 16).unwrap() > 0);
}

#[tokio::test]
async fn non_member_wallet_returns_wallet_not_member_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = donate_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        &env.usdt_address,
        wallet,
        "1000000".to_string(),
    ).await;

    assert!(
        matches!(result, Err(DonationLogicError::WalletNotMember)),
        "expected WalletNotMember, got {:?}", result,
    );
}

#[tokio::test]
async fn invalid_amount_returns_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = donate_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        &env.usdt_address,
        wallet,
        "not-a-number".to_string(),
    ).await;

    assert!(
        matches!(result, Err(DonationLogicError::InvalidAmount(_))),
        "expected InvalidAmount, got {:?}", result,
    );
}

#[tokio::test]
async fn amount_above_balance_returns_insufficient_balance_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    // MockUSDT mints 100 USDT (6 decimals) to the deployer at construction —
    // ask for far more than that.
    let result = donate_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        &env.usdt_address,
        wallet,
        "1000000000000".to_string(), // 1,000,000 USDT
    ).await;

    assert!(
        matches!(result, Err(DonationLogicError::InsufficientBalance { .. })),
        "expected InsufficientBalance, got {:?}", result,
    );
}

#[tokio::test]
async fn invalid_coop_id_returns_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = donate_logic(
        &subgraph,
        &env.blockchain,
        "not-bytes32",
        &env.usdt_address,
        wallet,
        "1000000".to_string(),
    ).await;

    assert!(
        matches!(result, Err(DonationLogicError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {:?}", result,
    );
}
