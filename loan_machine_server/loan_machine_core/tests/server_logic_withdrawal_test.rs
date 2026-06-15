mod common;

use crate::common::{donate_as, server_returning, vinculate_second_admin, withdraw_as};

use alloy::primitives::U256;
use loan_machine_core::server_logic::coop_financials::get_coop_financials_logic;
use loan_machine_core::server_logic::member_financials::get_member_financials_logic;
use loan_machine_core::server_logic::withdrawal::{withdraw_logic, WithdrawalLogicError};
use loan_machine_core::services::blockchain::BlockchainError;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;
use serde_json::json;

// Subgraph returns empty arrays for every lookup → all values fall through to chain.
fn empty_subgraph() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [], "ReputationChangedEvents": [] } })
}

// ── basic gates ──────────────────────────────────────────────────

#[tokio::test]
async fn non_member_wallet_returns_wallet_not_member_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = withdraw_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        wallet,
        "1000000".to_string(),
    ).await;

    assert!(
        matches!(result, Err(WithdrawalLogicError::WalletNotMember)),
        "expected WalletNotMember, got {:?}", result,
    );
}

#[tokio::test]
async fn invalid_amount_returns_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = withdraw_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        wallet,
        "not-a-number".to_string(),
    ).await;

    assert!(
        matches!(result, Err(WithdrawalLogicError::InvalidAmount(_))),
        "expected InvalidAmount, got {:?}", result,
    );
}

#[tokio::test]
async fn invalid_coop_id_returns_error() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = withdraw_logic(
        &subgraph,
        &env.blockchain,
        "not-bytes32",
        wallet,
        "1000000".to_string(),
    ).await;

    assert!(
        matches!(result, Err(WithdrawalLogicError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {:?}", result,
    );
}

// ── member with no donation ─────────────────────────────────────

#[tokio::test]
async fn member_with_no_donation_cannot_withdraw() {
    let env = common::get_deployed().await;

    // admin2 is vinculated but never donates in this test file.
    vinculate_second_admin(&env).await;
    let wallet = from_alloy(env.second_admin);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    // Sanity check: a member with zero donations has zero withdrawable.
    let financials = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(),
    ).await.expect("vinculated member should return financials");
    assert_eq!(financials.donation,    "0");
    assert_eq!(financials.withdrawable, "0");

    let result = withdraw_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        wallet,
        "1".to_string(),
    ).await;

    assert!(
        matches!(result, Err(WithdrawalLogicError::InsufficientBalance { .. })),
        "expected InsufficientBalance, got {:?}", result,
    );
}

// ── full donate → withdraw cycle on the founder wallet ─────────────
//
// Guarded by `platform_admin_lock` so this test's mutations to admin1's
// donation balance can't interleave with `donation_is_isolated_between_wallets`,
// which also donates/withdraws as admin1. Both tests restore admin1's
// donation balance to its pre-test value before releasing the lock, so
// either ordering leaves the shared fixture in the same state.

#[tokio::test]
async fn donate_then_withdraw_full_cycle_updates_financials() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let _guard = env.platform_admin_lock.lock().await;

    let amount = U256::from(5_000_000u64); // 5 USDT

    let coop_before = get_coop_financials_logic(&env.blockchain, &env.coop_id_hex)
        .await.expect("coop financials before donate");
    let member_before = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(),
    ).await.expect("member financials before donate");

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;

    let coop_after_donate = get_coop_financials_logic(&env.blockchain, &env.coop_id_hex)
        .await.expect("coop financials after donate");
    let member_after_donate = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(),
    ).await.expect("member financials after donate");

    let before_total: u128 = coop_before.total_donations.parse().unwrap();
    let after_donate_total: u128 = coop_after_donate.total_donations.parse().unwrap();
    assert_eq!(after_donate_total, before_total + 5_000_000);

    let before_avail: u128 = coop_before.available_balance.parse().unwrap();
    let after_donate_avail: u128 = coop_after_donate.available_balance.parse().unwrap();
    assert_eq!(after_donate_avail, before_avail + 5_000_000);

    let member_before_withdrawable: u128 = member_before.withdrawable.parse().unwrap();
    let member_after_withdrawable: u128 = member_after_donate.withdrawable.parse().unwrap();
    assert_eq!(member_after_withdrawable, member_before_withdrawable + 5_000_000);

    // withdraw_logic returns a ready-to-sign bundle for the full withdrawable amount.
    let bundle = withdraw_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(), amount.to_string(),
    ).await.expect("member with sufficient balance should get a withdrawal bundle");

    assert_eq!(
        bundle.loan_machine_address.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
    );
    // withdraw(uint256,bytes32) selector
    assert!(bundle.withdraw_calldata.starts_with("0x"));
    assert!(u64::from_str_radix(bundle.gas_withdraw.trim_start_matches("0x"), 16).unwrap() > 0);

    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;

    let coop_after_withdraw = get_coop_financials_logic(&env.blockchain, &env.coop_id_hex)
        .await.expect("coop financials after withdraw");
    let member_after_withdraw = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(),
    ).await.expect("member financials after withdraw");

    let after_withdraw_total: u128 = coop_after_withdraw.total_donations.parse().unwrap();
    let after_withdraw_avail: u128 = coop_after_withdraw.available_balance.parse().unwrap();
    let member_after_withdraw_withdrawable: u128 = member_after_withdraw.withdrawable.parse().unwrap();

    assert_eq!(after_withdraw_total, before_total, "total donations should return to baseline");
    assert_eq!(after_withdraw_avail, before_avail, "available balance should return to baseline");
    assert_eq!(
        member_after_withdraw_withdrawable, member_before_withdrawable,
        "member withdrawable balance should return to baseline",
    );

    // Now the member is back at baseline — withdrawing anything more than
    // that should be rejected again.
    let over = withdraw_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(),
        (member_before_withdrawable + 1).to_string(),
    ).await;
    assert!(
        matches!(over, Err(WithdrawalLogicError::InsufficientBalance { .. })),
        "expected InsufficientBalance, got {:?}", over,
    );
}

