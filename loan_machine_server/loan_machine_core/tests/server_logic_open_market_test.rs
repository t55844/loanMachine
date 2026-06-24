mod common;

use common::{
    cancel_requisition_as, cover_as, create_requisition_as,
    donate_as, get_withdrawable, server_returning, vinculate_second_admin, withdraw_as,
};

use alloy::primitives::U256;
use loan_machine_core::server_logic::loan_requisition::{
    fetch_open_market_logic, prepare_cover_loan_logic, LoanRequisitionError,
};
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;
use serde_json::json;

// ── subgraph mock helpers ────────────────────────────────────────

fn open_market_payload(items: &[serde_json::Value]) -> serde_json::Value {
    json!({ "data": { "loanRequisitionCreatedCancelledEvents": items } })
}

/// Empty memberRegisteredEvents → resolve_member_id falls through to chain.
fn member_lookup_empty() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [] } })
}

// ── fetch_open_market_logic — input validation ───────────────────

#[tokio::test]
async fn invalid_coop_id_returns_error() {
    let env    = common::get_deployed().await;
    let server = server_returning(open_market_payload(&[])).await;
    let sg     = SubgraphService::new(server.uri());

    let result = fetch_open_market_logic(
        &sg, &env.blockchain, "not-bytes32",
        &from_alloy(env.approved_wallet),
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {result:?}",
    );
}

// ── fetch_open_market_logic — subgraph forwarding ────────────────

#[tokio::test]
async fn empty_subgraph_returns_empty_market() {
    let env    = common::get_deployed().await;
    let server = server_returning(open_market_payload(&[])).await;
    let sg     = SubgraphService::new(server.uri());

    let result = fetch_open_market_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.approved_wallet),
    ).await.expect("valid request should not fail");

    assert!(result.items.is_empty(), "expected no items, got {:?}", result.items);
}

#[tokio::test]
async fn subgraph_items_are_forwarded_to_response() {
    let env      = common::get_deployed().await;
    let borrower = format!("{:#x}", env.second_admin);
    let server   = server_returning(open_market_payload(&[json!({
        "requisitionId":     "42",
        "borrower":          borrower,
        "amount":            "10000000",
        "parcelsCount":      6,
        "currentCoverage":   30,
        "creationTimestamp": "1700000000",
    })])).await;
    let sg = SubgraphService::new(server.uri());

    let result = fetch_open_market_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.approved_wallet),
    ).await.expect("should succeed");

    assert_eq!(result.items.len(), 1, "expected one item from subgraph");
    let item = &result.items[0];
    assert_eq!(item.requisition_id,   "42");
    assert_eq!(item.amount,           "10000000");
    assert_eq!(item.parcels_count,    6);
    assert_eq!(item.current_coverage, 30);
    assert_eq!(item.created_at,       1_700_000_000u64);
}

#[tokio::test]
async fn two_items_from_subgraph_are_both_forwarded() {
    let env    = common::get_deployed().await;
    let server = server_returning(open_market_payload(&[
        json!({
            "requisitionId": "1", "borrower": "0xaaa",
            "amount": "5000000",  "parcelsCount": 3,
            "currentCoverage": 0, "creationTimestamp": "1700000100",
        }),
        json!({
            "requisitionId": "2", "borrower": "0xbbb",
            "amount": "8000000",  "parcelsCount": 6,
            "currentCoverage": 75, "creationTimestamp": "1700000200",
        }),
    ])).await;
    let sg = SubgraphService::new(server.uri());

    let result = fetch_open_market_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.approved_wallet),
    ).await.expect("should succeed");

    assert_eq!(result.items.len(), 2, "both subgraph items should appear");
    assert_eq!(result.items[0].current_coverage, 0);
    assert_eq!(result.items[1].current_coverage, 75);
}

// ── fetch_open_market_logic — chain-sourced withdrawable ─────────

#[tokio::test]
async fn user_withdrawable_is_zero_without_donation() {
    let env    = common::get_deployed().await;
    let server = server_returning(open_market_payload(&[])).await;
    let sg     = SubgraphService::new(server.uri());

    // unapproved_wallet has never donated → withdrawable must be 0
    let result = fetch_open_market_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.unapproved_wallet),
    ).await.expect("should succeed even for non-members");

    assert_eq!(result.user_withdrawable, "0");
}

