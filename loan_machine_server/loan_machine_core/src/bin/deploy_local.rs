// Deploys the full stack to a local Anvil node for development.
//
// Usage:
//   1. In one terminal: anvil
//   2. In another:      cargo run --bin deploy_local --features deployable
//   3. Copy the printed addresses into .env
//   4. cargo leptos watch

use alloy::network::EthereumWallet;
use alloy::primitives::U256;
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

use loan_machine_core::services::blockchain::deployable::{
    CoopRegistry, LoanMachine, MockUSDT,
};

// Anvil's default first account (deterministic across restarts)
const ANVIL_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const RPC_URL:   &str = "http://localhost:8545";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = ANVIL_KEY.parse()?;
    let admin  = signer.address();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(RPC_URL.parse()?);

    println!("Deploying as admin: {admin}");

    let usdt = MockUSDT::deploy(provider.clone()).await?;
    println!("MockUSDT:     {}", usdt.address());

    let loan_machine = LoanMachine::deploy(provider.clone(), *usdt.address()).await?;
    println!("LoanMachine:  {}", loan_machine.address());

    loan_machine
        .initializeMultisig(vec![admin], U256::from(1u64), "dev-code".into())
        .from(admin)
        .send().await?.watch().await?;

    let registry = CoopRegistry::deploy(provider.clone()).await?;
    println!("CoopRegistry: {}", registry.address());

    let coop_id = registry
        .registerCoop("Dev Coop".into(), *loan_machine.address())
        .from(admin)
        .call().await?.coopId;

    registry
        .registerCoop("Dev Coop".into(), *loan_machine.address())
        .from(admin)
        .send().await?.watch().await?;

    println!("Coop ID:      0x{}", hex::encode(coop_id));
    println!();
    println!("── Paste into .env ────────────────────────────────");
    println!("FACTORY_ADDRESS={}", registry.address());
    println!("RPC_URL={RPC_URL}");
    println!("CHAIN_ID=31337");

    Ok(())
}