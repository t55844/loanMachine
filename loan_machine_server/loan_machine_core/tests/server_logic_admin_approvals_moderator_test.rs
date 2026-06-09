//! E2E tests for admin_approvals flows that require a moderator.
//!
//! Lives in its own binary so it can call `bootstrap_admin1_as_moderator`
//! once per `#[tokio::test]` without affecting the non-moderator suite in
//! `server_logic_admin_approvals_test.rs`. The bootstrap is idempotent;
//! the cost after the first call is one chain read.

mod common;
use crate::common::{
    bootstrap_admin1_as_moderator, one_cooperative_payload,
    propose_fresh_wallet_approval, server_returning,
};

use alloy::primitives::{keccak256, U256};

use loan_machine_core::server_logic::admin_approvals::prepare_cosign_proposal_logic;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

fn cosign_selector() -> [u8; 4] {
    let h = keccak256(b"cosignProposal(uint256,bytes32)");
    [h[0], h[1], h[2], h[3]]
}

/// Happy path. Admin1 is a moderator (post-bootstrap) and vinculated as
/// founder. A fresh ApproveWallet proposal is pending, requiring cosign.
/// `prepare_cosign_proposal_logic` should resolve admin1's memberId,
/// pass on-chain checks during estimate_gas, and return a fully-formed
/// bundle whose calldata is `cosignProposal(pid, admin1_member_id)`.
#[tokio::test]
async fn prepare_cosign_succeeds_for_moderator() {
    let env = common::get_deployed().await;
    bootstrap_admin1_as_moderator(&env).await;
    let (_target, pid) = propose_fresh_wallet_approval(&env).await;

    // resolve_member_id is subgraph-first with chain fallback. Returning a
    // generic payload (no memberRegisteredEvents that match) sends it to
    // the chain, where admin1 is vinculated as founder.
    let server   = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let bundle = prepare_cosign_proposal_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, pid,
        from_alloy(env.approved_wallet),
    ).await.expect("admin1 is moderator + vinculated → cosign should estimate");

    // ── shape ─────────────────────────────────────────────
    assert!(bundle.to.starts_with("0x"));
    assert!(bundle.gas_hex.starts_with("0x"));
    assert_eq!(
        bundle.to.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
        "tx targets the loan machine"
    );
    let gas_value = u64::from_str_radix(bundle.gas_hex.trim_start_matches("0x"), 16)
        .expect("gas_hex is valid hex");
    assert!(gas_value > 21_000, "gas estimate should be above bare-tx minimum");

    // ── calldata ──────────────────────────────────────────
    // cosignProposal(uint256 proposalId, bytes32 moderatorMemberId)
    //   4 bytes selector + 32 bytes pid + 32 bytes memberId = 68 bytes total
    let raw = hex::decode(bundle.data.trim_start_matches("0x"))
        .expect("data is valid hex");
    assert_eq!(raw.len(), 4 + 32 + 32, "calldata is selector + two 32-byte args");
    assert_eq!(&raw[..4], &cosign_selector(),  "selector is cosignProposal");
    assert_eq!(U256::from_be_slice(&raw[4..36]),  U256::from(pid),
               "first arg is proposal id");
    assert_eq!(&raw[36..68], env.founder_member_id.as_slice(),
               "second arg is admin1's memberId resolved on-chain, not from the client");
}

/// Negative path that depends on the moderator state: a *vinculated*
/// moderator cosigning the wrong kind of proposal. AddAdmin doesn't
/// require moderator cosign, so `cosignProposal` reverts with
/// LoanMachine_ModeratorCosignRequired.
///
/// NOTE: as of the current `contract_errors.rs`, selector 0x71055050 is
/// NOT in the translator table — the entry mapped to "Administrator has not
/// been proposed yet." uses a stale selector. So this test asserts on the
/// fallback "Unknown error (0x71055050)." string until the translator
/// is fixed. Update the assertion when you add the proper mapping.
#[tokio::test]
async fn prepare_cosign_rejects_proposal_that_doesnt_need_cosign() {
    use crate::common::propose_add_admin;
    use loan_machine_core::server_logic::admin_approvals::AdminApprovalError;
    use loan_machine_core::services::blockchain::BlockchainError;

    let env = common::get_deployed().await;
    bootstrap_admin1_as_moderator(&env).await;

    // AddAdmin proposes admin1-already-confirmed; we don't confirm again,
    // so it sits pending without moderator cosign required.
    let pid = propose_add_admin(&env, env.scratch_admin_address).await;

    let server   = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_cosign_proposal_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, pid,
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    let message = match err {
        AdminApprovalError::Blockchain(BlockchainError::ContractRevert { message, .. }) => message,
        other => panic!("expected ContractRevert, got {other:?}"),
    };
    assert_eq!(message, "Unknown error (0x71055050).");
}