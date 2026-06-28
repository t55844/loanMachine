//! Higher-level bootstrap flows that put the deployed coop into specific
//! preparatory states (moderator elected, member vinculated, etc.) for
//! tests of functionality that lives downstream of those states.
//!
//! Provider construction is inlined per helper — extracting it into a
//! `-> impl Provider` function erases the transport type, which breaks
//! `LoanMachine::new`'s `Http<Client>` bound.

use alloy::network::EthereumWallet;
use alloy::primitives::{keccak256, Address, FixedBytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

use loan_machine_core::services::blockchain::deployable::LoanMachine;

use crate::common::deploy::DeployedEnv;

fn signer_from_hex(hex_key: &str) -> PrivateKeySigner {
    let bytes = hex::decode(hex_key.trim_start_matches("0x"))
        .expect("admin key is valid hex");
    PrivateKeySigner::from_slice(&bytes).expect("admin key is 32 bytes")
}

pub fn admin2_member_id() -> FixedBytes<32> {
    keccak256(b"test-admin2-member-id")
}

pub fn admin3_member_id() -> FixedBytes<32> {
    keccak256(b"test-admin3-member-id")
}

/// Vinculate admin2 as a member via `joinCoop`. Idempotent under
/// `second_admin_lock`.
pub async fn vinculate_second_admin(env: &DeployedEnv) -> FixedBytes<32> {
    let _guard = env.second_admin_lock.lock().await;

    let signer = signer_from_hex(&env.second_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    let existing = lm.getMemberId(env.second_admin).call().await
        .expect("getMemberId(admin2)")._0;
    if existing != FixedBytes::ZERO {
        return existing;
    }

    let id = admin2_member_id();
    lm.joinCoop(id, env.second_admin)
        .send().await.expect("send joinCoop(admin2)")
        .watch().await.expect("mine joinCoop(admin2)");
    id
}

/// Make admin1 a moderator via the election flow. See module docs in the
/// previous draft for the math. Idempotent under `platform_admin_lock`.
///
/// Lock order: platform_admin_lock (held by this fn) > second_admin_lock
/// (acquired inside vinculate_second_admin). No other code path takes
/// these in reverse, so no deadlock.
pub async fn bootstrap_admin1_as_moderator(env: &DeployedEnv) {
    let _guard = env.platform_admin_lock.lock().await;

    let signer = signer_from_hex(&env.platform_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    if lm.isModerator(env.founder_member_id).call().await
        .expect("isModerator")._0
    {
        return;
    }

    let admin2_id = vinculate_second_admin(env).await;

    lm.openElection(env.founder_member_id, admin2_id)
        .send().await.expect("send openElection")
        .watch().await.expect("mine openElection");

    let election_id: u32 = lm.getCurrentElectionId().call().await
        .expect("getCurrentElectionId")._0
        .try_into().expect("election active immediately after open");

    lm.voteForModerator(election_id, env.founder_member_id, env.founder_member_id)
        .send().await.expect("send voteForModerator")
        .watch().await.expect("mine voteForModerator");

    assert!(
        lm.isModerator(env.founder_member_id).call().await
            .expect("post-vote isModerator check")._0,
        "unbeatable-majority auto-close should have made admin1 a moderator"
    );
}

/// Open an election between two vinculated, non-moderator members.
/// Caller is responsible for ensuring no election is already active.
/// Returns the new election_id.
pub async fn open_election(
    env:          &DeployedEnv,
    candidate_id: FixedBytes<32>,
    opponent_id:  FixedBytes<32>,
) -> u32 {
    let _guard = env.platform_admin_lock.lock().await;

    let signer = signer_from_hex(&env.platform_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    lm.openElection(candidate_id, opponent_id)
        .send().await.expect("send openElection")
        .watch().await.expect("mine openElection");

    lm.getCurrentElectionId().call().await
        .expect("getCurrentElectionId after open")._0
        .try_into().expect("election_id >= 0 immediately after open")
}

/// Cast a vote as admin2 in the given election.
pub async fn cast_vote_as_second_admin(
    env:          &DeployedEnv,
    election_id:  u32,
    candidate_id: FixedBytes<32>,
    voter_id:     FixedBytes<32>,
) {
    let _guard = env.second_admin_lock.lock().await;

    let signer = signer_from_hex(&env.second_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    lm.voteForModerator(election_id, candidate_id, voter_id)
        .send().await.expect("send voteForModerator")
        .watch().await.expect("mine voteForModerator");
}

/// Ensure there is an active election between the founder and admin2.
/// Idempotent under `platform_admin_lock`: the check and the open are
/// both done while holding the lock, so concurrent callers serialize and
/// only the first actually opens the election.
///
/// Lock order: second_admin_lock (vinculate, released before entering)
///             → platform_admin_lock (check + open, held together).
/// Returns `(election_id, admin2_member_id)`.
pub async fn bootstrap_active_election(env: &DeployedEnv) -> (u32, FixedBytes<32>) {
    let admin2_id = vinculate_second_admin(env).await;

    let _guard = env.platform_admin_lock.lock().await;

    let signer = signer_from_hex(&env.platform_admin_key_hex);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(env.rpc_url.parse().unwrap());
    let lm_addr: Address = env.loan_machine_address.parse().unwrap();
    let lm = LoanMachine::new(lm_addr, provider);

    let current_id: i32 = lm.getCurrentElectionId().call().await
        .expect("getCurrentElectionId")._0;
    if current_id >= 0 {
        return (current_id.try_into().expect("current_id >= 0"), admin2_id);
    }

    lm.openElection(env.founder_member_id, admin2_id)
        .send().await.expect("send openElection")
        .watch().await.expect("mine openElection");

    let election_id: u32 = lm.getCurrentElectionId().call().await
        .expect("getCurrentElectionId after open")._0
        .try_into().expect("election_id >= 0 immediately after open");

    (election_id, admin2_id)
}

/// Approve and vinculate admin3 as a member. Idempotent under `third_admin_lock`.
///
/// Flow:
///   1. admin1 proposes wallet approval for admin3  (under `platform_admin_lock`)
///   2. admin2 confirms → proposal executes          (under `second_admin_lock`)
///   3. admin3 calls `joinCoop`
///
/// Lock order: third_admin_lock → platform_admin_lock (released) → second_admin_lock (released).
/// No two locks are held simultaneously.
pub async fn vinculate_third_admin(env: &DeployedEnv) -> FixedBytes<32> {
    let _guard = env.third_admin_lock.lock().await;

    let lm_addr: Address = env.loan_machine_address.parse().unwrap();

    // Idempotency check.
    {
        let signer = signer_from_hex(&env.third_admin_key_hex);
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm = LoanMachine::new(lm_addr, provider);
        let existing = lm.getMemberId(env.third_admin).call().await
            .expect("getMemberId(admin3)")._0;
        if existing != FixedBytes::ZERO {
            return existing;
        }
    }

    // Step 1: admin1 proposes approval. Held under `platform_admin_lock` to avoid
    // a race between `.call()` (simulate PID) and `.send()` (actual proposal).
    let pid: u64 = {
        let _platform_guard = env.platform_admin_lock.lock().await;
        let signer = signer_from_hex(&env.platform_admin_key_hex);
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm = LoanMachine::new(lm_addr, provider);
        let pid: U256 = lm
            .proposeApproveWalletAsAdmin(env.third_admin)
            .from(env.approved_wallet)
            .call().await
            .expect("simulate proposeApproveWalletAsAdmin(admin3)")
            ._0;
        lm.proposeApproveWalletAsAdmin(env.third_admin)
            .send().await.expect("send proposeApproveWalletAsAdmin(admin3)")
            .watch().await.expect("mine proposeApproveWalletAsAdmin(admin3)");
        pid.try_into().expect("proposalId fits u64")
    }; // platform_admin_lock released here

    // Step 2: admin2 confirms (second_admin_lock acquired inside confirm_as_second_admin).
    {
        let _second_guard = env.second_admin_lock.lock().await;
        let signer = signer_from_hex(&env.second_admin_key_hex);
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm = LoanMachine::new(lm_addr, provider);
        lm.confirmProposal(U256::from(pid))
            .send().await.expect("send confirmProposal(admin3 approval)")
            .watch().await.expect("mine confirmProposal(admin3 approval)");
    }

    // Step 3: admin3 joins the coop.
    let id = admin3_member_id();
    {
        let signer = signer_from_hex(&env.third_admin_key_hex);
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(env.rpc_url.parse().unwrap());
        let lm = LoanMachine::new(lm_addr, provider);
        lm.joinCoop(id, env.third_admin)
            .send().await.expect("send joinCoop(admin3)")
            .watch().await.expect("mine joinCoop(admin3)");
    }

    id
}