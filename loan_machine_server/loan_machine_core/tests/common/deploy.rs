// core tests/common/deploy.rs
//
// Anvil + addresses live in a static OnceCell (one process, runtime-agnostic).
// BlockchainService is built per-test so its reqwest client is scoped to the
// current #[tokio::test] runtime and doesn't leak HTTP state across runtimes.

use std::sync::{Arc, Mutex};

use alloy::network::EthereumWallet;
use alloy::node_bindings::{Anvil, AnvilInstance};
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

use loan_machine_core::services::blockchain::deployable::{
    CoopRegistry, LoanMachine, MockUSDT,
};
use loan_machine_core::services::blockchain::BlockchainService;

use tokio::sync::OnceCell;

pub const TEST_ACCESS_CODE: &str = "test-access-code";

/// Long-lived state: Anvil process + on-chain addresses. Initialized once.
struct Deployed {
    rpc_url:              String,
    coop_registry_address:      String,
    loan_machine_address: String,
    usdt_address:         String,
    coop_id_hex:          String,
    access_code:          String,
    approved_wallet:      Address,
    unapproved_wallet:    Address,
    platform_admin_key_hex:  String,  
    second_admin:            Address,
    third_admin:             Address,
    loan_machine_bytecode:   Vec<u8>,
    _anvil:               AnvilInstance,
    platform_admin_lock: Arc<tokio::sync::Mutex<()>>,
}

/// What each test sees. Same field names as before, but `blockchain` is
/// rebuilt per test so its HTTP client lives on the test's own runtime.
pub struct DeployedEnv {
    pub rpc_url:              String,
    pub coop_registry_address:      String,
    pub loan_machine_address: String,
    pub usdt_address:         String,
    pub coop_id_hex:          String,
    pub access_code:          String,
    pub approved_wallet:      Address,
    pub unapproved_wallet:    Address,
    pub platform_admin_key_hex:  String,    // hex of admin1's private key
    pub second_admin:            Address,
    pub third_admin:             Address,
    pub loan_machine_bytecode:   Vec<u8>,
    pub blockchain:           Arc<BlockchainService>,
    pub platform_admin_lock: Arc<tokio::sync::Mutex<()>>,
}

static DEPLOYED: OnceCell<Deployed> = OnceCell::const_new();

pub async fn get_deployed() -> DeployedEnv {
    let d = DEPLOYED.get_or_init(deploy).await;

    let blockchain = BlockchainService::init(&d.rpc_url, &d.coop_registry_address)
        .await
        .expect("BlockchainService::init (per-test)");

    DeployedEnv {
        rpc_url:              d.rpc_url.clone(),
        coop_registry_address:      d.coop_registry_address.clone(),
        loan_machine_address: d.loan_machine_address.clone(),
        usdt_address:         d.usdt_address.clone(),
        coop_id_hex:          d.coop_id_hex.clone(),
        access_code:          d.access_code.clone(),
        approved_wallet:      d.approved_wallet,
        unapproved_wallet:    d.unapproved_wallet,
        platform_admin_key_hex:  d.platform_admin_key_hex.clone(),
        second_admin:            d.second_admin,
        third_admin:             d.third_admin,
        loan_machine_bytecode:   d.loan_machine_bytecode.clone(),
        blockchain:           Arc::new(blockchain),
        platform_admin_lock: d.platform_admin_lock.clone(),
    }
}

async fn deploy() -> Deployed {
    let anvil = Anvil::new()
        .args(["--accounts", "10", "--disable-code-size-limit"])
        .spawn();

    let rpc_url = anvil.endpoint();

    let admin1:   PrivateKeySigner = anvil.keys()[0].clone().into();
    let admin2:   PrivateKeySigner = anvil.keys()[1].clone().into();
    let admin3:   PrivateKeySigner = anvil.keys()[2].clone().into();
    let member:   PrivateKeySigner = anvil.keys()[3].clone().into();
    let stranger: PrivateKeySigner = anvil.keys()[4].clone().into();

    let admin1_addr   = admin1.address();
    let admin2_addr   = admin2.address();
    let admin3_addr   = admin3.address();
    let member_addr   = member.address();
    let stranger_addr = stranger.address();

    let provider_admin1 = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(admin1.clone()))
        .on_http(rpc_url.parse().unwrap());

    let usdt = MockUSDT::deploy(provider_admin1.clone()).await
        .expect("deploy MockUSDT");
    let usdt_address = *usdt.address();

    let loan_machine = LoanMachine::deploy(provider_admin1.clone(), usdt_address)
        .await
        .expect("deploy LoanMachine");
    let loan_machine_address = *loan_machine.address();

    loan_machine
        .initializeMultisig(
            vec![admin1_addr, admin2_addr, admin3_addr],
            U256::from(2u64),
            TEST_ACCESS_CODE.to_string(),
        )
        .send().await.expect("send initializeMultisig")
        .watch().await.expect("mine initializeMultisig");

    let registry = CoopRegistry::deploy(provider_admin1.clone()).await
        .expect("deploy CoopRegistry");
    let registry_address = *registry.address();

    let coop_id: FixedBytes<32> = registry
        .registerCoop("Test Coop".into(), loan_machine_address)
        .from(admin1_addr)
        .call().await.expect("simulate registerCoop")
        .coopId;

    registry
        .registerCoop("Test Coop".into(), loan_machine_address)
        .from(admin1_addr)
        .send().await.expect("send registerCoop")
        .watch().await.expect("mine registerCoop");

    let proposal_id: U256 = loan_machine
        .bootstrapApproveWallet(member_addr)
        .from(admin1_addr)
        .call().await.expect("simulate bootstrapApproveWallet")
        .proposalId;

    loan_machine
        .bootstrapApproveWallet(member_addr)
        .from(admin1_addr)
        .send().await.expect("send bootstrapApproveWallet")
        .watch().await.expect("mine bootstrapApproveWallet");

    for signer in [admin2, admin3] {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(rpc_url.parse().unwrap());

        let lm_as_admin = LoanMachine::new(loan_machine_address, provider);

        lm_as_admin
            .confirmProposal(proposal_id)
            .send().await.expect("send confirmBootstrapApproval")
            .watch().await.expect("mine confirmBootstrapApproval");
    }
    let platform_admin_key_hex = format!(
        "0x{}",
        hex::encode(admin1.to_bytes())
    );
    let loan_machine_bytecode = LoanMachine::BYTECODE.to_vec();

    Deployed {
        rpc_url:              rpc_url.clone(),
        coop_registry_address:      registry_address.to_string(),
        loan_machine_address: loan_machine_address.to_string(),
        usdt_address:         usdt_address.to_string(),
        coop_id_hex:          format!("0x{}", hex::encode(coop_id)),
        access_code:          TEST_ACCESS_CODE.to_string(),
        approved_wallet:      member_addr,
        unapproved_wallet:    stranger_addr,
        _anvil:               anvil,
        platform_admin_key_hex:  platform_admin_key_hex,
        second_admin:            admin2_addr,
        third_admin:             admin3_addr,
        loan_machine_bytecode:   loan_machine_bytecode,
        platform_admin_lock: Arc::new(tokio::sync::Mutex::new(())),
    }
}
