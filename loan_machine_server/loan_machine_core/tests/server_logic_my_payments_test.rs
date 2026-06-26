mod common;

use std::str::FromStr;

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use serde_json::json;

use common::{
    donate_as, mint_usdt, create_requisition_as, cover_as,
    vinculate_second_admin, admin2_member_id, repay_as, server_returning,
};
use loan_machine_core::server_logic::loan_requisition::{
    fetch_my_active_loans_logic, prepare_repayment_logic, LoanRequisitionError,
};
use loan_machine_core::services::blockchain::deployable::{LoanMachine, MockUSDT};
use loan_machine_core::services::blockchain::BlockchainError;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

// ── helpers ───────────────────────────────────────────────────

fn empty_subgraph() -> serde_json::Value {
    json!({ "data": { "memberRegisteredEvents": [], "ReputationChangedEvents": [] } })
}

fn expect_revert(err: BlockchainError) -> String {
    match err {
        BlockchainError::ContractRevert { message, .. } => message,
        other => panic!("expected ContractRevert, got {other:?}"),
    }
}

/// Advances Anvil's block clock by 31 days and mines an empty block so that
/// subsequent `eth_call` simulations see the new timestamp.
///
/// Must be called under `platform_admin_lock` to keep time advances ordered
/// across parallel tests.
async fn advance_past_borrow_duration(rpc_url: &str) {
    use alloy::providers::Provider as _;
    let provider = ProviderBuilder::new()
        .on_http(rpc_url.parse().unwrap());
    let seconds: u64 = 31 * 24 * 3600;
    // evm_increaseTime sets the offset for future blocks.
    let _: serde_json::Value = provider
        .raw_request("evm_increaseTime".into(), (seconds,))
        .await
        .expect("evm_increaseTime");
    // Mine an empty block so eth_call simulations see the updated timestamp.
    let _: serde_json::Value = provider
        .raw_request("evm_mine".into(), ((),))
        .await
        .expect("evm_mine");
}

/// Sets up a fully-covered Active loan.
///
/// * Lender: approved_wallet (founder) via `platform_admin_key_hex`
/// * Borrower: second_admin (admin2) via `second_admin_key_hex`
///
/// After this call the borrower holds USDT (received from `_fundLoan`) and
/// the loan status is Active. Returns `(req_id, parcel_value)` where
/// `parcel_value = loan_amount / parcels_count` (exact integer division).
///
/// Advances Anvil's clock by 31 days before creating each requisition so the
/// `BORROW_DURATION` check (30 days) never blocks second_admin from borrowing
/// again, even when tests run sequentially under `platform_admin_lock`.
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

// ─────────────────────────────────────────────────────────────
// fetch_my_active_loans_logic
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn fetch_active_loans_invalid_coop_id() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = fetch_my_active_loans_logic(
        &sg, &env.blockchain, "not-bytes32",
        &from_alloy(env.approved_wallet),
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {result:?}",
    );
}

#[tokio::test]
async fn fetch_active_loans_wallet_with_no_loans_returns_empty() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = fetch_my_active_loans_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.unapproved_wallet),
    ).await;

    assert!(
        matches!(result, Ok(ref v) if v.is_empty()),
        "expected empty vec for wallet with no loans, got {result:?}",
    );
}

#[tokio::test]
async fn fetch_active_loans_returns_correct_parcel_data() {
    let env = common::get_deployed().await;

    let loan_amount   = U256::from(9_000_000u64); // 9 USDT, 3 × 3 USDT
    let parcels_count = 3u32;

    let (req_id, parcel_value) =
        make_active_loan(&env, loan_amount, parcels_count, 30).await;

    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let items = fetch_my_active_loans_logic(
        &sg, &env.blockchain, &env.coop_id_hex,
        &from_alloy(env.second_admin),
    ).await.expect("fetch should succeed");

    let item = items.iter()
        .find(|i| i.requisition_id == req_id.to_string())
        .expect("active loan should appear in result");

    assert_eq!(item.parcels_count,  parcels_count);
    assert_eq!(item.parcels_pending, parcels_count, "no parcels paid yet");
    assert_eq!(item.next_payment_amount, parcel_value.to_string());
    assert!(item.can_pay, "should be payable immediately");
    assert_eq!(item.parcels.len(), parcels_count as usize);
    assert!(
        item.parcels.iter().all(|p| !p.is_paid),
        "all parcels should start as unpaid",
    );
}

