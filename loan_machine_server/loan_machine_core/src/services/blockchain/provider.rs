// src/services/blockchain/provider.rs
//
// Only responsible for: building the RPC provider.
// Nothing else lives here.

use alloy::{
    providers::{ProviderBuilder, RootProvider},
    transports::http::{Client, Http},
};

use super::BlockchainError;

pub type Provider = RootProvider<Http<Client>>;

pub fn build_provider(rpc_url: &str) -> Result<Provider, BlockchainError> {
    let url = rpc_url
        .parse()
        .map_err(|e| BlockchainError::InvalidRpcUrl(format!("{e}")))?;

    Ok(ProviderBuilder::new().on_http(url))
}