// ── cross-wallet isolation ───────────────────────────────────────

#[tokio::test]
async fn donation_is_isolated_between_wallets() {
    let env = common::get_deployed().await;

    // Vinculation acquires/releases `second_admin_lock` internally and must
    // happen before we take `platform_admin_lock` (documented lock order in
    // tests/common/elections.rs: platform_admin_lock > second_admin_lock).
    let admin2_id = vinculate_second_admin(&env).await;

    let admin1_wallet = from_alloy(env.approved_wallet);
    let admin2_wallet = from_alloy(env.second_admin);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    // admin2 never donates in this test file, so its balance is always zero.
    let admin2_before = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, admin2_wallet.clone(),
    ).await.expect("admin2 financials before");
    assert_eq!(admin2_before.donation,    "0");
    assert_eq!(admin2_before.withdrawable, "0");

    let _guard = env.platform_admin_lock.lock().await;

    let amount = U256::from(3_000_000u64); // 3 USDT
    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;

    // admin1's donation is visible on admin1's own financials...
    let admin1_after = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, admin1_wallet.clone(),
    ).await.expect("admin1 financials after donate");
    let admin1_withdrawable: u128 = admin1_after.withdrawable.parse().unwrap();
    assert!(admin1_withdrawable >= 3_000_000);

    // ...but is NOT visible on admin2's financials, and admin2 still cannot
    // withdraw any of it.
    let admin2_after = get_member_financials_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, admin2_wallet.clone(),
    ).await.expect("admin2 financials after admin1 donate");
    assert_eq!(admin2_after.donation,    "0");
    assert_eq!(admin2_after.withdrawable, "0");

    let admin2_withdraw = withdraw_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, admin2_wallet.clone(), amount.to_string(),
    ).await;
    assert!(
        matches!(admin2_withdraw, Err(WithdrawalLogicError::InsufficientBalance { .. })),
        "expected admin2's withdraw to be blocked, got {:?}", admin2_withdraw,
    );

    // cleanup — restore admin1's donation balance to its pre-test value.
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;

    let _ = admin2_id; // returned by vinculate_second_admin; not otherwise needed here
}

// ── extreme amounts ──────────────────────────────────────────────

#[tokio::test]
async fn zero_amount_withdraw_reverts_with_invalid_amount() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let _guard = env.platform_admin_lock.lock().await;

    let amount = U256::from(1_000_000u64); // 1 USDT
    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;

    let result = withdraw_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(), "0".to_string(),
    ).await;

    match result {
        Err(WithdrawalLogicError::Blockchain(BlockchainError::ContractRevert { message, .. })) => {
            assert_eq!(message, "Invalid amount.");
        }
        other => panic!("expected ContractRevert(\"Invalid amount.\"), got {:?}", other),
    }

    // cleanup — restore admin1's donation balance to its pre-test value.
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;
}

#[tokio::test]
async fn withdraw_one_unit_above_withdrawable_returns_insufficient_balance() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(empty_subgraph()).await;
    let subgraph = SubgraphService::new(server.uri());

    let _guard = env.platform_admin_lock.lock().await;

    let amount = U256::from(2_000_000u64); // 2 USDT
    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;

    let result = withdraw_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, wallet.clone(), "2000001".to_string(),
    ).await;

    match result {
        Err(WithdrawalLogicError::InsufficientBalance { available, requested }) => {
            assert_eq!(available, "2000000");
            assert_eq!(requested, "2000001");
        }
        other => panic!("expected InsufficientBalance, got {:?}", other),
    }

    // cleanup — restore admin1's donation balance to its pre-test value.
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, amount).await;
}
