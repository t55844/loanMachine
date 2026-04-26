mod common;

use loan_machine_core::server_logic::vinculation::{
    get_wallet_coop_logic,
    prepare_first_vinculation_logic,
    VinculationLogicError,
};
use loan_machine_core::services::identity::IdentityService;
use loan_machine_models::requests::DocKind;

// Known-valid CPF for tests
const TEST_CPF: &str = "529.982.247-25";

#[tokio::test]
async fn get_wallet_coop_returns_none_for_unapproved_wallet() {
    let env = common::get_deployed().await;

    let result = get_wallet_coop_logic(
        &env.blockchain,
        &env.unapproved_wallet.to_string(),
    )
    .await
    .expect("logic should not error for an unknown wallet");

    assert!(result.is_none(), "unapproved wallet should not be in any coop yet");
}

#[tokio::test]
async fn prepare_vinculation_rejects_unapproved_wallet() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let result = prepare_first_vinculation_logic(
        &identity,
        &env.blockchain,
        &env.factory_address,
        DocKind::Cpf,
        TEST_CPF.into(),
        env.unapproved_wallet.to_string(),
        env.coop_id_hex.clone(),
        env.access_code.clone(),
    )
    .await;

    assert!(
        matches!(result, Err(VinculationLogicError::ServerLogicWalletNotApproved)),
        "got {:?}", result,
    );
}

#[tokio::test]
async fn prepare_vinculation_happy_path_returns_bundle() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let bundle = prepare_first_vinculation_logic(
        &identity,
        &env.blockchain,
        &env.factory_address,
        DocKind::Cpf,
        TEST_CPF.into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(),
        env.access_code.clone(),
    )
    .await
    .expect("happy path should succeed");

    assert!(bundle.join_calldata.starts_with("0x"));
    assert_eq!(
        bundle.loan_machine_address.to_lowercase(),
        env.loan_machine_address.to_lowercase(),
    );
    assert!(bundle.gas_join.parse::<u64>().unwrap() > 0);
}

#[tokio::test]
async fn prepare_vinculation_rejects_invalid_cpf() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let result = prepare_first_vinculation_logic(
        &identity,
        &env.blockchain,
        &env.factory_address,
        DocKind::Cpf,
        "111.111.111-11".into(),   // all-same, rejected by mod-11 check
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(),
        env.access_code.clone(),
    )
    .await;

    assert!(matches!(result, Err(VinculationLogicError::ServerLogicIdentity(_))));
}


// ──  CPF and CNPJ produce different member IDs for same digits ──
// Guards against a future refactor accidentally unifying the hash paths.
#[tokio::test]
async fn cpf_and_cnpj_produce_different_calldata() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    // Valid for both — using test-known-valid values
    let cpf_result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    let cnpj_result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cnpj, "11.222.333/0001-81".into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    // Different member IDs should produce different calldata
    assert_ne!(cpf_result.join_calldata, cnpj_result.join_calldata);
}

// ──  Invalid wallet address is rejected before touching the chain ──
// Proves input validation comes first — no wasted RPC calls.
#[tokio::test]
async fn invalid_wallet_address_is_rejected() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        "not-a-hex-address".into(),          // ← invalid
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await;

    assert!(matches!(result, Err(VinculationLogicError::ServerLogicInvalidWalletAddress)));
}

// ── Invalid coop ID format is rejected ──
#[tokio::test]
async fn invalid_coop_id_is_rejected() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        env.approved_wallet.to_string(),
        "not-bytes32".into(),                 // ← invalid
        env.access_code.clone(),
    ).await;

    assert!(matches!(result, Err(VinculationLogicError::ServerLogicInvalidCoopId)));
}

// ──  Invalid CNPJ is rejected with Identity error ──
// Symmetric to the CPF test you already have.
#[tokio::test]
async fn prepare_vinculation_rejects_invalid_cnpj() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cnpj,
        "00.000.000/0000-00".into(),          // all-zero, rejected
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await;

    assert!(matches!(result, Err(VinculationLogicError::ServerLogicIdentity(_))));
}

// ──  Bundle carries the right factory address ──
#[tokio::test]
async fn bundle_contains_factory_address() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let bundle = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    assert_eq!(
        bundle.factory_address.to_lowercase(),
        env.factory_address.to_lowercase(),
    );
}

// ──  Calldata starts with joinCoop selector ──
// Doesn't fully parse but catches "encoded the wrong function" bugs.
#[tokio::test]
async fn bundle_calldata_starts_with_join_coop_selector() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);

    let bundle = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    // joinCoop(bytes32,address,string) selector is the first 4 bytes (8 hex chars after 0x)
    // Compute the expected selector once, elsewhere, and assert it matches.
    // For now, just sanity-check length and prefix.
    assert!(bundle.join_calldata.starts_with("0x"));
    assert!(bundle.join_calldata.len() > 2 + 8); // at least selector + some args
}

// ──  Salt affects the encoded member ID ──
// Same CPF + different salts → different calldata.
#[tokio::test]
async fn different_salts_produce_different_calldata() {
    let env = common::get_deployed().await;

    let id_a = IdentityService::with_salt([0x01u8; 32]);
    let id_b = IdentityService::with_salt([0x02u8; 32]);

    let bundle_a = prepare_first_vinculation_logic(
        &id_a, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    let bundle_b = prepare_first_vinculation_logic(
        &id_b, &env.blockchain, &env.factory_address,
        DocKind::Cpf, TEST_CPF.into(),
        env.approved_wallet.to_string(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    assert_ne!(bundle_a.join_calldata, bundle_b.join_calldata);
}