// ─────────────────────────────────────────────────────────────
// prepare_repayment_logic
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn prepare_repayment_invalid_coop_id() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = prepare_repayment_logic(
        &sg, &env.blockchain, "not-bytes32", &env.usdt_address,
        from_alloy(env.approved_wallet), "1",
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::InvalidCoopId(_))),
        "expected InvalidCoopId, got {result:?}",
    );
}

#[tokio::test]
async fn prepare_repayment_nonexistent_loan_returns_payment_not_due() {
    let env    = common::get_deployed().await;
    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    // req 99999 does not exist; getNextPaymentAmount returns (0, false)
    let result = prepare_repayment_logic(
        &sg, &env.blockchain, &env.coop_id_hex, &env.usdt_address,
        from_alloy(env.approved_wallet), "99999",
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::PaymentNotDue)),
        "expected PaymentNotDue for non-existent loan, got {result:?}",
    );
}

#[tokio::test]
async fn prepare_repayment_non_member_wallet_returns_error() {
    let env = common::get_deployed().await;

    // Need an active loan so that can_pay = true, exposing the member check.
    let (req_id, _) = make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;

    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    let result = prepare_repayment_logic(
        &sg, &env.blockchain, &env.coop_id_hex, &env.usdt_address,
        from_alloy(env.unapproved_wallet), &req_id.to_string(),
    ).await;

    assert!(
        matches!(result, Err(LoanRequisitionError::WalletNotMember)),
        "expected WalletNotMember for non-member wallet, got {result:?}",
    );
}

#[tokio::test]
async fn prepare_repayment_returns_bundle_with_approve_calldata() {
    let env = common::get_deployed().await;
    let (req_id, _) = make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;

    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    // Reset any leftover allowance so the logic always encodes an approve step.
    let _guard = env.second_admin_lock.lock().await;
    {
        let signer = PrivateKeySigner::from_str(
            env.second_admin_key_hex.trim_start_matches("0x")
        ).unwrap();
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm_addr:   Address = env.loan_machine_address.parse().unwrap();
        let usdt_addr: Address = env.usdt_address.parse().unwrap();
        MockUSDT::new(usdt_addr, provider)
            .approve(lm_addr, U256::ZERO)
            .send().await.unwrap()
            .watch().await.unwrap();
    }

    let bundle = prepare_repayment_logic(
        &sg, &env.blockchain, &env.coop_id_hex, &env.usdt_address,
        from_alloy(env.second_admin), &req_id.to_string(),
    ).await.expect("should return bundle");

    assert_ne!(bundle.approve_calldata, "0x", "zero allowance → approve calldata expected");
    assert!(bundle.approve_calldata.starts_with("0x"));
    assert!(bundle.repay_calldata.starts_with("0x"));
    assert!(bundle.repay_calldata.len() > 10);
    assert_eq!(
        bundle.usdt_address.to_lowercase(),
        env.usdt_address.to_lowercase(),
        "USDT address should match",
    );
    assert_eq!(
        bundle.loan_machine_address.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
        "contract address should match",
    );
}

#[tokio::test]
async fn prepare_repayment_skips_approve_when_allowance_sufficient() {
    let env = common::get_deployed().await;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;

    let server = server_returning(empty_subgraph()).await;
    let sg     = SubgraphService::new(server.uri());

    // Hold second_admin_lock for the whole approve + check sequence so no
    // concurrent test can reset the allowance underneath us.
    let _guard = env.second_admin_lock.lock().await;
    {
        let signer = PrivateKeySigner::from_str(
            env.second_admin_key_hex.trim_start_matches("0x")
        ).unwrap();
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm_addr:   Address = env.loan_machine_address.parse().unwrap();
        let usdt_addr: Address = env.usdt_address.parse().unwrap();
        MockUSDT::new(usdt_addr, provider)
            .approve(lm_addr, parcel_value)
            .send().await.unwrap()
            .watch().await.unwrap();
    }

    let bundle = prepare_repayment_logic(
        &sg, &env.blockchain, &env.coop_id_hex, &env.usdt_address,
        from_alloy(env.second_admin), &req_id.to_string(),
    ).await.expect("should return bundle");

    assert_eq!(bundle.approve_calldata, "0x", "sufficient allowance → no approve needed");
}

