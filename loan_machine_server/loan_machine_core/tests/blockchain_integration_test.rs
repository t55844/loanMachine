// blockchain_integration_test.rs

mod common;
use common::*;
use alloy::primitives::FixedBytes;
use std::sync::Arc;
// Each test calls this to get a fresh provider+service
// bound to its own runtime — no cross-runtime I/O handles.
pub fn make_factory_service() -> loan_machine_core::services::blockchain::factory::FactoryService {
    let env = get_deployed();

    let provider = loan_machine_core::services::blockchain::provider::build_provider(&env.rpc_url)
        .expect("failed to build provider");

    loan_machine_core::services::blockchain::factory::FactoryService::new(
        Arc::new(provider),
        env.factory_address.parse().unwrap(),
    )
}

#[tokio::test]
async fn get_loan_machine_returns_correct_address() {
    // provider built here → owned by THIS runtime → no cross-runtime I/O
    let service = make_factory_service();

    let addr = service
        .get_loan_machine(coop_id())
        .await
        .unwrap();

    assert_eq!(addr, loan_machine_addr());
}

#[tokio::test]
async fn get_wallet_coop_finds_member1() {
    let service = make_factory_service();
    let wallet = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".parse().unwrap();

    let found = service
        .get_wallet_coop(wallet)
        .await
        .unwrap();

    assert_eq!(found, coop_id());
}

#[tokio::test]
async fn get_wallet_coop_returns_zero_for_unknown() {
    let service = make_factory_service();
    let unknown = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720".parse().unwrap();

    let result = service
        .get_wallet_coop(unknown)
        .await
        .unwrap();

    assert_eq!(result, FixedBytes::<32>::ZERO);
}