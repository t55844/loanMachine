mod common;

use loan_machine_core::server_logic::create_coop::{
    prepare_create_coop_logic,
    register_deployed_coop_logic,
    CreateCoopLogicError,
};

use loan_machine_core::services::coop_deployment::{CoopDeploymentService};
use crate::common::deploy::DeployedEnv;

fn parse_hex_u64(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("bad hex gas value {s:?}: {e}"))
}

// Helper: build a service for a test using the deployed env's values.
async fn make_service(env: &DeployedEnv) -> CoopDeploymentService {
    use secrecy::SecretString;

    CoopDeploymentService::new(
        SecretString::from(env.platform_admin_key_hex.clone()),  // ← wrap then move
        &env.factory_address,
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
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicInvalidName)));
}

#[tokio::test]
async fn prepare_rejects_whitespace_only_name() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let result = prepare_create_coop_logic(
        &svc, "   ".into(),
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicInvalidName)));
}

#[tokio::test]
async fn prepare_rejects_too_long_name() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let long_name = "a".repeat(101);
    let result = prepare_create_coop_logic(
        &svc, long_name,
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicInvalidName)));
}

#[tokio::test]
async fn prepare_rejects_wrong_admin_count() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    // 2 admins instead of 3
    let result = prepare_create_coop_logic(
        &svc, "Test Coop".into(),
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicDeployment(_))));
}

#[tokio::test]
async fn prepare_rejects_founder_not_in_admins() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    // Founder address not in the admins list
    let result = prepare_create_coop_logic(
        &svc, "Test Coop".into(),
        format!("{:?}", env.unapproved_wallet),       // founder
        vec![                                           // 3 admins, none are founder
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicDeployment(_))));
}

#[tokio::test]
async fn prepare_rejects_invalid_admin_address() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let result = prepare_create_coop_logic(
        &svc, "Test Coop".into(),
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            "not-a-hex-address".into(),
            format!("{:?}", env.third_admin),
        ],
        2,
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicDeployment(_))));
}

// ── Happy path: bundle is well-formed ────────────────────────

#[tokio::test]
async fn prepare_happy_path_returns_bundle() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let bundle = prepare_create_coop_logic(
        &svc, "Cooperativa Test".into(),
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
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
        format!("{:?}", env.approved_wallet),
        format!("{:?}", env.second_admin),
        format!("{:?}", env.third_admin),
    ];

    let b1 = prepare_create_coop_logic(
        &svc, "Coop A".into(),
        format!("{:?}", env.approved_wallet),
        admins.clone(),
        2,
    ).await.unwrap();

    let b2 = prepare_create_coop_logic(
        &svc, "Coop B".into(),
        format!("{:?}", env.approved_wallet),
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
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
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
        format!("{:?}", env.approved_wallet),
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicDeployment(_))));
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
        format!("{:?}", env.unapproved_wallet),
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicDeployment(_))));
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
    let admin_addr = format!("{:?}", admin_signer.address());

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
        format!("{:?}", env.approved_wallet),
    ).await;

    assert!(matches!(result, Err(CreateCoopLogicError::ServerLogicInvalidName)));
}

// ─── Regressions for the .env debugging session ──────────────

/// Catches: registry address pointing at the wrong contract
/// (we spent an hour on this when our .env had factory_address as registry).
/// If platformAdmin() doesn't return our signer, something is misconfigured.
#[tokio::test]
async fn registry_platform_admin_matches_signer() {
    use alloy::providers::ProviderBuilder;
    use alloy::signers::local::PrivateKeySigner;
    use loan_machine_core::services::blockchain::abis::CoopRegistry;
    use std::str::FromStr;

    let env = common::get_deployed().await;

    let signer = PrivateKeySigner::from_str(
        env.platform_admin_key_hex.trim_start_matches("0x")
    ).unwrap();
    let signer_addr = signer.address();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());

    let registry_addr = env.factory_address.parse().unwrap();
    let registry = CoopRegistry::new(registry_addr, provider);

    let on_chain_admin = registry.platformAdmin().call().await
        .expect("platformAdmin() should succeed on a real CoopRegistry — \
                 if this fails, the address points at something else")
        ._0;

    assert_eq!(on_chain_admin, signer_addr,
        "platform admin mismatch: registry says {on_chain_admin}, \
         but signer is {signer_addr}. The deploy script and PLATFORM_ADMIN_PRIVATE_KEY \
         must use the same key — see tests/common/deploy.rs");
}

