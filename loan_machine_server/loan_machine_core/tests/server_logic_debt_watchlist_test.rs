mod common;

use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use serde_json::json;

use common::{
    donate_as, mint_usdt, create_requisition_as, cover_as, repay_as,
    vinculate_second_admin, admin2_member_id,
    server_returning,
};
use loan_machine_core::server_logic::coop_financials::{get_debt_watchlist_logic, CoopFinancialsError};
use loan_machine_core::server_logic::loan_requisition::fetch_my_active_loans_logic;
use loan_machine_core::services::blockchain::deployable::LoanMachine;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

// ── helpers ───────────────────────────────────────────────────

fn empty_subgraph() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [], "ReputationChangedEvents": [] } })
}

/// Advances Anvil's block clock by `seconds` and mines a block to commit it.
/// Must be called while holding `platform_admin_lock`.
async fn advance_seconds(rpc_url: &str, seconds: u64) {
    use alloy::providers::Provider as _;
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());
    let _: serde_json::Value = provider
        .raw_request("evm_increaseTime".into(), (seconds,))
        .await
        .expect("evm_increaseTime");
    let _: serde_json::Value = provider
        .raw_request("evm_mine".into(), ((),))
        .await
        .expect("evm_mine");
}

async fn advance_past_borrow_duration(rpc_url: &str) {
    advance_seconds(rpc_url, 31 * 24 * 3600).await;
}

/// Creates an active loan for second_admin and returns `(req_id, parcel_value)`.
/// Advances past `BORROW_DURATION` (30 days) before creating each loan so that
/// back-to-back calls from the same wallet are never blocked.
async fn make_active_loan(
    env:           &common::deploy::DeployedEnv,
    loan_amount:   U256,
    parcels_count: u32,
    days_interval: u32,
) -> (U256, U256) {
    let admin2_id = vinculate_second_admin(env).await;
    let _guard = env.platform_admin_lock.lock().await;

    advance_past_borrow_duration(&env.rpc_url).await;

    let pool_funds = loan_amount * U256::from(2);
    mint_usdt(env, &env.platform_admin_key_hex, pool_funds).await;
    donate_as(env, &env.platform_admin_key_hex, env.founder_member_id, pool_funds).await;

    let req_id = create_requisition_as(
        env, &env.second_admin_key_hex, admin2_id,
        loan_amount, parcels_count, days_interval,
    ).await;
    cover_as(env, &env.platform_admin_key_hex, env.founder_member_id, req_id, 100).await;

    parcel_value_for(env, req_id).await
}

async fn parcel_value_for(env: &common::deploy::DeployedEnv, req_id: U256) -> (U256, U256) {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let parcel_value = LoanMachine::new(lm_addr, provider)
        .getNextPaymentAmount(req_id)
        .call().await
        .expect("getNextPaymentAmount")
        .paymentAmount;
    (req_id, parcel_value)
}

/// Repay one parcel for second_admin, serialized under `second_admin_lock`
/// to prevent concurrent `approve()` calls from clobbering the allowance.
async fn repay_second_admin(
    env:           &common::deploy::DeployedEnv,
    req_id:        U256,
    parcel_value:  U256,
) {
    let _guard = env.second_admin_lock.lock().await;
    repay_as(env, &env.second_admin_key_hex, admin2_member_id(), req_id, parcel_value).await;
}