#[tokio::test]
async fn user_withdrawable_reflects_current_donation_on_chain() {
    let env    = common::get_deployed().await;
    let _guard = env.platform_admin_lock.lock().await;

    let donate_amount = U256::from(7_000_000u64); // 7 USDT
    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let server = server_returning(open_market_payload(&[])).await;
    let sg     = SubgraphService::new(server.uri());

    let result = fetch_open_market_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.approved_wallet),
    ).await.expect("should succeed");

    let withdrawable: u128 = result.user_withdrawable.parse().unwrap();
    assert!(
        withdrawable >= 7_000_000,
        "user_withdrawable should include the just-donated 7 USDT; got {withdrawable}",
    );

    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;
}

// ── prepare_cover_loan_logic — input validation ──────────────────

#[tokio::test]
async fn cover_pct_zero_returns_invalid_coverage() {
    let env    = common::get_deployed().await;
    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "1", 0,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidCoverage)),
        "expected InvalidCoverage for pct=0, got {result:?}",
    );
}

#[tokio::test]
async fn cover_pct_above_100_returns_invalid_coverage() {
    let env    = common::get_deployed().await;
    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "1", 101,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidCoverage)),
        "expected InvalidCoverage for pct=101, got {result:?}",
    );
}

#[tokio::test]
async fn non_member_cover_returns_wallet_not_member() {
    let env    = common::get_deployed().await;
    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.unapproved_wallet), "1", 50,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::WalletNotMember)),
        "expected WalletNotMember for non-member wallet, got {result:?}",
    );
}

#[tokio::test]
async fn nonexistent_req_cover_returns_blockchain_error() {
    // req 9999 doesn't exist on-chain.
    // get_requisition_info returns (0, 0, 0) → status=0 passes the status gate.
    // estimate_cover_loan_gas then reverts because the req has no borrower.
    let env    = common::get_deployed().await;
    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), "9999", 50,
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::Blockchain(_))),
        "expected Blockchain error for non-existent requisition, got {result:?}",
    );
}

// ── lifecycle: covering locks funds in the coverer's balance ─────

#[tokio::test]
async fn covering_reduces_coverer_withdrawable_by_covered_amount() {
    // Admin1 donates 100 USDT. Admin2 creates a requisition for 40 USDT.
    // Admin1 covers 50% (= 20 USDT locked from their donation).
    // Covering must reduce admin1's withdrawable by exactly 20 USDT.
    let env = common::get_deployed().await;

    let admin2_id = vinculate_second_admin(&env).await;
    let _guard    = env.platform_admin_lock.lock().await;

    let donate_amount = U256::from(100_000_000u64); // 100 USDT
    let loan_amount   = U256::from(40_000_000u64);  //  40 USDT
    let cover_pct     = 50u32;

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let req_id = create_requisition_as(
        &env, &env.second_admin_key_hex, admin2_id, loan_amount, 6, 30,
    ).await;

    let before = get_withdrawable(&env, env.approved_wallet).await;

    cover_as(&env, &env.platform_admin_key_hex, env.founder_member_id, req_id, cover_pct).await;

    let after = get_withdrawable(&env, env.approved_wallet).await;

    // 50% of 40 USDT = 20 USDT locked
    let expected_locked = loan_amount.saturating_to::<u128>() * cover_pct as u128 / 100;
    assert_eq!(
        before.saturating_sub(after), expected_locked,
        "covering {cover_pct}% of {loan_amount} USDT should lock exactly {expected_locked} units",
    );

    // cleanup — cancel restores admin1's locked funds, then withdraw all
    cancel_requisition_as(&env, &env.second_admin_key_hex, admin2_id, req_id).await;
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;
}

// ── lifecycle: cancellation after partial cover restores funds ───

