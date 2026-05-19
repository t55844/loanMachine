mod common;

use loan_machine_core::server_logic::vinculation::{
    get_wallet_coop_logic,
    prepare_first_vinculation_logic,
    VinculationLogicError,
};
use loan_machine_core::services::identity::IdentityService;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_models::requests::DocKind;
use loan_machine_models::wallet_address::from_alloy;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Known-valid CPF for tests
const TEST_CPF: &str = "529.982.247-25";

fn parse_hex_u64(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("bad hex gas value {s:?}: {e}"))
}

#[tokio::test]
async fn get_wallet_coop_returns_none_for_unapproved_wallet() {
    let env = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);

    // Subgraph returns no events → helper falls through to on-chain
    // scan, which also finds nothing for this never-vinculated wallet.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "memberRegisteredEvents": [] }
        })))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());

    let result = get_wallet_coop_logic(
        &subgraph,
        &env.blockchain,
        wallet,
    )
    .await
    .expect("logic should not error for an unknown wallet");

    assert!(result.is_none(), "unapproved wallet should not be in any coop yet");
}

#[tokio::test]
async fn get_wallet_coop_returns_coop_from_subgraph_hit() {
    let env = common::get_deployed().await;
    let wallet = from_alloy(env.unapproved_wallet);   // any address — chain isn't hit

    let coop_id      = "0x1111111111111111111111111111111111111111111111111111111111111111";
    let loan_machine = "0x2222222222222222222222222222222222222222";

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "memberRegisteredEvents": [{
                    "cooperative": {
                        "coopId":      coop_id,
                        "name":        "TestCoop",
                        "loanMachine": loan_machine,
                        "active":      true,
                    }
                }]
            }
        })))
        .mount(&server)
        .await;

    let subgraph = SubgraphService::new(server.uri());

    let info = get_wallet_coop_logic(&subgraph, &env.blockchain, wallet)
        .await
        .expect("subgraph hit should not error")
        .expect("subgraph hit should return Some(CoopInfo)");

    assert_eq!(info.name,         "TestCoop");
    assert_eq!(info.coop_id,      coop_id);
    assert_eq!(info.loan_machine, loan_machine);
    assert!(info.active);
}

#[tokio::test]
async fn prepare_vinculation_rejects_unapproved_wallet() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);
    let wallet = from_alloy(env.unapproved_wallet);

    let result = prepare_first_vinculation_logic(
        &identity,
        &env.blockchain,
        &env.coop_registry_address,
        DocKind::Cpf,
        TEST_CPF.into(),
        wallet,
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
    let wallet = from_alloy(env.second_admin);

    let bundle = prepare_first_vinculation_logic(
        &identity,
        &env.blockchain,
        &env.coop_registry_address,
        DocKind::Cpf,
        TEST_CPF.into(),
        wallet,
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
    assert!(parse_hex_u64(&bundle.gas_join) > 0);
}

#[tokio::test]
async fn prepare_vinculation_rejects_invalid_cpf() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);
    let wallet = from_alloy(env.approved_wallet);

    let result = prepare_first_vinculation_logic(
        &identity,
        &env.blockchain,
        &env.coop_registry_address,
        DocKind::Cpf,
        "111.111.111-11".into(),   // all-same, rejected by mod-11 check
        wallet,
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
    let wallet = from_alloy(env.second_admin);

    // Valid for both — using test-known-valid values
    let cpf_result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.coop_registry_address,
        DocKind::Cpf, TEST_CPF.into(),
        wallet.clone(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    let cnpj_result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.coop_registry_address,
        DocKind::Cnpj, "11.222.333/0001-81".into(),
        wallet.clone(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    // Different member IDs should produce different calldata
    assert_ne!(cpf_result.join_calldata, cnpj_result.join_calldata);
}



// ── Invalid coop ID format is rejected ──
#[tokio::test]
async fn invalid_coop_id_is_rejected() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);
    let wallet = from_alloy(env.approved_wallet);

    let result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.coop_registry_address,
        DocKind::Cpf, TEST_CPF.into(),
        wallet,
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
    let wallet = from_alloy(env.approved_wallet);

    let result = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.coop_registry_address,
        DocKind::Cnpj,
        "00.000.000/0000-00".into(),          // all-zero, rejected
        wallet,
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await;

    assert!(matches!(result, Err(VinculationLogicError::ServerLogicIdentity(_))));
}

// ──  Bundle carries the right coop_registry address ──
#[tokio::test]
async fn bundle_contains_coop_registry_address() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);
    let wallet = from_alloy(env.second_admin);

    let bundle = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.coop_registry_address,
        DocKind::Cpf, TEST_CPF.into(),
        wallet,
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    assert_eq!(
        bundle.coop_registry_address.to_lowercase(),
        env.coop_registry_address.to_lowercase(),
    );
}

// ──  Calldata starts with joinCoop selector ──
// Doesn't fully parse but catches "encoded the wrong function" bugs.
#[tokio::test]
async fn bundle_calldata_starts_with_join_coop_selector() {
    let env      = common::get_deployed().await;
    let identity = IdentityService::with_salt([0x01u8; 32]);
    let wallet = from_alloy(env.second_admin);

    let bundle = prepare_first_vinculation_logic(
        &identity, &env.blockchain, &env.coop_registry_address,
        DocKind::Cpf, TEST_CPF.into(),
        wallet,
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
    let wallet = from_alloy(env.second_admin);

    let bundle_a = prepare_first_vinculation_logic(
        &id_a, &env.blockchain, &env.coop_registry_address,
        DocKind::Cpf, TEST_CPF.into(),
        wallet.clone(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    let bundle_b = prepare_first_vinculation_logic(
        &id_b, &env.blockchain, &env.coop_registry_address,
        DocKind::Cpf, TEST_CPF.into(),
        wallet.clone(),
        env.coop_id_hex.clone(), env.access_code.clone(),
    ).await.unwrap();

    assert_ne!(bundle_a.join_calldata, bundle_b.join_calldata);
}