// ─────────────────────────────────────────────────────────────
// get_debt_watchlist_logic — invalid coop id
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn watchlist_invalid_coop_id_returns_error() {
    let env = common::get_deployed().await;

    let result = get_debt_watchlist_logic(&env.blockchain, "not-bytes32").await;

    assert!(
        matches!(result, Err(CoopFinancialsError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {result:?}",
    );
}

// ─────────────────────────────────────────────────────────────
// fresh loan: not yet past due → not in watchlist
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn fresh_loan_not_in_watchlist_before_due_date() {
    let env = common::get_deployed().await;
    let (req_id, _) = make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist should succeed");

    let found = items.iter().any(|i| i.requisition_id == req_id.to_string());
    assert!(
        !found,
        "fresh loan (nextDue +30 days from now) must not appear in overdue watchlist",
    );
}

// ─────────────────────────────────────────────────────────────
// loan past first due date → appears in watchlist
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn overdue_loan_appears_in_watchlist_after_passing_due_date() {
    let env = common::get_deployed().await;
    let (req_id, _) = make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;

    {
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 31 * 24 * 3600).await;
    }

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist should succeed");

    let entry = items.iter()
        .find(|i| i.requisition_id == req_id.to_string())
        .expect("overdue loan must appear in watchlist");

    assert!(entry.is_overdue);
    assert_eq!(
        entry.borrower.to_lowercase(),
        format!("{:#x}", env.second_admin).to_lowercase(),
        "borrower must match second_admin",
    );
}

// ─────────────────────────────────────────────────────────────
// watchlist entry: due date field is in the past
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn watchlist_entry_due_date_is_strictly_before_chain_time() {
    let env = common::get_deployed().await;
    let (req_id, _) = make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;

    let chain_time: u64;
    {
        use alloy::providers::Provider as _;
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 31 * 24 * 3600).await;

        let provider = ProviderBuilder::new().on_http(env.rpc_url.parse().unwrap());
        let val: serde_json::Value = provider
            .raw_request("eth_getBlockByNumber".into(), ("latest", false))
            .await
            .unwrap_or_default();
        chain_time = val["timestamp"]
            .as_str()
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(0);
    }

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist should succeed");

    let entry = items.iter()
        .find(|i| i.requisition_id == req_id.to_string())
        .expect("overdue loan must be in watchlist");

    assert!(
        entry.next_due_date < chain_time,
        "due date {} must be before chain time {} for an overdue entry",
        entry.next_due_date, chain_time,
    );
}

// ─────────────────────────────────────────────────────────────
// paying all parcels on time removes loan from watchlist
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn paying_all_parcels_on_time_removes_loan_from_watchlist() {
    let env = common::get_deployed().await;
    let parcels_count = 3u32;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(9_000_000u64), parcels_count, 30).await;

    for _ in 0..parcels_count {
        repay_second_admin(&env, req_id, parcel_value).await;
    }

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist should succeed");

    assert!(
        !items.iter().any(|i| i.requisition_id == req_id.to_string()),
        "fully-repaid loan must be removed from watchlist",
    );
}

// ─────────────────────────────────────────────────────────────
// paying all parcels late (overdue) removes loan from watchlist
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn paying_all_parcels_while_overdue_removes_loan_from_watchlist() {
    let env = common::get_deployed().await;
    let parcels_count = 2u32;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(6_000_000u64), parcels_count, 30).await;

    {
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 31 * 24 * 3600).await;
    }

    {
        let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
            .await
            .expect("watchlist before repay");
        assert!(
            items.iter().any(|i| i.requisition_id == req_id.to_string()),
            "sanity: loan must appear in watchlist before repayment",
        );
    }

    for _ in 0..parcels_count {
        repay_second_admin(&env, req_id, parcel_value).await;
    }

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist after repay");

    assert!(
        !items.iter().any(|i| i.requisition_id == req_id.to_string()),
        "fully-repaid overdue loan must leave watchlist",
    );
}

// ─────────────────────────────────────────────────────────────
// partial late payment: loan stays in watchlist while parcels remain
//
// Advance 95 days so ALL three payment windows expire before any
// repayment.  After paying parcel 1 the contract sets nextDueDate to
// payment_dates[1], which is still in the past — so the server-side
// `now > next_due_date` check keeps the loan in the overdue list.
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn partial_late_payment_loan_stays_in_watchlist() {
    let env = common::get_deployed().await;
    let parcels_count = 3u32;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(9_000_000u64), parcels_count, 30).await;

    {
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 95 * 24 * 3600).await;
    }

    // Pay only the first parcel late.
    repay_second_admin(&env, req_id, parcel_value).await;

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist after partial late repay");

    assert!(
        items.iter().any(|i| i.requisition_id == req_id.to_string()),
        "loan with remaining overdue parcels must stay in watchlist after partial payment",
    );
}

