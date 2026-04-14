// core tests/common/deploy.rs

use std::sync::{Arc, OnceLock};
use std::process::Command;
use alloy::node_bindings::{Anvil, AnvilInstance};

// Only plain data crosses the thread boundary.
// Strings and addresses are trivially Send + Sync — no unsafe needed.
pub struct DeployedEnv {
    pub rpc_url:          String,
    pub factory_address:  String,
    pub loan_machine_addr: String,
    pub coop_id:          String,
    _anvil: AnvilInstance, // kept alive for the whole test binary lifetime
}

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

    DeployedEnv {
        rpc_url,
        factory_address:   parse_field(&stdout, "Factory"),
        loan_machine_addr: parse_field(&stdout, "LoanMachine"),
        coop_id:           parse_field(&stdout, "Coop ID"),
        _anvil:            anvil,
    }
}


pub fn coop_id() -> alloy::primitives::FixedBytes<32> {
    get_deployed().coop_id.parse().unwrap()
}

pub fn loan_machine_addr() -> alloy::primitives::Address {
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