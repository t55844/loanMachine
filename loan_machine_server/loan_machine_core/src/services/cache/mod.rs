use moka::future::Cache;
use std::time::Duration;
use loan_machine_models::wallet_address::WalletAddress;
use crate::services::subgraph::{SubgraphError, SubgraphService};
use crate::server_logic::subgraph_queries::membership::is_member;

#[derive(Clone)]
pub struct MembershipCache {
    inner: Cache<(WalletAddress, String), bool>,
}

impl MembershipCache {
    pub fn new() -> Self {
        Self {
            inner: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(60))
                .build(),
        }
    }

    pub async fn is_member(
        &self,
        subgraph: &SubgraphService,
        wallet:   &WalletAddress,
        coop_id:  &str,
    ) -> Result<bool, SubgraphError> {
        let key = (wallet.clone(), coop_id.to_string());
        if let Some(v) = self.inner.get(&key).await {
            return Ok(v);
        }
        let v = is_member(subgraph, coop_id, wallet).await?;
        self.inner.insert(key, v).await;
        Ok(v)
    }

    pub async fn invalidate(&self, wallet: &WalletAddress, coop_id: &str) {
        self.inner.invalidate(&(wallet.clone(), coop_id.to_string())).await;
    }
}