mod common;
use crate::common::deploy::DeployedEnv;
use crate::common::{mount_query, one_cooperative_payload, server_returning};

use alloy::network::EthereumWallet;
use alloy::primitives::{keccak256, Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

use loan_machine_core::server_logic::admin_approvals::{
    list_pending_approvals_logic, prepare_confirm_proposal_logic,
    prepare_cosign_proposal_logic, AdminApprovalError,
};
use loan_machine_core::services::blockchain::deployable::LoanMachine;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::from_alloy;

use serde_json::json;
use wiremock::MockServer;

const ZERO_ADDR: &str = "0x0000000000000000000000000000000000000000";

/// One-off helper kept in this file (not promoted to common/). Broadcasts
/// `proposeWalletApproval(target)` from the founder (admin1) and returns
/// the new on-chain proposal id. Used only by the happy-path encoding
/// test; every other test mocks the subgraph.
async fn propose_wallet_approval_inline(env: &DeployedEnv, target: Address) -> u64 {
    let _guard = env.platform_admin_lock.lock().await;

    let bytes  = hex::decode(env.platform_admin_key_hex.trim_start_matches("0x")).unwrap();
    let signer = PrivateKeySigner::from_slice(&bytes).unwrap();
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    let pid: U256 = lm.proposeWalletApproval(target).call().await
        .expect("simulate proposeWalletApproval").proposalId;
    lm.proposeWalletApproval(target).send().await
        .expect("send proposeWalletApproval")
        .watch().await
        .expect("mine proposeWalletApproval");

    pid.try_into().expect("proposalId fits u64")
}

fn confirm_selector() -> [u8; 4] {
    let h = keccak256(b"confirmProposal(uint256)");
    [h[0], h[1], h[2], h[3]]
}

// ── list_pending_approvals_logic ─────────────────────────────────────

#[tokio::test]
async fn list_returns_threshold_and_admin_count_from_chain() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    mount_query(&server, "ApproveWalletProposals", json!({
        "data": { "created": [], "executed": [], "confirmed": [], "cosigned": [] }
    })).await;

    let subgraph = SubgraphService::new(server.uri());
    let r = list_pending_approvals_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, from_alloy(env.approved_wallet)
    ).await.expect("list");

    assert!(r.proposals.is_empty());
    assert_eq!(r.threshold,    2, "threshold from chain");
    assert_eq!(r.total_admins, 3, "admin count from chain");
}

#[tokio::test]
async fn list_filters_executed_proposals() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    mount_query(&server, "ApproveWalletProposals", json!({
        "data": {
            "created": [
                {"proposalId": "0", "proposer": ZERO_ADDR, "blockTimestamp": "100"},
                {"proposalId": "1", "proposer": ZERO_ADDR, "blockTimestamp": "101"},
            ],
            "executed":  [{"proposalId": "0"}],
            "confirmed": [],
            "cosigned":  [],
        }
    })).await;

    let subgraph = SubgraphService::new(server.uri());
    let r = list_pending_approvals_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, from_alloy(env.approved_wallet)
    ).await.unwrap();

    assert_eq!(r.proposals.len(), 1);
    assert_eq!(r.proposals[0].proposal_id, 1);
}

#[tokio::test]
async fn list_viewer_confirmed_flag_is_per_caller() {
    let env = common::get_deployed().await;
    let admin2_hex = format!("{:#x}", env.second_admin).to_lowercase();

    let server = MockServer::start().await;
    mount_query(&server, "ApproveWalletProposals", json!({
        "data": {
            "created":   [{"proposalId": "0", "proposer": ZERO_ADDR, "blockTimestamp": "100"}],
            "executed":  [],
            "confirmed": [{"proposalId": "0", "admin": admin2_hex}],
            "cosigned":  [],
        }
    })).await;

    let subgraph = SubgraphService::new(server.uri());

    let from_a2 = list_pending_approvals_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, from_alloy(env.second_admin)
    ).await.unwrap();
    let from_a3 = list_pending_approvals_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, from_alloy(env.third_admin)
    ).await.unwrap();

    assert!( from_a2.proposals[0].viewer_confirmed, "admin2 confirmed → flag true for admin2");
    assert!(!from_a3.proposals[0].viewer_confirmed, "admin3 didn't confirm → flag false for admin3");
}