// ─────────────────────────────────────────────────────────────
// two consecutive loans: only the overdue one appears
//
// Two back-to-back calls to make_active_loan advance the clock by 62
// days total.  Loan A (created at T+31) has nextDue = T+61 which is 1
// day in the past.  Loan B (created at T+62) has nextDue = T+92 which
// is 30 days in the future.
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn watchlist_shows_only_overdue_loan_not_current_one() {
    let env = common::get_deployed().await;

    let (req_id_a, _) = make_active_loan(&env, U256::from(6_000_000u64), 1, 30).await;
    // Second call advances another 31 days, making loan A's window expire.
    let (req_id_b, _) = make_active_loan(&env, U256::from(6_000_000u64), 1, 30).await;

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await
        .expect("watchlist");

    assert!(
        items.iter().any(|i| i.requisition_id == req_id_a.to_string()),
        "loan A (1 day overdue) must be in watchlist",
    );
    assert!(
        !items.iter().any(|i| i.requisition_id == req_id_b.to_string()),
        "loan B (due in 30 days) must not be in watchlist",
    );
}

// ─────────────────────────────────────────────────────────────
// fetch_my_active_loans_logic — is_overdue flag
//
// Both checks run under platform_admin_lock to prevent concurrent time
// advances from causing false positives on the "not overdue" assertion.
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn is_overdue_flag_transitions_correctly_around_due_date() {
    let env = common::get_deployed().await;
    let admin2_id = vinculate_second_admin(&env).await;

    let req_id: U256;
    let parcel_value: U256;

    {
        let _guard = env.platform_admin_lock.lock().await;

        // Create loan under the lock so no concurrent time advance can expire
        // the payment window before we check is_overdue = false.
        advance_past_borrow_duration(&env.rpc_url).await;

        let loan_amount = U256::from(6_000_000u64);
        let pool_funds  = loan_amount * U256::from(2);
        mint_usdt(&env, &env.platform_admin_key_hex, pool_funds).await;
        donate_as(&env, &env.platform_admin_key_hex, env.founder_member_id, pool_funds).await;

        let id = create_requisition_as(
            &env, &env.second_admin_key_hex, admin2_id,
            loan_amount, 2, 30,
        ).await;
        cover_as(&env, &env.platform_admin_key_hex, env.founder_member_id, id, 100).await;

        (req_id, parcel_value) = parcel_value_for(&env, id).await;

        // ── check 1: not overdue immediately after creation ──────────
        let server = server_returning(empty_subgraph()).await;
        let sg     = SubgraphService::new(server.uri());
        let items  = fetch_my_active_loans_logic(
            &sg, &env.blockchain, &env.coop_id_hex, &from_alloy(env.second_admin),
        ).await.expect("fetch before time jump");

        let item = items.iter()
            .find(|i| i.requisition_id == req_id.to_string())
            .expect("loan must appear in active list");
        assert!(!item.is_overdue, "loan just created must not be overdue (nextDue is 30 days away)");

        // ── advance past first due date ───────────────────────────────
        advance_seconds(&env.rpc_url, 31 * 24 * 3600).await;

        // ── check 2: overdue after time jump ──────────────────────────
        let server2 = server_returning(empty_subgraph()).await;
        let sg2     = SubgraphService::new(server2.uri());
        let items2  = fetch_my_active_loans_logic(
            &sg2, &env.blockchain, &env.coop_id_hex, &from_alloy(env.second_admin),
        ).await.expect("fetch after time jump");

        let item2 = items2.iter()
            .find(|i| i.requisition_id == req_id.to_string())
            .expect("overdue loan must still be in active list");
        assert!(item2.is_overdue, "loan must be overdue after its first due date passes");
    }

    // Clean up: repay all parcels outside the lock.
    for _ in 0..2u32 {
        repay_second_admin(&env, req_id, parcel_value).await;
    }
}

