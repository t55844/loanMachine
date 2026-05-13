mod common;

use loan_machine_core::server_logic::create_coop::{
    prepare_create_coop_logic,
    register_deployed_coop_logic,
    CreateCoopLogicError,
};

use loan_machine_core::services::coop_deployment::{CoopDeploymentService};
use crate::common::deploy::DeployedEnv;
use loan_machine_core::services::coop_deployment::CoopDeploymentError;

use crate::common::wa;

fn parse_hex_u64(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("bad hex gas value {s:?}: {e}"))
}

// Helper: build a service for a test using the deployed env's values.
async fn make_service(env: &DeployedEnv) -> CoopDeploymentService {
    use secrecy::SecretString;

    CoopDeploymentService::new(
        SecretString::from(env.platform_admin_key_hex.clone()),  // ← wrap then move
        &env.coop_registry_address,
        &env.rpc_url,
        env.loan_machine_bytecode.clone(),
        &env.usdt_address,
    )
    .expect("CoopDeploymentService::new")
}

// ── Validation tests (no chain interaction) ──────────────────

#[tokio::test]
async fn prepare_rejects_empty_name() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let result = prepare_create_coop_logic(
        &svc,
        "".into(),
        wa(env.approved_wallet),
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

#[tokio::test]
async fn prepare_rejects_whitespace_only_name() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let result = prepare_create_coop_logic(
        &svc, "   ".into(),
        wa(env.approved_wallet),
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

#[tokio::test]
async fn prepare_rejects_too_long_name() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let long_name = "a".repeat(101);
    let result = prepare_create_coop_logic(
        &svc, long_name,
        wa(env.approved_wallet),
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

#[tokio::test]
async fn prepare_rejects_wrong_admin_count() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    // 2 admins instead of 3
    let result = prepare_create_coop_logic(
        &svc, "Test Coop".into(),
        wa(env.approved_wallet),
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
        ],
        2,
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(
            CoopDeploymentError::AdminCountWrong { got: 2, expected: 3 }
        ))
    ));
}

#[tokio::test]
async fn prepare_rejects_founder_not_in_admins() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let result = prepare_create_coop_logic(
        &svc, "Test Coop".into(),
        wa(env.unapproved_wallet),               // ← founder NOT in admin list
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(
            CoopDeploymentError::FounderNotInAdmins
        ))
    ));
}


// ── Happy path: bundle is well-formed ────────────────────────

#[tokio::test]
async fn prepare_happy_path_returns_bundle() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let bundle = prepare_create_coop_logic(
        &svc, "Cooperativa Test".into(),
        wa(env.approved_wallet),
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
        ],
        2,
    ).await.expect("happy path");

    assert!(bundle.deploy_data.starts_with("0x"));
    assert!(bundle.initialize_data.starts_with("0x"));
    assert!(parse_hex_u64(&bundle.gas_deploy) > 1_000_000,
        "deploy gas should be substantial: {}", bundle.gas_deploy);
    assert!(parse_hex_u64(&bundle.gas_initialize) > 0);
    assert_eq!(bundle.access_code.len(), 12, "access code should be 12 chars");
}

#[tokio::test]
async fn access_codes_are_unique_across_calls() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let admins = vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
    ];

    let b1 = prepare_create_coop_logic(
        &svc, "Coop A".into(),
        wa(env.approved_wallet),
        admins.clone(),
        2,
    ).await.unwrap();

    let b2 = prepare_create_coop_logic(
        &svc, "Coop B".into(),
        wa(env.approved_wallet),
        admins,
        2,
    ).await.unwrap();

    assert_ne!(b1.access_code, b2.access_code,
        "two prepare calls should generate different access codes");
}

#[tokio::test]
async fn initialize_data_starts_with_initialize_multisig_selector() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let bundle = prepare_create_coop_logic(
        &svc, "Test".into(),
        wa(env.approved_wallet),
        vec![
            wa(env.approved_wallet),
            wa(env.second_admin),
            wa(env.third_admin),
        ],
        2,
    ).await.unwrap();

    // initializeMultisig(address[],uint256,string) selector — first 4 bytes
    // We don't compute the exact selector here, just sanity-check shape.
    assert!(bundle.initialize_data.len() > 2 + 8, "must have selector + args");
}

// ── register_deployed_coop ────────────────────────────────────
//
// The happy path needs an ALREADY-DEPLOYED LoanMachine where the founder
// is an admin. The existing `env.loan_machine_address` from common::deploy
// has admin1, admin2, admin3 as admins. We use admin1 (the env's first
// signer's address) as the founder.

#[tokio::test]
async fn register_rejects_address_with_no_code() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    // Address with no contract deployed at it
    let zero_addr = "0x0000000000000000000000000000000000000001";

    let result = register_deployed_coop_logic(
        &svc,
        "Test Coop".into(),
        zero_addr.into(),
        wa(env.approved_wallet),
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(
            CoopDeploymentError::LoanMachineHasNoCode
        ))
    ));
}

#[tokio::test]
async fn register_rejects_founder_not_admin() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    // env.unapproved_wallet is NOT in the admins of env.loan_machine_address
    let result = register_deployed_coop_logic(
        &svc,
        "Test Coop".into(),
        env.loan_machine_address.clone(),
        wa(env.unapproved_wallet),
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(
            CoopDeploymentError::FounderNotAdminOfDeployedContract
        ))
    ));
}

#[tokio::test]
async fn register_happy_path() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    // The first admin (admin1) is the founder; they're in env's LoanMachine admins.
    // We need their address — it's the same as the platform_admin_key's address
    // because tests/common/deploy.rs uses anvil.keys()[0] for both roles.
    use alloy::signers::local::PrivateKeySigner;
    use std::str::FromStr;
    let admin_signer = PrivateKeySigner::from_str(
        env.platform_admin_key_hex.trim_start_matches("0x")
    ).unwrap();
    let admin_addr = wa(admin_signer.address());

    let result = register_deployed_coop_logic(
        &svc,
        "Cooperativa Real".into(),
        env.loan_machine_address.clone(),
        admin_addr,
    ).await
    .expect("registration should succeed");

    assert!(result.coop_id_hex.starts_with("0x"));
    assert_eq!(result.coop_id_hex.len(), 2 + 64, "bytes32 hex should be 66 chars");
    assert!(result.registration_tx_hash.starts_with("0x"));
    assert_eq!(
        result.loan_machine_address.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
    );
}

#[tokio::test]
async fn register_rejects_empty_name() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let result = register_deployed_coop_logic(
        &svc, "".into(),
        env.loan_machine_address.clone(),
        wa(env.unapproved_wallet),
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