#[tokio::test]
async fn list_cosigned_flag_visible_to_all() {
    let env = common::get_deployed().await;

    let server = MockServer::start().await;
    mount_query(&server, "ApproveWalletProposals", json!({
        "data": {
            "created":   [{"proposalId": "0", "proposer": ZERO_ADDR, "blockTimestamp": "100"}],
            "executed":  [],
            "confirmed": [],
            "cosigned":  [{"proposalId": "0"}],
        }
    })).await;

    let subgraph = SubgraphService::new(server.uri());
    let r = list_pending_approvals_logic(
        &subgraph, &env.blockchain, &env.coop_id_hex, from_alloy(env.approved_wallet)
    ).await.unwrap();

    assert!(r.proposals[0].moderator_cosigned);
}

#[tokio::test]
async fn list_rejects_invalid_coop_id() {
    let env = common::get_deployed().await;
    let server = MockServer::start().await;
    let subgraph = SubgraphService::new(server.uri());

    let err = list_pending_approvals_logic(
        &subgraph, &env.blockchain, "not-bytes32", from_alloy(env.approved_wallet)
    ).await.unwrap_err();
    assert!(matches!(err, AdminApprovalError::InvalidCoopId(_)));
}

// ── prepare_confirm_proposal_logic ───────────────────────────────────

#[tokio::test]
async fn prepare_confirm_encodes_proposal_id() {
    let env = common::get_deployed().await;
    let pid = propose_wallet_approval_inline(&env, env.unapproved_wallet).await;

    let b = prepare_confirm_proposal_logic(
        &env.blockchain, &env.coop_id_hex, pid, from_alloy(env.second_admin)
    ).await.expect("admin2 hasn't confirmed → gas estimation should succeed");

    assert!(b.to.starts_with("0x"));
    assert!(b.gas_hex.starts_with("0x"));
    assert_eq!(
        b.to.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
        "tx targets the loan machine"
    );

    let raw = hex::decode(b.data.trim_start_matches("0x")).unwrap();
    assert_eq!(&raw[..4], &confirm_selector());
    assert_eq!(U256::from_be_slice(&raw[4..36]), U256::from(pid));
}

#[tokio::test]
async fn prepare_confirm_rejects_invalid_coop_id() {
    let env = common::get_deployed().await;
    let err = prepare_confirm_proposal_logic(
        &env.blockchain, "not-bytes32", 0, from_alloy(env.approved_wallet)
    ).await.unwrap_err();
    assert!(matches!(err, AdminApprovalError::InvalidCoopId(_)));
}

// ── prepare_cosign_proposal_logic ────────────────────────────────────

#[tokio::test]
async fn prepare_cosign_rejects_non_vinculated_wallet() {
    let env = common::get_deployed().await;
    let server = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_cosign_proposal_logic(
        &subgraph,
        &env.blockchain,
        &env.coop_id_hex,
        0,
        from_alloy(env.unapproved_wallet),
    ).await.unwrap_err();

    assert!(matches!(err, AdminApprovalError::NotVinculated));
}

#[tokio::test]
async fn prepare_cosign_rejects_invalid_coop_id() {
    let env = common::get_deployed().await;
    let server = server_returning(one_cooperative_payload()).await;
    let subgraph = SubgraphService::new(server.uri());

    let err = prepare_cosign_proposal_logic(
        &subgraph,
        &env.blockchain,
        "not-bytes32",
        0,
        from_alloy(env.approved_wallet),
    ).await.unwrap_err();

    assert!(matches!(err, AdminApprovalError::InvalidCoopId(_)));
}