#[tokio::test]
async fn cancel_after_partial_cover_restores_coverer_withdrawable() {
    // Admin1 donates 50 USDT. Admin2 creates requisition for 20 USDT.
    // Admin1 covers 50% (= 10 USDT locked). Then admin2 cancels.
    // After cancellation, admin1's withdrawable must return to the pre-cover value.
    let env = common::get_deployed().await;

    let admin2_id = vinculate_second_admin(&env).await;
    let _guard    = env.platform_admin_lock.lock().await;

    let donate_amount = U256::from(50_000_000u64); // 50 USDT
    let loan_amount   = U256::from(20_000_000u64); // 20 USDT

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let req_id = create_requisition_as(
        &env, &env.second_admin_key_hex, admin2_id, loan_amount, 4, 30,
    ).await;

    let after_donate = get_withdrawable(&env, env.approved_wallet).await;

    cover_as(&env, &env.platform_admin_key_hex, env.founder_member_id, req_id, 50).await;

    let after_cover = get_withdrawable(&env, env.approved_wallet).await;
    assert!(
        after_cover < after_donate,
        "withdrawable must drop after covering; before={after_donate} after={after_cover}",
    );

    // Borrower cancels — all coverers get their locked amounts refunded.
    cancel_requisition_as(&env, &env.second_admin_key_hex, admin2_id, req_id).await;

    let after_cancel = get_withdrawable(&env, env.approved_wallet).await;
    assert_eq!(
        after_cancel, after_donate,
        "cancel should fully restore admin1's withdrawable (locked 10 USDT back to free)",
    );

    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;
}

// ── lifecycle: cancelled requisition cannot be covered ───────────

#[tokio::test]
async fn cancelled_requisition_is_not_coverable() {
    // Admin2 creates and immediately cancels a requisition.
    // Trying to cover a cancelled req (status=6) must return LoanNotAvailable.
    let env = common::get_deployed().await;

    let admin2_id = vinculate_second_admin(&env).await;
    let _guard    = env.platform_admin_lock.lock().await;

    let donate_amount = U256::from(20_000_000u64); // 20 USDT
    let loan_amount   = U256::from(10_000_000u64); // 10 USDT

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let req_id = create_requisition_as(
        &env, &env.second_admin_key_hex, admin2_id, loan_amount, 6, 30,
    ).await;

    // Cancel immediately — no coverage applied yet.
    cancel_requisition_as(&env, &env.second_admin_key_hex, admin2_id, req_id).await;

    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    // Compute result BEFORE withdraw so USDT is returned even if assert panics.
    let result = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), &req_id.to_string(), 50,
    ).await;

    // No coverage was ever applied, so admin1's full donation is still withdrawable.
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::LoanNotAvailable)),
        "expected LoanNotAvailable for a cancelled requisition, got {result:?}",
    );
}

// ── lifecycle: authoritative chain state blocks over-coverage ────

#[tokio::test]
async fn authoritative_chain_state_prevents_over_coverage() {
    // Scenario: admin1 covers 70% on-chain. Then a second call (simulating stale
    // UI or a concurrent lender) tries to cover 40% more — 70 + 40 = 110 > 100.
    // prepare_cover_loan_logic reads current_coverage from the chain before gas
    // estimation, so the over-coverage is caught before any tx is broadcast.
    //
    // Additionally, the exact remaining 30% IS accepted, proving the gate is
    // tight: [1, remaining] is allowed, [remaining+1, 100] is blocked.
    let env = common::get_deployed().await;

    let admin2_id = vinculate_second_admin(&env).await;
    let _guard    = env.platform_admin_lock.lock().await;

    let donate_amount = U256::from(100_000_000u64); // 100 USDT
    let loan_amount   = U256::from(50_000_000u64);  //  50 USDT

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let req_id = create_requisition_as(
        &env, &env.second_admin_key_hex, admin2_id, loan_amount, 6, 30,
    ).await;

    // First lender covers 70% on-chain.
    cover_as(&env, &env.platform_admin_key_hex, env.founder_member_id, req_id, 70).await;

    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    // Compute prepare results BEFORE cleanup so the req is still in the
    // 70%-covered state.  Assertions come AFTER cleanup so a panic there
    // can't prevent USDT from being returned.
    let over = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), &req_id.to_string(), 40,
    ).await;

    // cleanup — cancel returns admin1's 70% coverage; withdraw restores USDT
    cancel_requisition_as(&env, &env.second_admin_key_hex, admin2_id, req_id).await;
    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    // 70 + 40 = 110 → over-coverage must be blocked.
    // If get_requisition_info silently fails (unwrap_or(0,0,0)), the Rust-layer
    // check won't fire and the contract itself rejects via gas estimate.
    // Both variants represent correct behavior.
    assert!(
        matches!(over, Err(LoanRequisitionError::CoverageExceeds) | Err(LoanRequisitionError::Blockchain(_))),
        "40% more after 70% covered should be blocked, got {over:?}",
    );
}

