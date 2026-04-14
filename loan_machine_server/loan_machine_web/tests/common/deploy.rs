// web tests/common/deploy.rs

use std::sync::OnceLock;
use std::process::Command;
use std::sync::Arc;
use alloy::node_bindings::{Anvil, AnvilInstance};
use alloy::primitives::{Address, FixedBytes};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::network::EthereumWallet;
use super::hash_error::decode_revert_hash_error;

// bring in the ABI bindings directly — same ones FactoryService uses
use loan_machine_core::services::blockchain::abis::LoanMachine;


fn decode_or_panic(context: &str, err: impl std::fmt::Debug) -> !{
    let msg = format!("{err:?}");
    if let Some(start) = msg.find("0x") {
        let hex_end = msg[start..]
            .find(|c: char| !c.is_ascii_hexdigit() && c != 'x')
            .map(|i| start + i)
            .unwrap_or(msg.len());
        let decoded = decode_revert_hash_error(&msg[start..hex_end]);
        panic!("{context}: {decoded}");
    }
    panic!("{context}: {msg}");
}
pub struct DeployedEnv {
    pub rpc_url:           String,
    pub factory_address:   String,
    pub loan_machine_addr: String,
    pub coop_id:           String,
    _anvil: AnvilInstance,
}

// anvil account 0 — platform admin
const ADMIN_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
// anvil account 1 — coop admin (the one deploy.js passes to deployCoop)
const COOP_ADMIN_KEY: &str =
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
// anvil account 2 — member1
const MEMBER1_KEY: &str =
    "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a";

static DEPLOYED: OnceLock<DeployedEnv> = OnceLock::new();

pub fn get_deployed() -> &'static DeployedEnv {
    DEPLOYED.get_or_init(|| {
        std::thread::spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(deploy())
        })
        .join()
        .expect("deploy thread panicked")
    })
}

async fn deploy() -> DeployedEnv {
    let anvil = Anvil::new()
        .args(["--accounts", "10", "--disable-code-size-limit"])
        .spawn();

    let rpc_url = anvil.endpoint();

    // deploy.js only deploys contracts now — no state setup
    let output = Command::new("cmd")
        .args(["/C", "npx hardhat run scripts/deploy.js --network localhost"])
        .env("RPC_URL", &rpc_url)
        .current_dir("../../hardhat")
        .output()
        .expect("failed to run deploy script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!(">>> deploy stdout:\n{stdout}");
    eprintln!(">>> deploy stderr:\n{stderr}");

    let loan_machine_addr = parse_field(&stdout, "LoanMachine");
    let coop_id           = parse_field(&stdout, "Coop ID");

    setup_test_state(&rpc_url, &loan_machine_addr, &coop_id).await;

    DeployedEnv {
        rpc_url,
        factory_address:   parse_field(&stdout, "Factory"),
        loan_machine_addr,
        coop_id,
        _anvil: anvil,
    }
}

async fn setup_test_state(
    rpc_url:           &str,
    loan_machine_addr: &str,
    _coop_id:          &str,
) {
 let lm_addr: Address = loan_machine_addr.parse().unwrap();

    let coop_signer: PrivateKeySigner = COOP_ADMIN_KEY.parse().unwrap();
    let coop_wallet = EthereumWallet::from(coop_signer);
    let coop_provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(coop_wallet)
            .on_http(rpc_url.parse().unwrap()),
    );

    let m1_signer: PrivateKeySigner = MEMBER1_KEY.parse().unwrap();
    let m1_wallet = EthereumWallet::from(m1_signer);
    let m1_provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(m1_wallet)
            .on_http(rpc_url.parse().unwrap()),
    );

    let member1: Address = "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".parse().unwrap();
    let member2: Address = "0x90F79bf6EB2c4f870365E785982E1f101E93b906".parse().unwrap();

    let lm = LoanMachine::new(lm_addr, coop_provider.clone());

    // approve member1 — coopAdmin signs
    lm.approveWallet(member1).send().await
        .unwrap_or_else(|e| decode_or_panic("approveWallet member1 send", e))
        .watch().await
        .unwrap_or_else(|e| decode_or_panic("approveWallet member1 watch", e));

    // joinCoop — member1 signs
    let lm_m1 = LoanMachine::new(lm_addr, m1_provider.clone());
    lm_m1.joinCoop(1u32, member1, "test123".to_string()).send().await
        .unwrap_or_else(|e| decode_or_panic("joinCoop member1 send", e))
        .watch().await
        .unwrap_or_else(|e| decode_or_panic("joinCoop member1 watch", e));

    // approve member2 — coopAdmin signs
    lm.approveWallet(member2).send().await
        .unwrap_or_else(|e| decode_or_panic("approveWallet member2 send", e))
        .watch().await
        .unwrap_or_else(|e| decode_or_panic("approveWallet member2 watch", e));
}

pub fn coop_id() -> FixedBytes<32> {
    get_deployed().coop_id.parse().unwrap()
}

pub fn loan_machine_addr() -> Address {
    get_deployed().loan_machine_addr.parse().unwrap()
}

fn parse_field(output: &str, label: &str) -> String {
    output
        .lines()
        .find(|l| l.contains(':') && l.split(':').next()
            .map_or(false, |p| p.contains(label)))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| panic!("could not find {label} in deploy output"))
}