// ─────────────────────────────────────────────────────────────
// Contract revert paths (extreme / edge cases)
//
// Simulated via estimate_gas (eth_estimateGas) so the tx never lands
// on-chain. Each test verifies the revert reason translated by
// BlockchainError::from_gas_estimate.
// ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn repay_zero_amount_is_rejected_by_contract() {
    let env = common::get_deployed().await;
    let (req_id, _) = make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;
    let admin2_id   = admin2_member_id();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    let err = LoanMachine::new(lm_addr, provider)
        .repay(req_id, U256::ZERO, admin2_id)
        .from(env.second_admin)
        .estimate_gas().await
        .map_err(BlockchainError::from_gas_estimate)
        .expect_err("zero amount should revert");

    assert_eq!(expect_revert(err), "Invalid amount.");
}

#[tokio::test]
async fn repay_less_than_parcel_amount_is_rejected_by_contract() {
    let env = common::get_deployed().await;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;
    let admin2_id = admin2_member_id();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address  = env.loan_machine_address.parse().unwrap();
    let wrong_amount = parcel_value - U256::from(1);

    let err = LoanMachine::new(lm_addr, provider)
        .repay(req_id, wrong_amount, admin2_id)
        .from(env.second_admin)
        .estimate_gas().await
        .map_err(BlockchainError::from_gas_estimate)
        .expect_err("amount below parcel value should revert");

    assert_eq!(expect_revert(err), "Invalid amount.");
}

#[tokio::test]
async fn repay_more_than_parcel_amount_is_rejected_by_contract() {
    let env = common::get_deployed().await;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;
    let admin2_id = admin2_member_id();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let wrong_amount = parcel_value + U256::from(1);

    let err = LoanMachine::new(lm_addr, provider)
        .repay(req_id, wrong_amount, admin2_id)
        .from(env.second_admin)
        .estimate_gas().await
        .map_err(BlockchainError::from_gas_estimate)
        .expect_err("amount above parcel value should revert");

    assert_eq!(expect_revert(err), "Invalid amount.");
}

#[tokio::test]
async fn repay_without_usdt_approval_is_rejected_by_contract() {
    let env = common::get_deployed().await;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(6_000_000u64), 2, 30).await;
    let admin2_id = admin2_member_id();

    // Hold second_admin_lock for the reset + simulate sequence so no other
    // test can approve underneath us between these two steps.
    let _guard = env.second_admin_lock.lock().await;
    {
        let signer = PrivateKeySigner::from_str(
            env.second_admin_key_hex.trim_start_matches("0x")
        ).unwrap();
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm_addr:   Address = env.loan_machine_address.parse().unwrap();
        let usdt_addr: Address = env.usdt_address.parse().unwrap();
        MockUSDT::new(usdt_addr, provider)
            .approve(lm_addr, U256::ZERO)
            .send().await.unwrap()
            .watch().await.unwrap();
    }

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    // MockUSDT is OpenZeppelin ERC20 which reverts with Error(string) rather
    // than returning false, so the contract's LoanMachine_TokenTransferFailed
    // custom error never fires. We just verify the simulation is rejected.
    LoanMachine::new(lm_addr, provider)
        .repay(req_id, parcel_value, admin2_id)
        .from(env.second_admin)
        .estimate_gas().await
        .expect_err("repay without USDT approval should revert");
}

// ── happy path (smoke test) ───────────────────────────────────

#[tokio::test]
async fn repay_correct_amount_with_approval_succeeds() {
    let env = common::get_deployed().await;
    let (req_id, parcel_value) =
        make_active_loan(&env, U256::from(9_000_000u64), 3, 30).await;
    let admin2_id = admin2_member_id();

    // Repay one parcel.
    repay_as(&env, &env.second_admin_key_hex, admin2_id, req_id, parcel_value).await;

    // Verify parcels_pending decreased.
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lc = LoanMachine::new(lm_addr, provider)
        .getLoanContract(req_id)
        .call().await
        .expect("getLoanContract")
        ._0;

    assert_eq!(lc.parcelsPending, 2, "one parcel paid, two remaining");
}
