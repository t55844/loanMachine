// Deploys the full stack to a local Anvil node for development.
//
// Usage:
//   1. In one terminal: anvil
//   2. In another:      cargo run --bin deploy_local --features deployable
//   3. Copy the printed addresses into .env
//   4. cargo leptos watch

use alloy::network::EthereumWallet;
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;

use loan_machine_core::services::blockchain::deployable::{
    CoopRegistry, MockUSDT,
};

// Anvil's default first account (deterministic across restarts)
const ANVIL_KEY: &str = "0xed0c1ddb10dbcd0fd69f89521d999520170d8ef995fb4664ded9ac53efaef603";
const RPC_URL:   &str = "http://localhost:8545";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: PrivateKeySigner = ANVIL_KEY.parse()?;
    let admin  = signer.address();
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(signer))
        .on_http(RPC_URL.parse()?);

    let usdt     = MockUSDT::deploy(provider.clone()).await?;
    let registry = CoopRegistry::deploy(provider.clone()).await?;

    println!("MockUSDT:     {}", usdt.address());
    println!("CoopRegistry: {}", registry.address());

    // ── Patch .env in place ───────────────────────────────────
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("Cargo.lock").exists())
        .unwrap()
        .join(".env");

    let updates = [
        ("USDC_ADDRESS",               usdt.address().to_string()),
        ("FACTORY_ADDRESS",            registry.address().to_string()),
        ("PLATFORM_ADMIN_PRIVATE_KEY", ANVIL_KEY.to_string()),
    ];

    let content = std::fs::read_to_string(&env_path).unwrap_or_default();
    let patched = content.lines().map(|line| {
        if let Some((k, _)) = line.split_once('=') {
            if let Some((_, v)) = updates.iter().find(|(uk, _)| *uk == k.trim()) {
                return format!("{}={}", k.trim(), v);
            }
        }
        line.to_string()
    }).collect::<Vec<_>>().join("\n") + "\n";

    std::fs::write(&env_path, patched)?;
    println!("✓ .env updated");

    Ok(())
}