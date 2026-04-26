// src/config/chain_config/mod.rs

pub struct ChainConfig {
    pub rpc_url: String,
    pub chain_id: u64,
}

impl ChainConfig {
    pub fn from_env() -> Self {
        Self{
            rpc_url: std::env::var("RPC_URL")
                .expect("RPC_URL must be set"),
            chain_id: std::env::var("CHAIN_ID")
                .unwrap_or_else(|_| "31337".into())
                .parse::<u64>()
                .expect("CHAIN_ID must be a valid number")
        }
    }
}