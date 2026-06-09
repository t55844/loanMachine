mod common;
use crate::common::{
    make_deployment_service, make_identity_from_env, parse_hex_u64, platform_admin_signer, wa,
};

use loan_machine_core::server_logic::create_coop::{
    prepare_create_coop_logic, register_deployed_coop_logic, CreateCoopLogicError,
};
use loan_machine_core::services::coop_deployment::CoopDeploymentError;

// ── Validation tests (no chain interaction) ──────────────────

#[tokio::test]
async fn prepare_rejects_empty_name() {
    let env = common::get_deployed().await;
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let result = prepare_create_coop_logic(
        &identity, &svc,
        "".into(),
        wa(env.approved_wallet),
        vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

#[tokio::test]
async fn prepare_rejects_whitespace_only_name() {
    let env = common::get_deployed().await;
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let result = prepare_create_coop_logic(
        &identity, &svc,
        "   ".into(),
        wa(env.approved_wallet),
        vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

#[tokio::test]
async fn prepare_rejects_too_long_name() {
    let env = common::get_deployed().await;
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let result = prepare_create_coop_logic(
        &identity, &svc,
        "a".repeat(101),
        wa(env.approved_wallet),
        vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}

#[tokio::test]
async fn prepare_rejects_wrong_admin_count() {
    let env = common::get_deployed().await;
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let result = prepare_create_coop_logic(
        &identity, &svc,
        "Test Coop".into(),
        wa(env.approved_wallet),
        vec![wa(env.approved_wallet), wa(env.second_admin)], // 2 instead of 3
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
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let result = prepare_create_coop_logic(
        &identity, &svc,
        "Test Coop".into(),
        wa(env.unapproved_wallet), // ← founder NOT in admin list
        vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)],
        2,
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(CoopDeploymentError::FounderNotInAdmins))
    ));
}

// ── Happy path ────────────────────────────────────────────────

#[tokio::test]
async fn prepare_happy_path_returns_bundle() {
    let env = common::get_deployed().await;
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let bundle = prepare_create_coop_logic(
        &identity, &svc,
        "Cooperativa Test".into(),
        wa(env.approved_wallet),
        vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)],
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
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let admins = vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)];

    let b1 = prepare_create_coop_logic(
        &identity, &svc, "Coop A".into(),
        wa(env.approved_wallet),
        admins.clone(), 2,
    ).await.unwrap();

    let b2 = prepare_create_coop_logic(
        &identity, &svc, "Coop B".into(),
        wa(env.approved_wallet),
        admins, 2,
    ).await.unwrap();

    assert_ne!(b1.access_code, b2.access_code);
}

#[tokio::test]
async fn initialize_data_starts_with_initialize_multisig_selector() {
    let env = common::get_deployed().await;
    let identity = make_identity_from_env();
    let svc = make_deployment_service(&env);

    let bundle = prepare_create_coop_logic(
        &identity, &svc, "Test".into(),
        wa(env.approved_wallet),
        vec![wa(env.approved_wallet), wa(env.second_admin), wa(env.third_admin)],
        2,
    ).await.unwrap();

    assert!(bundle.initialize_data.len() > 2 + 8, "must have selector + args");
}

// ── register_deployed_coop ────────────────────────────────────

#[tokio::test]
async fn register_rejects_address_with_no_code() {
    let env = common::get_deployed().await;
    let svc = make_deployment_service(&env);

    let result = register_deployed_coop_logic(
        &svc, "Test Coop".into(),
        "0x0000000000000000000000000000000000000001".into(),
        wa(env.approved_wallet),
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(CoopDeploymentError::LoanMachineHasNoCode))
    ));
}

#[tokio::test]
async fn register_rejects_founder_not_admin() {
    let env = common::get_deployed().await;
    let svc = make_deployment_service(&env);

    let result = register_deployed_coop_logic(
        &svc, "Test Coop".into(),
        env.loan_machine_address.clone(),
        wa(env.unapproved_wallet),
    ).await;

    assert!(matches!(
        result,
        Err(CreateCoopLogicError::Deployment(CoopDeploymentError::FounderNotAdminOfDeployedContract))
    ));
}

#[tokio::test]
async fn register_happy_path() {
    let env = common::get_deployed().await;
    let svc = make_deployment_service(&env);
    let admin_signer = platform_admin_signer(&env);

    let result = register_deployed_coop_logic(
        &svc,
        "Cooperativa Real".into(),
        env.loan_machine_address.clone(),
        wa(admin_signer.address()),
    ).await.expect("registration should succeed");

    assert!(result.coop_id_hex.starts_with("0x"));
    assert_eq!(result.coop_id_hex.len(), 66, "bytes32 = 0x + 64 hex chars");
    assert!(result.registration_tx_hash.starts_with("0x"));
    assert_eq!(
        result.loan_machine_address.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
    );
}

#[tokio::test]
async fn register_rejects_empty_name() {
    let env = common::get_deployed().await;
    let svc = make_deployment_service(&env);

    let result = register_deployed_coop_logic(
        &svc, "".into(),
        env.loan_machine_address.clone(),
        wa(env.unapproved_wallet),
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::InvalidName)));
}