/// Catches: regression in CoopRegistry.registerCoop's coopId formula.
/// If someone reintroduces block.timestamp into the keccak, the simulate-vs-send
/// pattern will silently produce mismatched coopIds. We had this exact bug.
#[tokio::test]
async fn registering_same_args_twice_is_idempotent_in_id() {

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    use alloy::signers::local::PrivateKeySigner;
    use std::str::FromStr;
    let admin = PrivateKeySigner::from_str(env.platform_admin_key_hex.trim_start_matches("0x")).unwrap();
    let admin_addr = format!("{:?}", admin.address());

    // First registration
    let r1 = register_deployed_coop_logic(
        &svc, "Determinism Test".into(),
        env.loan_machine_address.clone(),
        admin_addr.clone(),
    ).await.expect("first register");

    // Second registration with identical args should fail with AlreadyRegistered,
    // proving coopId is purely a function of (name, loanMachine).
    let r2 = register_deployed_coop_logic(
        &svc, "Determinism Test".into(),
        env.loan_machine_address.clone(),
        admin_addr,
    ).await;

    assert!(r2.is_err(),
        "registering identical (name, loan_machine) twice MUST fail — \
         if it succeeds, coopId is non-deterministic and the simulate-vs-send \
         timestamp bug has returned. First coopId was {}", r1.coop_id_hex);
}

/// Catches: contract call returning empty bytes when ABI doesn't match.
/// Symptom we saw: "buffer overrun while deserializing". This test ensures
/// pointing the service at a non-registry address fails clearly, not cryptically.
#[tokio::test]
async fn register_against_loan_machine_address_fails_clearly() {
    use loan_machine_core::services::coop_deployment::CoopDeploymentService;
    use secrecy::SecretString;

    let env = common::get_deployed().await;

    // Point CoopDeploymentService at a LoanMachine address instead of the registry.
    // This simulates the "wrong .env value" failure mode.
    let bad_service = CoopDeploymentService::new(
        SecretString::from(env.platform_admin_key_hex.clone()),
        &env.loan_machine_address,    // ← wrong contract type
        &env.rpc_url,
        env.loan_machine_bytecode.clone(),
        &env.usdt_address,
    ).expect("ctor doesn't validate the address");

    use alloy::signers::local::PrivateKeySigner;
    use std::str::FromStr;
    let admin = PrivateKeySigner::from_str(env.platform_admin_key_hex.trim_start_matches("0x")).unwrap();

    let result = register_deployed_coop_logic(
        &bad_service, "X".into(),
        env.loan_machine_address.clone(),
        format!("{:?}", admin.address()),
    ).await;

    assert!(result.is_err(),
        "calling registerCoop on a LoanMachine address must fail; \
         if this succeeds, ABI matching is broken");
}

/// Catches: registered coop is actually retrievable via getCoopInstance.
/// Closes the loop the production bug exposed — a "successful" registration
/// where the coopId returned to the client didn't match the on-chain key.
#[tokio::test]
async fn registered_coop_is_readable_via_get_coop_instance() {
    use alloy::primitives::{Address, FixedBytes};
    use alloy::providers::ProviderBuilder;
    use alloy::signers::local::PrivateKeySigner;
    use loan_machine_core::services::blockchain::abis::CoopRegistry;
    use std::str::FromStr;

    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let admin = PrivateKeySigner::from_str(env.platform_admin_key_hex.trim_start_matches("0x")).unwrap();
    let result = register_deployed_coop_logic(
        &svc, "Readback Test".into(),
        env.loan_machine_address.clone(),
        format!("{:?}", admin.address()),
    ).await.expect("register");

    let coop_id = FixedBytes::<32>::from_str(&result.coop_id_hex).unwrap();
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(env.rpc_url.parse().unwrap());
    let registry_addr: Address = env.factory_address.parse().unwrap();
    let registry = CoopRegistry::new(registry_addr, provider);

    let stored_lm = registry.getCoopInstance(coop_id).call().await
        .expect("getCoopInstance must succeed for a freshly-registered coop — \
                 if it reverts with CoopNotFound, the returned coop_id is wrong")
        ._0;

    let expected_lm: Address = env.loan_machine_address.parse().unwrap();
    assert_eq!(stored_lm, expected_lm);
}

/// Catches: gas estimate falling to the fallback (3_500_000) silently.
/// If the founder isn't funded or the bytecode is malformed, estimate_gas
/// throws and we'd silently use the unwrap_or fallback. That fallback is
/// fine for prod safety but masks bugs in tests.
#[tokio::test]
async fn deploy_gas_estimate_is_realistic_not_fallback() {
    let env = common::get_deployed().await;
    let svc = make_service(&env).await;

    let bundle = prepare_create_coop_logic(
        &svc, "Gas Test".into(),
        format!("{:?}", env.approved_wallet),
        vec![
            format!("{:?}", env.approved_wallet),
            format!("{:?}", env.second_admin),
            format!("{:?}", env.third_admin),
        ],
        2,
    ).await.unwrap();

    let gas = u64::from_str_radix(bundle.gas_deploy.trim_start_matches("0x"), 16).unwrap();
    // The fallback is 3_500_000 exactly. A real estimate is 5M+.
    // If we get the fallback, estimate_gas failed silently — investigate.
    assert_ne!(gas, 3_500_000,
        "gas_deploy is the fallback constant — eth_estimateGas failed silently; \
         check that the deployer wallet is funded and the bytecode is valid");
    assert!(gas > 3_500_000,
        "real deploy gas should exceed the fallback; got {gas}");
}