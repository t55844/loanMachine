//! On-chain donate/withdraw helpers for financial integration tests.
//!
//! Provider construction is inlined per helper, matching the pattern in
//! `tests/common/elections.rs` — extracting it into a `-> impl Provider`
//! function erases the transport type, which breaks `LoanMachine::new`'s
//! `Http<Client>` bound.

use std::str::FromStr;

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

use loan_machine_core::services::blockchain::deployable::{LoanMachine, MockUSDT};

use crate::common::deploy::DeployedEnv;

fn signer_from_hex(hex_key: &str) -> PrivateKeySigner {
    PrivateKeySigner::from_str(hex_key.trim_start_matches("0x"))
        .expect("valid private key hex")
}

/// Approve + donate `amount` (raw 6-decimal units) from the wallet derived
/// from `signer_key_hex`. Panics on revert — callers that need to assert a
/// revert should go through `withdraw_logic`/`donate_logic` instead, which
/// surface contract errors as typed results.
pub async fn donate_as(
    env:            &DeployedEnv,
    signer_key_hex: &str,
    member_id:      FixedBytes<32>,
    amount:         U256,
) {
    let signer = signer_from_hex(signer_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let lm_addr:   Address = env.loan_machine_address.parse().unwrap();
    let usdt_addr: Address = env.usdt_address.parse().unwrap();

    MockUSDT::new(usdt_addr, provider.clone())
        .approve(lm_addr, amount)
        .send().await.expect("send approve")
        .watch().await.expect("mine approve");

    LoanMachine::new(lm_addr, provider)
        .donate(amount, member_id)
        .send().await.expect("send donate")
        .watch().await.expect("mine donate");
}

/// `withdraw(amount, memberId)` from the wallet derived from
/// `signer_key_hex`. Panics on revert — same rationale as `donate_as`.
pub async fn withdraw_as(
    env:            &DeployedEnv,
    signer_key_hex: &str,
    member_id:      FixedBytes<32>,
    amount:         U256,
) {
    let signer = signer_from_hex(signer_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    LoanMachine::new(lm_addr, provider)
        .withdraw(amount, member_id)
        .send().await.expect("send withdraw")
        .watch().await.expect("mine withdraw");
}

/// Send `createLoanRequisition` on-chain and return the resulting requisition ID.
///
/// Caller is responsible for ensuring `availableBalance >= amount` (donate first)
/// and that the member has < 3 open requisitions before calling.
pub async fn create_requisition_as(
    env:            &DeployedEnv,
    signer_key_hex: &str,
    member_id:      FixedBytes<32>,
    amount:         U256,
    parcels_count:  u32,
    days_interval:  u32,
) -> U256 {
    let signer = signer_from_hex(signer_key_hex);
    let from   = signer.address();
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    // Simulate first to capture the return value (requisition ID).
    // `.from` must be explicit — without it the eth_call uses address(0) as
    // msg.sender, failing the validMember check.
    let req_id = lm
        .createLoanRequisition(amount, parcels_count, member_id, days_interval)
        .from(from)
        .call().await
        .expect("simulate createLoanRequisition")
        ._0;

    lm.createLoanRequisition(amount, parcels_count, member_id, days_interval)
        .send().await.expect("send createLoanRequisition")
        .watch().await.expect("mine createLoanRequisition");

    req_id
}

/// Send `coverLoan(requisitionId, coveragePercentage, memberId)` on-chain.
/// Panics on revert — the caller must ensure the req is coverable and the
/// coverer has sufficient donation balance.
pub async fn cover_as(
    env:            &DeployedEnv,
    signer_key_hex: &str,
    member_id:      FixedBytes<32>,
    req_id:         U256,
    coverage_pct:   u32,
) {
    let signer = signer_from_hex(signer_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    LoanMachine::new(lm_addr, provider)
        .coverLoan(req_id, coverage_pct, member_id)
        .send().await.expect("send coverLoan")
        .watch().await.expect("mine coverLoan");
}

/// Send `cancelLoanRequisition(requisitionId, memberId)` on-chain.
/// Must be sent by the borrower — `signer_key_hex` must be the borrower's key
/// and `member_id` must be their memberId.
pub async fn cancel_requisition_as(
    env:            &DeployedEnv,
    signer_key_hex: &str,
    member_id:      FixedBytes<32>,
    req_id:         U256,
) {
    let signer = signer_from_hex(signer_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    LoanMachine::new(lm_addr, provider)
        .cancelLoanRequisition(req_id, member_id)
        .send().await.expect("send cancelLoanRequisition")
        .watch().await.expect("mine cancelLoanRequisition");
}

/// Read the withdrawable donation balance for `wallet_addr` directly from the chain.
/// Returns raw 6-decimal USDT units as u128.
pub async fn get_withdrawable(env: &DeployedEnv, wallet_addr: Address) -> u128 {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());

    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    LoanMachine::new(lm_addr, provider)
        .getUserFinancials(wallet_addr)
        .call().await
        .expect("getUserFinancials")
        .withdrawable
        .saturating_to::<u128>()
}

/// Mint `amount` of MockUSDT to the wallet derived from `signer_key_hex`.
/// `MockUSDT.mint` is unrestricted, so any signer can mint to itself.
pub async fn mint_usdt(env: &DeployedEnv, signer_key_hex: &str, amount: U256) {
    let signer = signer_from_hex(signer_key_hex);
    let addr = signer.address();
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let usdt_addr: Address = env.usdt_address.parse().unwrap();

    MockUSDT::new(usdt_addr, provider)
        .mint(addr, amount)
        .send().await.expect("send mint")
        .watch().await.expect("mine mint");
}