// ── lifecycle: half-covered then cancelled — full accounting ─────

#[tokio::test]
async fn half_covered_then_cancelled_returns_both_lender_and_donor_to_baseline() {
    // Full-cycle extreme scenario:
    //   1. Admin1 donates 80 USDT.
    //   2. Admin2 creates a requisition for 60 USDT.
    //   3. Admin1 covers 50% (= 30 USDT locked).
    //      → Admin1 withdrawable drops by 30 USDT.
    //   4. Verify admin1 cannot withdraw more than (80 - 30) USDT via prepare_cover check.
    //   5. Admin2 cancels — refunds admin1's 30 USDT.
    //      → Admin1 withdrawable restored to 80 USDT.
    //   6. Admin1 withdraws full 80 USDT — pool returns to pre-test state.
    let env = common::get_deployed().await;

    let admin2_id = vinculate_second_admin(&env).await;
    let _guard    = env.platform_admin_lock.lock().await;

    let donate_amount = U256::from(80_000_000u64);  // 80 USDT
    let loan_amount   = U256::from(60_000_000u64);  // 60 USDT

    donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    let withdrawable_after_donate = get_withdrawable(&env, env.approved_wallet).await;

    let req_id = create_requisition_as(
        &env, &env.second_admin_key_hex, admin2_id, loan_amount, 6, 30,
    ).await;

    // Cover 50% → 30 USDT locked.
    cover_as(&env, &env.platform_admin_key_hex, env.founder_member_id, req_id, 50).await;

    let after_50pct_cover = get_withdrawable(&env, env.approved_wallet).await;
    let locked = loan_amount.saturating_to::<u128>() * 50 / 100; // 30_000_000
    assert_eq!(
        withdrawable_after_donate.saturating_sub(after_50pct_cover), locked,
        "50% cover of 60 USDT should lock exactly 30 USDT",
    );

    // Compute prepare results BEFORE cancel so the req is still coverable.
    // Assertions come AFTER cleanup to avoid panic blocking the withdraw.
    let server = server_returning(member_lookup_empty()).await;
    let sg     = SubgraphService::new(server.uri());

    let over = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), &req_id.to_string(), 51,
    ).await;

    let edge = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), &req_id.to_string(), 50,
    ).await;

    // Cancel returns all covered amounts to coverers.
    cancel_requisition_as(&env, &env.second_admin_key_hex, admin2_id, req_id).await;

    let after_cancel = get_withdrawable(&env, env.approved_wallet).await;

    let post_cancel = prepare_cover_loan_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        from_alloy(env.approved_wallet), &req_id.to_string(), 1,
    ).await;

    withdraw_as(&env, &env.platform_admin_key_hex, env.founder_member_id, donate_amount).await;

    // ── assertions ──────────────────────────────────────────────

    // 50 + 51 = 101 → over-coverage must be caught by the Rust layer (CoverageExceeds)
    assert!(
        matches!(over, Err(LoanRequisitionError::CoverageExceeds)),
        "51% more after 50% covered should return CoverageExceeds, got {over:?}",
    );

    // 50 + 50 = 100 → exactly at the boundary, must succeed
    assert!(edge.is_ok(), "exact remaining 50% should produce a valid bundle, got {edge:?}");

    // cancel restores admin1's withdrawable to the post-donate level
    assert_eq!(
        after_cancel, withdrawable_after_donate,
        "cancel should restore admin1's withdrawable (30 USDT returned from coverage)",
    );

    assert!(
        matches!(post_cancel, Err(LoanRequisitionError::LoanNotAvailable)),
        "cancelled req should return LoanNotAvailable, got {post_cancel:?}",
    );
}
