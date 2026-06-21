//! Helpers to broadcast common multisig proposals from test admins.
//!
//! Locking discipline:
//! - All proposal CREATIONS go through admin1 under `platform_admin_lock`.
//!   This serializes the on-chain `proposalCounter` increment so the id
//!   returned by `eth_call` matches what `eth_sendTransaction` actually
//!   produces.
//! - Confirmation helpers use the relevant admin's key under its own
//!   lock. `confirmProposal` doesn't touch the counter, so different
//!   admins can confirm in parallel safely.

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolValue;

use loan_machine_core::services::blockchain::deployable::LoanMachine;

use crate::common::deploy::DeployedEnv;
use std::sync::atomic::{AtomicU64, Ordering};

/// `ProposalType.AddAdmin` — Solidity enums are uint8 in the ABI.
const PROPOSAL_TYPE_ADD_ADMIN: u8 = 1;

fn signer_from_hex(hex_key: &str) -> PrivateKeySigner {
    let bytes = hex::decode(hex_key.trim_start_matches("0x"))
        .expect("admin key is valid hex");
    PrivateKeySigner::from_slice(&bytes).expect("admin key is 32 bytes")
}


/// Process-wide counter; each call produces a distinct never-before-used target.
/// Avoids `LoanMachine_WalletApprovalAlreadyProposed` collisions across tests
/// sharing one anvil (which all tests in this binary do).
fn next_unique_target() -> Address {
    static COUNTER: AtomicU64 = AtomicU64::new(0xA000_0000_0000_0001);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut bytes = [0u8; 20];
    bytes[12..].copy_from_slice(&n.to_be_bytes());
    Address::from(bytes)
}

/// Propose a wallet approval for a fresh target address via admin invitation.
/// Returns `(target, proposal_id)`. Tests that care only about the proposal id
/// can ignore the target.
pub async fn propose_fresh_wallet_approval(env: &DeployedEnv) -> (Address, u64) {
    let target = next_unique_target();
    let pid = propose_wallet_approval(env, target).await;
    (target, pid)
}

pub async fn propose_wallet_approval(env: &DeployedEnv, target: Address) -> u64 {
    let _guard = env.platform_admin_lock.lock().await;

    let signer   = signer_from_hex(&env.platform_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    let pid: U256 = lm.proposeApproveWalletAsAdmin(target)
        .from(env.approved_wallet)
        .call().await
        .expect("simulate proposeApproveWalletAsAdmin")._0;
    lm.proposeApproveWalletAsAdmin(target).send().await
        .expect("send proposeApproveWalletAsAdmin")
        .watch().await
        .expect("mine proposeApproveWalletAsAdmin");
    pid.try_into().expect("proposalId fits u64")
}

pub async fn propose_add_admin(env: &DeployedEnv, new_admin: Address) -> u64 {
    let _guard = env.platform_admin_lock.lock().await;

    let signer   = signer_from_hex(&env.platform_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    let data: Bytes = new_admin.abi_encode().into();
    // `proposeAction` is `onlyAdmin`. Wallet filler doesn't populate
    // `from` on `.call()`, so without this the simulate reverts with
    // LoanMachine_NotAdmin even though `.send()` would have worked.
    let pid: U256 = lm
        .proposeAction(PROPOSAL_TYPE_ADD_ADMIN, data.clone())
        .from(env.approved_wallet)
        .call().await
        .expect("simulate proposeAction(AddAdmin)")
        .proposalId;
    lm.proposeAction(PROPOSAL_TYPE_ADD_ADMIN, data).send().await
        .expect("send proposeAction(AddAdmin)")
        .watch().await
        .expect("mine proposeAction(AddAdmin)");
    pid.try_into().expect("proposalId fits u64")
}

async fn confirm_with(env: &DeployedEnv, signer: PrivateKeySigner, proposal_id: u64) {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    lm.confirmProposal(U256::from(proposal_id)).send().await
        .expect("send confirmProposal")
        .watch().await
        .expect("mine confirmProposal");
}

pub async fn confirm_as_second_admin(env: &DeployedEnv, proposal_id: u64) {
    let _guard = env.second_admin_lock.lock().await;
    confirm_with(env, signer_from_hex(&env.second_admin_key_hex), proposal_id).await;
}

pub async fn confirm_as_third_admin(env: &DeployedEnv, proposal_id: u64) {
    let _guard = env.third_admin_lock.lock().await;
    confirm_with(env, signer_from_hex(&env.third_admin_key_hex), proposal_id).await;
}