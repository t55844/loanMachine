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
