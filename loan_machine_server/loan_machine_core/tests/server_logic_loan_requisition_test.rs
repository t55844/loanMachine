mod common;

use common::{donate_as, withdraw_as, vinculate_second_admin, server_returning};
use loan_machine_core::server_logic::loan_requisition::LoanRequisitionError::InvalidDaysInterval;

use alloy::primitives::U256;
use loan_machine_core::server_logic::loan_requisition::{
    create_loan_requisition_logic, LoanRequisitionError,
};
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;
use serde_json::json;

fn empty_subgraph() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [], "ReputationChangedEvents": [] } })
}

// ── pre-logic gates (no chain call) ──────────────────────────

#[tokio::test]
async fn invalid_coop_id_returns_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, "not-bytes32",
        from_alloy(env.approved_wallet), "1000000".into(), 6, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {result:?}",
    );
}

#[tokio::test]
async fn non_member_wallet_returns_not_member_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.unapproved_wallet), "1000000".into(), 6, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::WalletNotMember)),
        "expected WalletNotMember, got {result:?}",
    );
}

#[tokio::test]
async fn invalid_amount_string_returns_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "not-a-number".into(), 6, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidAmount(_))),
        "expected InvalidAmount, got {result:?}",
    );
}

#[tokio::test]
async fn zero_amount_returns_invalid_amount() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "0".into(), 6, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidAmount(_))),
        "expected InvalidAmount, got {result:?}",
    );
}

// ── server-layer interval guard ───────────────────────────────

#[tokio::test]
async fn zero_interval_returns_invalid_days_interval() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "1000000".into(), 6, 0,
    ).await;

    assert!(
        matches!(result, Err(InvalidDaysInterval)),
        "expected InvalidDaysInterval for interval=0, got {result:?}",
    );
}

#[tokio::test]
async fn interval_above_30_returns_invalid_days_interval() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "1000000".into(), 6, 31,
    ).await;

    assert!(
        matches!(result, Err(InvalidDaysInterval)),
        "expected InvalidDaysInterval for interval=31, got {result:?}",
    );
}

// ── contract-level rejects (caught by estimate_gas simulation) ──

#[tokio::test]
async fn parcels_count_zero_returns_blockchain_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "1000000".into(), 0, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::Blockchain(_))),
        "expected Blockchain error for parcels=0, got {result:?}",
    );
}

#[tokio::test]
async fn parcels_count_above_max_returns_blockchain_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "1000000".into(), 13, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::Blockchain(_))),
        "expected Blockchain error for parcels=13, got {result:?}",
    );
}

#[tokio::test]
async fn amount_exceeds_available_balance_returns_blockchain_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    // Amount no test setup could ever fund.
    let huge = U256::from(999_999_999_999_000_000_000u128).to_string();

    let result = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), huge, 6, 30,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::Blockchain(_))),
        "expected Blockchain error for amount > availableBalance, got {result:?}",
    );
}

// ── happy path ────────────────────────────────────────────────

#[tokio::test]
async fn valid_request_returns_bundle_with_calldata_and_gas() {
    let env = common::get_deployed().await;

    // Use second_admin so open-requisition state doesn't accumulate on the
    // founder across parallel test runs. Vinculation is idempotent.
    let _admin2_id = vinculate_second_admin(&env).await;

    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    // Donate from founder to ensure availableBalance >= loan_amount.
    // Hold platform_admin_lock for the entire donate→create→withdraw sequence.
    let _guard        = env.platform_admin_lock.lock().await;
    let donate_amount = U256::from(50_000_000u64); // 50 USDT
    let loan_amount   = U256::from(10_000_000u64); // 10 USDT

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let bundle = create_loan_requisition_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.second_admin),
        loan_amount.to_string(), 6, 30,
    ).await.expect("valid requisition should succeed");

    assert!(bundle.calldata.starts_with("0x"), "calldata should be 0x-prefixed");
    assert!(bundle.calldata.len() > 10,        "calldata should contain encoded arguments");
    assert_eq!(
        bundle.loan_machine_address.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
        "bundle should target the correct contract",
    );
    assert!(
        u64::from_str_radix(bundle.gas_hex.trim_start_matches("0x"), 16).unwrap() > 0,
        "gas estimate should be positive",
    );

    // Restore available balance so other tests are not affected.
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;
}
