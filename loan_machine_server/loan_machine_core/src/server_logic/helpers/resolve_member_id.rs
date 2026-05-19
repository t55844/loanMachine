//! Wallet → memberId lookup within a specific coop.
//! Subgraph first, on-chain fallback. Returns None if the wallet is not
//! vinculated to anyone in this coop.

use alloy::primitives::{Address, B256, FixedBytes};
use serde::{Deserialize, Serialize};

use crate::services::blockchain::abis::LoanMachine;
use crate::services::blockchain::provider::Provider;
use crate::services::blockchain::BlockchainError;
use crate::services::subgraph::SubgraphService;
use loan_machine_models::wallet_address::{to_alloy, WalletAddress};

pub async fn resolve_member_id(
    subgraph: &SubgraphService,
    provider: &Provider,
    loan_machine_addr: Address,
    wallet: &WalletAddress,
) -> Result<Option<FixedBytes<32>>, BlockchainError> {
    if let Some(id) = subgraph_lookup(subgraph, loan_machine_addr, wallet).await {
        return Ok(Some(id));
    }
    let id = LoanMachine::new(loan_machine_addr, provider.clone())
        .getMemberId(to_alloy(wallet))
        .call().await
        .map_err(BlockchainError::from_call)?._0;
    Ok((id != B256::ZERO).then_some(id))
}

const QUERY: &str = r#"
query MemberIdByWallet($coopId: String!, $wallet: Bytes!) {
  memberRegisteredEvents(
    where: { cooperative: $coopId, wallet: $wallet }
    first: 1
  ) {
    memberId
  }
}"#;

#[derive(Serialize)]
struct Vars<'a> {
    #[serde(rename = "coopId")] coop_id: String,
                                wallet:  &'a str,
}

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "memberId")] member_id: String,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "memberRegisteredEvents")]
    events: Vec<Row>,
}

async fn subgraph_lookup(
    subgraph: &SubgraphService,
    loan_machine_addr: Address,
    wallet: &WalletAddress,
) -> Option<FixedBytes<32>> {
    let coop_id    = format!("{loan_machine_addr:#x}").to_lowercase();
    let wallet_hex = wallet.to_string();             // ← Display gives "0x…"
    let vars = Vars { wallet: &wallet_hex, coop_id };

    let data: Data = subgraph.query(QUERY, vars).await.ok()?;
    row_to_id(data.events.into_iter().next()?)
}

fn row_to_id(row: Row) -> Option<FixedBytes<32>> {
    row.member_id.parse::<B256>().ok()
}