// Integration tests for the concurrent on-chain fallback in resolve_wallet_coop.
//
// The primary path (subgraph) is exercised in vinculation_integration_test.rs.
// These tests focus on the on-chain scan: forcing the subgraph mock to return
// empty so every call falls through to `on_chain_scan`, which runs all coop
// membership checks concurrently via `FuturesUnordered`.
//
// "Dummy" coops are created by deploying a fresh LoanMachine without calling
// `initializeMultisig` — its `walletToMemberId` mapping is all zeros, so
// `isWalletVinculated` returns `false` for every wallet without reverting.
// Registering several of them gives the concurrent scan real work to do.

mod common;
use crate::common::{member_events_empty, platform_admin_signer, server_returning};

use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;

use loan_machine_core::server_logic::vinculation::get_wallet_coop_logic;
use loan_machine_core::services::blockchain::deployable::{CoopRegistry, LoanMachine};
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

use crate::common::deploy::DeployedEnv;

/// Deploy an uninitiated LoanMachine and register it in CoopRegistry under
/// `name`. Caller must hold `platform_admin_lock` for the full call.
async fn register_dummy_coop(env: &DeployedEnv, name: &str) {
    let signer   = platform_admin_signer(env);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());

    let usdt_addr: Address = env.usdt_address.parse().unwrap();
    let dummy = LoanMachine::deploy(provider.clone(), usdt_addr)
        .await.expect("deploy dummy LoanMachine");

    let registry_addr: Address = env.coop_registry_address.parse().unwrap();
    CoopRegistry::new(registry_addr, provider)
        .registerCoop(name.into(), *dummy.address())
        .send().await.expect("send registerCoop")
        .watch().await.expect("mine registerCoop");
}

// ── single-coop baseline ──────────────────────────────────────────────────────

/// Subgraph returns empty → on-chain fallback → finds the vinculated wallet
/// in the one registered coop.  Basic sanity: without this the multi-coop
/// tests below are meaningless.
#[tokio::test]
async fn on_chain_scan_finds_vinculated_wallet_when_subgraph_empty() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.approved_wallet);

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_wallet_coop_logic(&subgraph, &env.blockchain, wallet)
        .await.expect("on-chain scan should not error");

    let info = result.expect("vinculated wallet should be found via on-chain scan");
    assert_eq!(
        info.loan_machine.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
    );
}

/// Subgraph returns empty → on-chain fallback → all coops checked → None
/// because the wallet was never vinculated.
#[tokio::test]
async fn on_chain_scan_returns_none_for_non_vinculated_wallet_when_subgraph_empty() {
    let env    = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_wallet_coop_logic(&subgraph, &env.blockchain, wallet)
        .await.expect("on-chain scan should not error");

    assert!(result.is_none(), "unapproved wallet should not be found in any coop");
}

// ── multi-coop concurrent correctness ────────────────────────────────────────

/// Registry with 3 coops (2 dummies + 1 real).  The concurrent scan must
/// correctly identify the one coop where `approved_wallet` is vinculated and
/// return its address — not a dummy — proving all futures are evaluated and the
/// correct result is selected.
#[tokio::test]
async fn on_chain_scan_with_multiple_coops_finds_the_vinculated_one() {
    let env = common::get_deployed().await;
    {
        let _lock = env.platform_admin_lock.lock().await;
        register_dummy_coop(&env, "Dummy A").await;
        register_dummy_coop(&env, "Dummy B").await;
    }

    let wallet   = from_alloy(env.approved_wallet);
    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_wallet_coop_logic(&subgraph, &env.blockchain, wallet)
        .await.expect("concurrent scan should not error with multiple coops");

    let info = result.expect("vinculated wallet should be found among multiple coops");
    assert_eq!(
        info.loan_machine.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
        "scan should identify the initialized coop, not a dummy",
    );
}

/// Registry with 3 coops (2 dummies + 1 real) and a wallet vinculated in
/// none of them.  The concurrent scan must exhaust every future and return
/// None — proving early-exit only fires on a positive match, not on None.
#[tokio::test]
async fn on_chain_scan_with_multiple_coops_returns_none_for_non_vinculated_wallet() {
    let env = common::get_deployed().await;
    {
        let _lock = env.platform_admin_lock.lock().await;
        register_dummy_coop(&env, "Dummy C").await;
        register_dummy_coop(&env, "Dummy D").await;
    }

    let wallet   = from_alloy(env.unapproved_wallet);
    let server   = server_returning(member_events_empty()).await;
    let subgraph = SubgraphService::new(server.uri());

    let result = get_wallet_coop_logic(&subgraph, &env.blockchain, wallet)
        .await.expect("concurrent scan should not error");

    assert!(
        result.is_none(),
        "non-vinculated wallet should not be found across any of the registered coops",
    );
}