#[tokio::test]
async fn loan_not_in_active_list_after_all_parcels_repaid() {
    let env = common::get_deployed().await;
    let parcels_count = 2u32;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(6_000_000u64), parcels_count, 30).await;

    for _ in 0..parcels_count {
        repay_second_admin(&env, req_id, parcel_value).await;
    }

    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let items = fetch_my_active_loans_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.second_admin),
    ).await.expect("fetch should succeed");

    assert!(
        !items.iter().any(|i| i.requisition_id == req_id.to_string()),
        "fully-repaid loan must not appear in active loans list",
    );
}

// ─────────────────────────────────────────────────────────────
// extreme: single-parcel loan paid late
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn single_parcel_loan_paid_late_succeeds_and_clears_watchlist() {
    let env = common::get_deployed().await;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(3_000_000u64), 1, 30).await;

    {
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 31 * 24 * 3600).await;
    }

    {
        let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
            .await.expect("watchlist before payment");
        assert!(
            items.iter().any(|i| i.requisition_id == req_id.to_string()),
            "single-parcel loan must be in watchlist before late payment",
        );
    }

    repay_second_admin(&env, req_id, parcel_value).await;

    let after = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await.expect("watchlist after payment");
    assert!(
        !after.iter().any(|i| i.requisition_id == req_id.to_string()),
        "single-parcel loan must leave watchlist after late payment",
    );
}

// ─────────────────────────────────────────────────────────────
// extreme: indivisible loan (7 USDT / 3 parcels) fully repaid late
//
// Verifies the _fundLoan rounding fix: parcelsValues = floor(7M/3) = 2 333 333,
// and all 3 repayments succeed, clearing the loan from the watchlist.
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn indivisible_loan_fully_repaid_late_clears_watchlist() {
    let env = common::get_deployed().await;
    let parcels_count = 3u32;
    let loan_amount   = U256::from(7_000_000u64);

    let (req_id, parcel_value) =
        make_active_loan(&env, loan_amount, parcels_count, 30).await;

    assert_eq!(
        parcel_value,
        U256::from(2_333_333u64),
        "parcel value must be floor(7_000_000 / 3)",
    );

    {
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 31 * 24 * 3600).await;
    }

    // All 3 repayments of 2 333 333 must succeed (no "Invalid amount" revert).
    for _ in 0..parcels_count {
        repay_second_admin(&env, req_id, parcel_value).await;
    }

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await.expect("watchlist");

    assert!(
        !items.iter().any(|i| i.requisition_id == req_id.to_string()),
        "indivisible-amount loan must be removed from watchlist after all parcels repaid",
    );
}

// ─────────────────────────────────────────────────────────────
// extreme: 12-parcel loan, all windows expired, fully repaid late
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn twelve_parcel_loan_fully_paid_late_clears_watchlist() {
    let env = common::get_deployed().await;
    let parcels_count = 12u32;
    let loan_amount   = U256::from(12_000_000u64); // 1 USDT per parcel, exact division

    let (req_id, parcel_value) =
        make_active_loan(&env, loan_amount, parcels_count, 30).await;

    {
        // 13 months: all 12 payment windows (30, 60, …, 360 days) expire.
        let _guard = env.platform_admin_lock.lock().await;
        advance_seconds(&env.rpc_url, 395 * 24 * 3600).await;
    }

    for _ in 0..parcels_count {
        repay_second_admin(&env, req_id, parcel_value).await;
    }

    let items = get_debt_watchlist_logic(&env.blockchain, &env.coop_id_hex)
        .await.expect("watchlist");

    assert!(
        !items.iter().any(|i| i.requisition_id == req_id.to_string()),
        "12-parcel loan must be gone from watchlist after all parcels repaid late",
    );
}
