//! Wallet → CoopInfo lookup.
//!
//! Subgraph first (one query, O(1)); on-chain scan as fallback for vinculations
//! younger than the indexing lag. In steady state the fallback never runs.

use alloy::primitives::Address;
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};

use loan_machine_models::responses::CoopInfo;
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};
use crate::services::blockchain::{BlockchainError, BlockchainService};
use crate::services::blockchain::abis::{CoopRegistry, LoanMachine};
use crate::services::subgraph::SubgraphService;

pub async fn resolve_wallet_coop(
    subgraph: &SubgraphService,
    blockchain: &BlockchainService,
    wallet: &WalletAddress,
) -> Result<Option<CoopInfo>, BlockchainError> {
    if let Some(info) = subgraph_lookup(subgraph, wallet).await {
        return Ok(Some(info));
    }
    on_chain_scan(blockchain, to_alloy(wallet)).await
}

// ── subgraph path ─────────────────────────────────────────

const QUERY: &str = r#"
query CoopByWallet($wallet: Bytes!) {
  memberRegisteredEvents(
    where: { wallet: $wallet }
    first: 1
    orderBy: blockTimestamp
    orderDirection: desc
  ) {
    cooperative {
      coopId
      name
      loanMachine
      active
    }
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> { wallet: &'a str }

#[derive(Deserialize)]
struct CoopRow {
    #[serde(rename = "coopId")]      coop_id:      String,
                                     name:         String,
    #[serde(rename = "loanMachine")] loan_machine: String,
                                     active:       bool,
}

#[derive(Deserialize)]
struct EventRow { cooperative: CoopRow }

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "memberRegisteredEvents")]
    events: Vec<EventRow>,
}

async fn subgraph_lookup(
    subgraph: &SubgraphService,
    wallet: &WalletAddress,
) -> Option<CoopInfo> {
    let wallet_hex = wallet.to_string();
    let vars = Vars { wallet: &wallet_hex };

    let data: Data = subgraph.query(QUERY, vars).await.ok()?;
    let coop = data.events.into_iter().next()?.cooperative;

    Some(CoopInfo {
        coop_id:      coop.coop_id,
        name:         coop.name,
        loan_machine: coop.loan_machine,
        active:       coop.active,
    })
}

// ── chain fallback ────────────────────────────────────────

// Concurrent scan: all coop membership checks fire in parallel (O(1) RTT
// latency, O(n) work). Long-term fix is a `getCoopForWallet` view on
// CoopRegistry for true O(1), but this is good enough while the coop count
// is small and this path only runs on subgraph indexing-lag fallbacks.
async fn on_chain_scan(
    blockchain: &BlockchainService,
    wallet: Address,
) -> Result<Option<CoopInfo>, BlockchainError> {
    let registry      = &blockchain.coop_registry;
    let provider      = registry.provider.clone();
    let registry_addr = registry.coop_registry_address;

    let coop_ids = CoopRegistry::new(registry_addr, provider.clone())
        .getAllCoops().call().await
        .map_err(BlockchainError::from_call)?._0;

    let mut futs: FuturesUnordered<_> = coop_ids.iter().map(|&coop_id| {
        let provider = provider.clone();
        async move {
            let lm_addr = CoopRegistry::new(registry_addr, provider.clone())
                .getCoopInstance(coop_id).call().await
                .map_err(BlockchainError::from_call)?._0;
            let vinculated = LoanMachine::new(lm_addr, provider)
                .isWalletVinculated(wallet).call().await
                .map_err(BlockchainError::from_call)?._0;
            Ok::<_, BlockchainError>(vinculated.then_some(coop_id))
        }
    }).collect();

    while let Some(result) = futs.next().await {
        if let Some(coop_id) = result? {
            return Ok(Some(registry.get_coop_info(coop_id).await?));
        }
    }
    Ok(None)
}